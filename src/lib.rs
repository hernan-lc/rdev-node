#![deny(clippy::all)]

pub mod conversions;
pub mod enums;
pub mod events;
#[cfg(target_os = "linux")]
mod linux_wayland;

use napi::bindgen_prelude::Function;
use napi::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode};
use napi::{Result, Status};
use napi_derive::napi;
use once_cell::sync::Lazy;
use rdev::listen;
use std::sync::{Condvar, Mutex};

// Re-export the main types for easier access
pub use enums::{ButtonType, EventTypeValue, KeyCode, NormalizedModifier};
pub use events::{
  ButtonPressEvent, ButtonReleaseEvent, DisplaySize, InputEvent, KeyPressEvent, KeyReleaseEvent,
  MouseMoveEvent, WheelEvent,
};

/// Normalizes a KeyCode to its modifier name if it's a modifier key
#[napi]
pub fn normalize_key_name(key_code: KeyCode) -> String {
  conversions::normalize_key_name(&key_code)
}

/// Checks if a KeyCode is a modifier key
#[napi]
pub fn is_modifier_key(key_code: KeyCode) -> bool {
  conversions::is_modifier_key(&key_code)
}

/// Converts a string key representation to its KeyCode
#[napi]
pub fn string_key_to_keycode(key: String) -> Option<KeyCode> {
  conversions::string_key_to_keycode(&key)
}

/// Checks that input simulation is available in the current environment.
///
/// On X11, `rdev` checks display access. On Wayland, this creates one reusable
/// virtual input device through `/dev/uinput`, which requires write access.
#[napi]
pub fn init_simulation() -> Result<()> {
  #[cfg(target_os = "linux")]
  if linux_wayland::is_wayland() {
    return linux_wayland::init_simulation()
      .map_err(|e| napi::Error::from_reason(format!("Input simulation is not available: {e}")));
  }
  rdev::display_size()
    .map(|_| ())
    .map_err(|e| napi::Error::from_reason(format!("Input simulation is not available: {e:?}")))
}

/// Get the size of the main display
#[napi]
pub fn get_display_size() -> Result<DisplaySize> {
  #[cfg(target_os = "linux")]
  if linux_wayland::is_wayland() {
    let (width, height) = linux_wayland::display_size()
      .map_err(|e| napi::Error::from_reason(format!("Failed to get display size: {e}")))?;
    return Ok(DisplaySize {
      width: width as f64,
      height: height as f64,
    });
  }
  let (width, height) = rdev::display_size()
    .map_err(|e| napi::Error::from_reason(format!("Failed to get display size: {e:?}")))?;
  Ok(DisplaySize {
    width: width as f64,
    height: height as f64,
  })
}

/// Threadsafe bridge delivering input events to JavaScript.
/// Built with default (strong) references so an active listener keeps the
/// Node.js event loop alive until it is stopped or fails.
type EventTsfn = ThreadsafeFunction<InputEvent, (), InputEvent, Status, false>;
/// Threadsafe bridge delivering listener failures to JavaScript.
type ErrorTsfn = ThreadsafeFunction<String, (), String, Status, false>;

struct ListenerState {
  generation: u64,
  events: EventTsfn,
  errors: Option<ErrorTsfn>,
}

#[derive(Default)]
struct ListenerSlot {
  next_generation: u64,
  native_running: bool,
  active: Option<ListenerState>,
}

static LISTENER: Lazy<Mutex<ListenerSlot>> = Lazy::new(|| Mutex::new(ListenerSlot::default()));
static LISTENER_DONE: Condvar = Condvar::new();

fn spawn_native_thread(
  slot: &mut ListenerSlot,
  spawn: impl FnOnce() -> std::io::Result<()>,
) -> Result<()> {
  spawn().map_err(|e| napi::Error::from_reason(format!("Failed to spawn listener thread: {e}")))?;
  slot.native_running = true;
  Ok(())
}

// Abort pending event delivery when a listener is detached. napi-rs otherwise
// drains already queued events after stopListener() has returned.
#[allow(deprecated)]
fn release_events(events: EventTsfn) {
  if let Err(error) = events.abort() {
    eprintln!("Failed to abort listener callbacks: {error}");
  }
}

#[allow(deprecated)]
fn release_errors(errors: ErrorTsfn) {
  if let Err(error) = errors.abort() {
    eprintln!("Failed to abort listener error callbacks: {error}");
  }
}

/// Start listening for input events.
///
/// The callback's return value is ignored. Only one listener may run at a
/// time; calling this while a listener is active returns an error.
///
/// X11 uses `rdev::listen`. Linux Wayland reads physical input devices through
/// evdev and requires persistent read access to `/dev/input/event*`. Native
/// failures are reported through `on_error` when provided; otherwise they are
/// written to stderr.
///
/// The listener holds the Node.js event loop alive until `stopListener()` is
/// called or the listen loop fails. The X11 `rdev` hook cannot be unhooked and
/// cannot restart after stop. The Wayland evdev loop exits after stop and may
/// be started again.
#[napi]
pub fn start_listener(
  callback: Function<InputEvent, ()>,
  on_error: Option<Function<String, ()>>,
) -> Result<()> {
  #[cfg(target_os = "linux")]
  let wayland = linux_wayland::is_wayland();
  #[cfg(not(target_os = "linux"))]
  let wayland = false;

  #[cfg(target_os = "linux")]
  if wayland {
    linux_wayland::preflight_capture()
      .map_err(|e| napi::Error::from_reason(format!("Input listener is not available: {e}")))?;
  }

  let mut slot = LISTENER.lock().unwrap_or_else(|e| e.into_inner());
  if slot.active.is_some() {
    return Err(napi::Error::from_reason("Listener is already running."));
  }
  if wayland && slot.native_running {
    let (updated, _) = LISTENER_DONE
      .wait_timeout_while(slot, std::time::Duration::from_secs(2), |state| {
        state.native_running
      })
      .unwrap_or_else(|e| e.into_inner());
    slot = updated;
  }
  if slot.native_running {
    return Err(napi::Error::from_reason(
      "The native listener is still running and cannot be restarted yet.",
    ));
  }

  let events: EventTsfn = callback.build_threadsafe_function().build()?;
  let errors: Option<ErrorTsfn> = on_error
    .map(|f| f.build_threadsafe_function().build())
    .transpose()?;

  let generation = slot.next_generation;
  slot.next_generation = slot.next_generation.wrapping_add(1);
  // Keep the lock until spawn succeeds. The new thread waits on this lock,
  // so it cannot report failure before its state is published.
  spawn_native_thread(&mut slot, || {
    std::thread::Builder::new()
      .name("rdev-node-listener".into())
      .spawn(move || {
        let deliver = move |event: rdev::Event| {
          let event: InputEvent = event.into();
          let slot = LISTENER.lock().unwrap_or_else(|e| e.into_inner());
          let current = slot.active.as_ref().filter(|s| s.generation == generation);
          if let Some(state) = current {
            state
              .events
              .call(event, ThreadsafeFunctionCallMode::NonBlocking);
          }
        };
        #[cfg(target_os = "linux")]
        let outcome = if wayland {
          linux_wayland::listen(deliver, || {
            !LISTENER
              .lock()
              .unwrap_or_else(|e| e.into_inner())
              .active
              .as_ref()
              .is_some_and(|s| s.generation == generation)
          })
        } else {
          listen(deliver).map_err(|e| format!("{e:?}"))
        };
        #[cfg(not(target_os = "linux"))]
        let outcome = listen(deliver).map_err(|e| format!("{e:?}"));
        let mut slot = LISTENER.lock().unwrap_or_else(|e| e.into_inner());
        let ours = slot
          .active
          .as_ref()
          .is_some_and(|s| s.generation == generation);
        if let Err(error) = outcome {
          let message = format!("Input listener failed: {error}");
          if ours {
            if let Some(state) = slot.active.as_ref() {
              if let Some(errors) = state.errors.as_ref() {
                errors.call(message, ThreadsafeFunctionCallMode::NonBlocking);
              } else {
                eprintln!("{message}");
              }
            }
            if let Some(state) = slot.active.take() {
              release_events(state.events);
            }
          } else {
            eprintln!("{message}");
          }
        }
        slot.native_running = false;
        LISTENER_DONE.notify_all();
      })
      .map(|_| ())
  })?;

  slot.active = Some(ListenerState {
    generation,
    events,
    errors,
  });

  Ok(())
}

/// Stop the active input event listener, if any.
///
/// Returns `true` when a listener was running and is now stopped. After
/// stopping, the event loop is no longer held alive. Wayland can restart;
/// the X11 `rdev` hook cannot be restarted while it remains blocked.
#[napi]
pub fn stop_listener() -> bool {
  let state = LISTENER
    .lock()
    .unwrap_or_else(|e| e.into_inner())
    .active
    .take();
  if let Some(state) = state {
    release_events(state.events);
    if let Some(errors) = state.errors {
      release_errors(errors);
    }
    true
  } else {
    false
  }
}

/// Simulate an input event
#[napi]
pub fn simulate_event(event: InputEvent) -> Result<()> {
  let rdev_event: rdev::Event = event
    .try_into()
    .map_err(|e| napi::Error::from_reason(format!("Invalid event data: {e}")))?;
  #[cfg(target_os = "linux")]
  if linux_wayland::is_wayland() {
    return linux_wayland::simulate(rdev_event.event_type)
      .map_err(|e| napi::Error::from_reason(format!("Failed to simulate event: {e}")));
  }
  rdev::simulate(&rdev_event.event_type)
    .map_err(|e| napi::Error::from_reason(format!("Failed to simulate event: {e:?}")))?;
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn failed_spawn_does_not_mark_native_listener_running() {
    let mut slot = ListenerSlot::default();
    let failure = spawn_native_thread(&mut slot, || Err(std::io::Error::other("injected failure")));
    assert!(failure.is_err());
    assert!(!slot.native_running);
    assert!(slot.active.is_none());
  }
}
