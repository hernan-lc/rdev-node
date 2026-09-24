//! Logical desktop bounds from the compositor's xdg-output protocol.

use std::collections::HashMap;
use wayland_client::{
  protocol::{wl_output, wl_registry},
  Connection, Dispatch, QueueHandle,
};
use wayland_protocols::xdg::xdg_output::zv1::client::{zxdg_output_manager_v1, zxdg_output_v1};

#[derive(Default)]
struct OutputInfo {
  position: Option<(i32, i32)>,
  size: Option<(i32, i32)>,
}

#[derive(Default)]
struct OutputState {
  manager: Option<zxdg_output_manager_v1::ZxdgOutputManagerV1>,
  outputs: Vec<(u32, wl_output::WlOutput)>,
  logical: HashMap<u32, OutputInfo>,
}

impl Dispatch<wl_registry::WlRegistry, ()> for OutputState {
  fn event(
    state: &mut Self,
    registry: &wl_registry::WlRegistry,
    event: wl_registry::Event,
    _: &(),
    _: &Connection,
    qh: &QueueHandle<Self>,
  ) {
    if let wl_registry::Event::Global {
      name,
      interface,
      version,
    } = event
    {
      match interface.as_str() {
        "zxdg_output_manager_v1" => {
          state.manager = Some(registry.bind(name, version.min(3), qh, ()));
        }
        "wl_output" => {
          let output = registry.bind(name, version.min(4), qh, ());
          state.outputs.push((name, output));
        }
        _ => {}
      }
    }
  }
}

impl Dispatch<wl_output::WlOutput, ()> for OutputState {
  fn event(
    _: &mut Self,
    _: &wl_output::WlOutput,
    _: wl_output::Event,
    _: &(),
    _: &Connection,
    _: &QueueHandle<Self>,
  ) {
  }
}

impl Dispatch<zxdg_output_manager_v1::ZxdgOutputManagerV1, ()> for OutputState {
  fn event(
    _: &mut Self,
    _: &zxdg_output_manager_v1::ZxdgOutputManagerV1,
    _: zxdg_output_manager_v1::Event,
    _: &(),
    _: &Connection,
    _: &QueueHandle<Self>,
  ) {
  }
}

impl Dispatch<zxdg_output_v1::ZxdgOutputV1, u32> for OutputState {
  fn event(
    state: &mut Self,
    _: &zxdg_output_v1::ZxdgOutputV1,
    event: zxdg_output_v1::Event,
    name: &u32,
    _: &Connection,
    _: &QueueHandle<Self>,
  ) {
    let info = state.logical.entry(*name).or_default();
    match event {
      zxdg_output_v1::Event::LogicalPosition { x, y } => info.position = Some((x, y)),
      zxdg_output_v1::Event::LogicalSize { width, height } => {
        info.size = Some((width, height));
      }
      _ => {}
    }
  }
}

fn desktop_size(outputs: &HashMap<u32, OutputInfo>) -> Result<(u64, u64), String> {
  let mut bounds: Option<(i64, i64, i64, i64)> = None;
  for info in outputs.values() {
    let (Some((x, y)), Some((width, height))) = (info.position, info.size) else {
      continue;
    };
    if width <= 0 || height <= 0 {
      continue;
    }
    let (x, y, right, bottom) = (
      i64::from(x),
      i64::from(y),
      i64::from(x) + i64::from(width),
      i64::from(y) + i64::from(height),
    );
    bounds = Some(match bounds {
      Some((left, top, old_right, old_bottom)) => (
        left.min(x),
        top.min(y),
        old_right.max(right),
        old_bottom.max(bottom),
      ),
      None => (x, y, right, bottom),
    });
  }
  let (left, top, right, bottom) = bounds.ok_or("No logical Wayland outputs were reported")?;
  Ok(((right - left) as u64, (bottom - top) as u64))
}

pub fn display_size() -> Result<(u64, u64), String> {
  let conn = Connection::connect_to_env().map_err(|e| format!("Cannot connect to Wayland: {e}"))?;
  let mut queue = conn.new_event_queue::<OutputState>();
  let qh = queue.handle();
  conn.display().get_registry(&qh, ());
  let mut state = OutputState::default();
  queue
    .roundtrip(&mut state)
    .map_err(|e| format!("Cannot read Wayland outputs: {e}"))?;
  let manager = state
    .manager
    .as_ref()
    .ok_or("The compositor does not advertise xdg-output")?;
  for (name, output) in &state.outputs {
    manager.get_xdg_output(output, &qh, *name);
  }
  queue
    .roundtrip(&mut state)
    .map_err(|e| format!("Cannot read Wayland output geometry: {e}"))?;
  desktop_size(&state.logical)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn logical_desktop_spans_all_outputs() {
    let mut outputs = HashMap::new();
    outputs.insert(
      1,
      OutputInfo {
        position: Some((-1920, 0)),
        size: Some((1920, 1080)),
      },
    );
    outputs.insert(
      2,
      OutputInfo {
        position: Some((0, 0)),
        size: Some((2560, 1440)),
      },
    );
    assert_eq!(desktop_size(&outputs).unwrap(), (4480, 1440));
  }
}
