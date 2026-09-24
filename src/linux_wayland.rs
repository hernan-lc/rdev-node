//! Linux device backend for Wayland sessions. Physical events are read through
//! evdev; injected events go through a dedicated uinput device. Neither path
//! asks the compositor for a new grant on each call.

mod keymap {
  include!("linux_keymap.rs");
}

use evdev::uinput::VirtualDevice;
use evdev::{
  AttributeSet, Device, EventType as EvType, InputEvent as EvEvent, KeyCode as EvKey,
  RelativeAxisCode,
};
use nix::poll::{poll, PollFd, PollFlags};
use once_cell::sync::Lazy;
use rdev::{Button, Event, EventType};
use std::collections::HashSet;
use std::fs;
use std::io;
use std::os::fd::AsFd;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime};
use x11rb::connection::Connection;
use x11rb::protocol::xproto::ConnectionExt;

const VIRTUAL_DEVICE_NAME: &str = "rdev-node virtual input";
static VIRTUAL_DEVICE: Lazy<Mutex<Option<VirtualDevice>>> = Lazy::new(|| Mutex::new(None));

pub fn is_wayland() -> bool {
  std::env::var_os("WAYLAND_DISPLAY").is_some()
    && std::env::var("XDG_SESSION_TYPE").is_ok_and(|kind| kind == "wayland")
}

fn device_paths() -> io::Result<Vec<PathBuf>> {
  let mut paths = Vec::new();
  for entry in fs::read_dir("/dev/input")? {
    let entry = entry?;
    if entry.file_name().to_string_lossy().starts_with("event") {
      paths.push(entry.path());
    }
  }
  paths.sort();
  Ok(paths)
}

fn relevant(device: &Device) -> bool {
  device.supported_keys().is_some_and(|keys| {
    keys.contains(EvKey::KEY_A) || keys.contains(EvKey::KEY_ENTER) || keys.contains(EvKey::BTN_LEFT)
  }) || device.supported_relative_axes().is_some_and(|axes| {
    axes.contains(RelativeAxisCode::REL_X) || axes.contains(RelativeAxisCode::REL_Y)
  }) || device.supported_absolute_axes().is_some_and(|axes| {
    axes.contains(evdev::AbsoluteAxisCode::ABS_X) || axes.contains(evdev::AbsoluteAxisCode::ABS_Y)
  })
}

fn open_devices() -> Result<Vec<(PathBuf, Device)>, String> {
  let paths = device_paths().map_err(|e| format!("Cannot enumerate /dev/input: {e}"))?;
  if paths.is_empty() {
    return Err("No /dev/input/event* devices were found".into());
  }
  let mut devices = Vec::new();
  let mut denied = Vec::new();
  for path in paths {
    match Device::open(&path) {
      Ok(device) => {
        if relevant(&device) {
          device
            .set_nonblocking(true)
            .map_err(|e| format!("Cannot configure {}: {e}", path.display()))?;
          devices.push((path, device));
        }
      }
      Err(error) if error.kind() == io::ErrorKind::PermissionDenied => denied.push(path),
      Err(error) => return Err(format!("Cannot open {}: {error}", path.display())),
    }
  }
  if !denied.is_empty() {
    return Err(format!(
      "Wayland global capture requires read access to /dev/input/event* ({} denied). Configure a persistent input-device permission before starting the listener.",
      denied.len()
    ));
  }
  if devices.is_empty() {
    return Err("No keyboard or pointer input devices were found".into());
  }
  Ok(devices)
}

pub fn preflight_capture() -> Result<(), String> {
  let _ = open_devices()?;
  let _ = PointerConnection::open()?;
  Ok(())
}

struct PointerConnection {
  conn: x11rb::rust_connection::RustConnection,
  root: u32,
}

impl PointerConnection {
  fn open() -> Result<Self, String> {
    let (conn, screen) = x11rb::connect(None)
      .map_err(|e| format!("Cannot connect to XWayland for absolute pointer coordinates: {e}"))?;
    let root = conn.setup().roots[screen].root;
    Ok(Self { conn, root })
  }

  fn location(&self) -> Result<(f64, f64), String> {
    let reply = self
      .conn
      .query_pointer(self.root)
      .map_err(|e| format!("Cannot query pointer location: {e}"))?
      .reply()
      .map_err(|e| format!("Cannot query pointer location: {e}"))?;
    Ok((f64::from(reply.root_x), f64::from(reply.root_y)))
  }

  fn warp(&self, x: i16, y: i16) -> Result<(), String> {
    self
      .conn
      .warp_pointer(0u32, self.root, 0, 0, 0, 0, x, y)
      .map_err(|e| format!("Cannot request XWayland pointer warp: {e}"))?
      .check()
      .map_err(|e| format!("XWayland pointer warp failed: {e}"))?;
    self
      .conn
      .flush()
      .map_err(|e| format!("Cannot flush XWayland pointer warp: {e}"))
  }
}

fn capture_event(
  event: EvEvent,
  pointer_location: &impl Fn() -> Result<(f64, f64), String>,
) -> Result<Option<Event>, String> {
  let event_type = if event.event_type() == EvType::KEY {
    let code = EvKey::new(event.code());
    let pressed = event.value() != 0;
    if (EvKey::BTN_LEFT.code()..=EvKey::BTN_TASK.code()).contains(&code.code()) {
      let button = match code {
        EvKey::BTN_LEFT => Button::Left,
        EvKey::BTN_RIGHT => Button::Right,
        EvKey::BTN_MIDDLE => Button::Middle,
        _ => Button::Unknown((code.code() - EvKey::BTN_LEFT.code()) as u8),
      };
      if pressed {
        EventType::ButtonPress(button)
      } else {
        EventType::ButtonRelease(button)
      }
    } else if (0x100..=0x15f).contains(&code.code()) {
      // Other BTN_* codes belong to touch, stylus, joystick or gamepad
      // devices; they are not keyboard keys in the public event model.
      return Ok(None);
    } else {
      let key = keymap::from_evdev(code);
      if pressed {
        EventType::KeyPress(key)
      } else {
        EventType::KeyRelease(key)
      }
    }
  } else if event.event_type() == EvType::RELATIVE {
    match RelativeAxisCode(event.code()) {
      RelativeAxisCode::REL_X | RelativeAxisCode::REL_Y => {
        let (x, y) = pointer_location()?;
        EventType::MouseMove { x, y }
      }
      RelativeAxisCode::REL_WHEEL => EventType::Wheel {
        delta_x: 0,
        delta_y: i64::from(event.value()),
      },
      RelativeAxisCode::REL_HWHEEL => EventType::Wheel {
        delta_x: i64::from(event.value()),
        delta_y: 0,
      },
      _ => return Ok(None),
    }
  } else if event.event_type() == EvType::ABSOLUTE
    && (event.code() == evdev::AbsoluteAxisCode::ABS_X.0
      || event.code() == evdev::AbsoluteAxisCode::ABS_Y.0)
  {
    let (x, y) = pointer_location()?;
    EventType::MouseMove { x, y }
  } else {
    return Ok(None);
  };
  Ok(Some(Event {
    event_type,
    name: None,
    time: SystemTime::now(),
  }))
}

pub fn listen(
  mut callback: impl FnMut(Event),
  should_stop: impl Fn() -> bool,
) -> Result<(), String> {
  let mut devices = open_devices()?;
  let pointer = PointerConnection::open()?;
  let mut last_scan = Instant::now();
  loop {
    if should_stop() {
      return Ok(());
    }
    let mut poll_fds: Vec<_> = devices
      .iter()
      .map(|(_, device)| PollFd::new(device.as_fd(), PollFlags::POLLIN))
      .collect();
    poll(&mut poll_fds, 100u16).map_err(|e| format!("Input poll failed: {e}"))?;
    let ready: Vec<usize> = poll_fds
      .iter()
      .enumerate()
      .filter_map(|(index, fd)| {
        fd.revents()
          .is_some_and(|flags| flags.contains(PollFlags::POLLIN))
          .then_some(index)
      })
      .collect();
    let disconnected: Vec<usize> = poll_fds
      .iter()
      .enumerate()
      .filter_map(|(index, fd)| {
        fd.revents()
          .is_some_and(|flags| {
            flags.intersects(PollFlags::POLLHUP | PollFlags::POLLERR | PollFlags::POLLNVAL)
          })
          .then_some(index)
      })
      .collect();
    drop(poll_fds);
    for index in ready {
      if disconnected.contains(&index) {
        continue;
      }
      let path = devices[index].0.clone();
      match devices[index].1.fetch_events() {
        Ok(events) => {
          for event in events {
            if should_stop() {
              return Ok(());
            }
            if let Some(event) = capture_event(event, &|| pointer.location())? {
              callback(event);
            }
          }
        }
        Err(error) if error.kind() == io::ErrorKind::WouldBlock => {}
        Err(error) => return Err(format!("Input device {} failed: {error}", path.display())),
      }
    }
    for index in disconnected.into_iter().rev() {
      devices.remove(index);
    }
    if last_scan.elapsed() >= Duration::from_secs(1) {
      let known: HashSet<_> = devices.iter().map(|(path, _)| path.clone()).collect();
      for path in device_paths().map_err(|e| format!("Cannot enumerate /dev/input: {e}"))? {
        if known.contains(&path) {
          continue;
        }
        let device = Device::open(&path)
          .map_err(|e| format!("Cannot open new input device {}: {e}", path.display()))?;
        if relevant(&device) {
          device
            .set_nonblocking(true)
            .map_err(|e| format!("Cannot configure {}: {e}", path.display()))?;
          devices.push((path, device));
        }
      }
      last_scan = Instant::now();
    }
  }
}

fn create_virtual_device() -> Result<VirtualDevice, String> {
  let mut keys = AttributeSet::<EvKey>::new();
  for key in keymap::SUPPORTED_KEYS {
    keys.insert(*key);
  }
  for button in [EvKey::BTN_LEFT, EvKey::BTN_RIGHT, EvKey::BTN_MIDDLE] {
    keys.insert(button);
  }
  let mut axes = AttributeSet::<RelativeAxisCode>::new();
  for axis in [
    RelativeAxisCode::REL_X,
    RelativeAxisCode::REL_Y,
    RelativeAxisCode::REL_WHEEL,
    RelativeAxisCode::REL_HWHEEL,
  ] {
    axes.insert(axis);
  }
  VirtualDevice::builder()
    .map_err(|e| format!("Cannot open /dev/uinput: {e}"))?
    .name(VIRTUAL_DEVICE_NAME)
    .with_keys(&keys)
    .map_err(|e| format!("Cannot configure virtual keys: {e}"))?
    .with_relative_axes(&axes)
    .map_err(|e| format!("Cannot configure virtual pointer: {e}"))?
    .build()
    .map_err(|e| format!("Cannot create virtual input device: {e}"))
}

fn move_pointer(device: &mut VirtualDevice, x: f64, y: f64) -> Result<(), String> {
  // Pointer queries below use XWayland coordinates, which may differ from
  // native Wayland logical output sizes when displays are scaled.
  let (width, height) =
    rdev::display_size().map_err(|e| format!("XWayland pointer geometry is unavailable: {e:?}"))?;
  if !x.is_finite()
    || !y.is_finite()
    || x < 0.0
    || y < 0.0
    || x >= width as f64
    || y >= height as f64
  {
    return Err("Mouse coordinates must be finite and within the display".into());
  }
  let pointer = PointerConnection::open()?;
  for _ in 0..8 {
    let (current_x, current_y) = pointer.location()?;
    let dx = (x - current_x).round() as i32;
    let dy = (y - current_y).round() as i32;
    if dx.abs() <= 1 && dy.abs() <= 1 {
      return Ok(());
    }
    device
      .emit(&[
        EvEvent::new(EvType::RELATIVE.0, RelativeAxisCode::REL_X.0, dx),
        EvEvent::new(EvType::RELATIVE.0, RelativeAxisCode::REL_Y.0, dy),
      ])
      .map_err(|e| format!("Cannot move virtual pointer: {e}"))?;
    std::thread::sleep(Duration::from_millis(15));
  }
  // Relative uinput motion is affected by compositor acceleration. XWayland
  // can provide an exact coordinate fallback when the native motion misses.
  let warp_x = i16::try_from(x.round() as i64).map_err(|_| "Mouse x is out of XWayland range")?;
  let warp_y = i16::try_from(y.round() as i64).map_err(|_| "Mouse y is out of XWayland range")?;
  pointer.warp(warp_x, warp_y)?;
  let (actual_x, actual_y) = pointer.location()?;
  if (actual_x - x).abs() <= 1.0 && (actual_y - y).abs() <= 1.0 {
    Ok(())
  } else {
    Err("Wayland pointer did not reach the requested absolute position".into())
  }
}

pub fn init_simulation() -> Result<(), String> {
  let mut slot = VIRTUAL_DEVICE.lock().unwrap_or_else(|e| e.into_inner());
  if slot.is_none() {
    *slot = Some(create_virtual_device()?);
  }
  Ok(())
}

pub fn simulate(event: EventType) -> Result<(), String> {
  init_simulation()?;
  let mut slot = VIRTUAL_DEVICE.lock().unwrap_or_else(|e| e.into_inner());
  let device = slot.as_mut().ok_or("Virtual input device is unavailable")?;
  let events = match event {
    EventType::KeyPress(key) | EventType::KeyRelease(key) => {
      let code = keymap::to_evdev(key).ok_or("This key has no Linux input code")?;
      let value = i32::from(matches!(event, EventType::KeyPress(_)));
      vec![EvEvent::new(EvType::KEY.0, code.code(), value)]
    }
    EventType::ButtonPress(button) | EventType::ButtonRelease(button) => {
      let code = match button {
        Button::Left => EvKey::BTN_LEFT,
        Button::Right => EvKey::BTN_RIGHT,
        Button::Middle => EvKey::BTN_MIDDLE,
        Button::Unknown(_) => return Err("Unknown button cannot be simulated".into()),
      };
      let value = i32::from(matches!(event, EventType::ButtonPress(_)));
      vec![EvEvent::new(EvType::KEY.0, code.code(), value)]
    }
    EventType::Wheel { delta_x, delta_y } => {
      let dx = i32::try_from(delta_x).map_err(|_| "Wheel delta_x is out of range")?;
      let dy = i32::try_from(delta_y).map_err(|_| "Wheel delta_y is out of range")?;
      vec![
        EvEvent::new(EvType::RELATIVE.0, RelativeAxisCode::REL_HWHEEL.0, dx),
        EvEvent::new(EvType::RELATIVE.0, RelativeAxisCode::REL_WHEEL.0, dy),
      ]
    }
    EventType::MouseMove { x, y } => return move_pointer(device, x, y),
  };
  device
    .emit(&events)
    .map_err(|e| format!("Cannot emit virtual input event: {e}"))
}

pub fn display_size() -> Result<(u64, u64), String> {
  crate::linux_outputs::display_size()
}

#[cfg(test)]
mod tests {
  use super::*;
  use rdev::Key;

  #[test]
  fn key_map_round_trip() {
    for code in keymap::SUPPORTED_KEYS {
      assert_eq!(keymap::to_evdev(keymap::from_evdev(*code)), Some(*code));
    }
    assert!(matches!(
      keymap::from_evdev(EvKey::KEY_F13),
      rdev::Key::Unknown(_)
    ));
  }

  #[test]
  fn evdev_events_cover_public_event_variants() {
    let location = || Ok((10.0, 20.0));
    let cases = [
      (
        EvEvent::new(EvType::KEY.0, EvKey::KEY_A.code(), 1),
        EventType::KeyPress(Key::KeyA),
      ),
      (
        EvEvent::new(EvType::KEY.0, EvKey::KEY_A.code(), 0),
        EventType::KeyRelease(Key::KeyA),
      ),
      (
        EvEvent::new(EvType::KEY.0, EvKey::BTN_LEFT.code(), 1),
        EventType::ButtonPress(Button::Left),
      ),
      (
        EvEvent::new(EvType::KEY.0, EvKey::BTN_LEFT.code(), 0),
        EventType::ButtonRelease(Button::Left),
      ),
      (
        EvEvent::new(EvType::RELATIVE.0, RelativeAxisCode::REL_X.0, 3),
        EventType::MouseMove { x: 10.0, y: 20.0 },
      ),
      (
        EvEvent::new(EvType::RELATIVE.0, RelativeAxisCode::REL_WHEEL.0, -2),
        EventType::Wheel {
          delta_x: 0,
          delta_y: -2,
        },
      ),
    ];
    for (input, expected) in cases {
      let actual = capture_event(input, &location).unwrap().unwrap();
      assert_eq!(actual.event_type, expected);
    }
  }
}
