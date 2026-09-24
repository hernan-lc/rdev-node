// Child-process probe for the listener integration test in complete.test.cjs.
// Starts the listener, simulates one key press, and exits 0 once that event
// is observed. Prints SKIP and exits 0 when no display is available, exits
// non-zero on failure. Every path ends in process.exit via a hard timeout,
// so this probe can never hang the suite.
const rdev = require('../index.js')

const HARD_TIMEOUT_MS = 15000
const hardTimeout = setTimeout(() => {
  console.error('TIMEOUT: no event received')
  process.exit(1)
}, HARD_TIMEOUT_MS)

function done(code, message) {
  if (message) {
    console.log(message)
  }
  clearTimeout(hardTimeout)
  try {
    rdev.stopListener()
  } catch {
    // ignore teardown errors; the exit code carries the result
  }
  process.exit(code)
}

try {
  rdev.initSimulation()
} catch (error) {
  done(0, `SKIP: ${error.message}`)
}

// rdev targets X11; under XWayland, XTEST-injected events are not reliably
// visible to XRECORD (verified with xdotool), so loopback cannot be tested.
if (process.env.XDG_SESSION_TYPE === 'wayland') {
  done(0, 'SKIP: Wayland session (XWayland loopback is unreliable)')
}

const targetKey = rdev.KeyCode.KeyB

try {
  rdev.startListener(
    (event) => {
      if (event.eventType === rdev.EventTypeValue.KeyPress && event.keyPress && event.keyPress.key === targetKey) {
        done(0, 'RECEIVED')
      }
    },
    (message) => {
      done(1, `ERROR: ${message}`)
    },
  )
} catch (error) {
  done(1, `ERROR: startListener threw: ${error.message}`)
}

// Reset the target key first: an aborted run may have left it logically
// pressed, which can suppress fresh press events on some X servers.
try {
  rdev.simulateEvent({
    eventType: rdev.EventTypeValue.KeyRelease,
    keyRelease: { key: targetKey },
    time: Date.now(),
  })
} catch {
  // initSimulation already proved the display is reachable; ignore reset errors
}

// Give the OS hook a moment to install, then simulate press/release pairs
// until the press is observed. Repeating is deliberate: the first injected
// event can race hook installation and be lost.
const simulator = setInterval(() => {
  try {
    rdev.simulateEvent({
      eventType: rdev.EventTypeValue.KeyPress,
      keyPress: { key: targetKey },
      time: Date.now(),
    })
    rdev.simulateEvent({
      eventType: rdev.EventTypeValue.KeyRelease,
      keyRelease: { key: targetKey },
      time: Date.now(),
    })
  } catch (error) {
    clearInterval(simulator)
    done(1, `ERROR: simulateEvent threw: ${error.message}`)
  }
}, 500)
