#![deny(clippy::all)]

pub mod conversions;
pub mod enums;
pub mod events;

use napi::bindgen_prelude::Function;
use napi::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode};
use napi::{Result, Status};
use napi_derive::napi;
use once_cell::sync::Lazy;
use rdev::listen;
use std::sync::Mutex;

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
/// `rdev` opens and closes its own display connection on every call, so no
/// state is retained here; this is an explicit availability check. It succeeds
/// when a display can be reached and fails otherwise (for example with a
/// `NoDisplay` error on headless Linux without an X server).
#[napi]
pub fn init_simulation() -> Result<()> {
  rdev::display_size()
    .map(|_| ())
    .map_err(|e| napi::Error::from_reason(format!("Input simulation is not available: {e:?}")))
}

/// Get the size of the main display
#[napi]
pub fn get_display_size() -> Result<DisplaySize> {
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
/// Failures of the underlying `rdev::listen` loop (for example no X display on
/// Linux) are reported through `on_error` when provided, and the listener is
/// released. Without `on_error` they are written to stderr.
///
/// The listener holds the Node.js event loop alive until `stopListener()` is
/// called or the listen loop fails. Note: `rdev` 0.5.3 offers no way to unhook
/// the OS listener. After stopping, the native thread remains blocked until
/// process exit, and another listener cannot start while that hook is alive.
#[napi]
pub fn start_listener(
  callback: Function<InputEvent, ()>,
  on_error: Option<Function<String, ()>>,
) -> Result<()> {
  let mut slot = LISTENER.lock().unwrap_or_else(|e| e.into_inner());
  if slot.active.is_some() {
    return Err(napi::Error::from_reason("Listener is already running."));
  }
  if slot.native_running {
    return Err(napi::Error::from_reason(
      "The native listener is still running; rdev cannot restart it after stopListener().",
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
        if let Err(error) = listen(move |event| {
          let event: InputEvent = event.into();
          let slot = LISTENER.lock().unwrap_or_else(|e| e.into_inner());
          let current = slot.active.as_ref().filter(|s| s.generation == generation);
          if let Some(state) = current {
            state
              .events
              .call(event, ThreadsafeFunctionCallMode::NonBlocking);
          }
        }) {
          let message = format!("Input listener failed: {error:?}");
          let mut slot = LISTENER.lock().unwrap_or_else(|e| e.into_inner());
          if slot.native_running {
            let ours = slot
              .active
              .as_ref()
              .is_some_and(|s| s.generation == generation);
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
            slot.native_running = false;
          }
        }
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
/// stopping, the event loop is no longer held alive. The native hook cannot
/// be restarted while it remains blocked inside `rdev::listen`.
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
