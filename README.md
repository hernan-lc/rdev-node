# rdev-node

A high-performance Node.js native addon for listening to system-wide keyboard and mouse events. Built with Rust.

## Install

```bash
npm install rdev-node
```

## Quick Start

```javascript
const { startListener, stopListener } = require('rdev-node');

startListener((event) => {
  console.log(event);
});

// Later: detach the listener so the process can exit.
stopListener();
```

ESM is also supported: `import { startListener, stopListener } from 'rdev-node'`.

## Requirements

- Node.js >= 20.3.0 (N-API 9)
- **Rust** (required to build from source) - Install from [rustup.rs](https://rustup.rs/)
- Linux (X11): build tools plus `libx11-dev libxtst-dev libxi-dev libxext-dev libxfixes-dev libxrender-dev pkg-config`
  (Alpine/musl: `build-base musl-dev libx11-dev libxtst-dev libxi-dev libxext-dev libxfixes-dev libxrender-dev pkgconfig`).
  At runtime the matching shared libraries must be present
  (`libx11-6 libxtst6 libxi6 libxext6 libxfixes3 libxrender1` on Debian/Ubuntu,
  `libx11 libxtst libxi libxext libxfixes libxrender` on Alpine).

## Platform Notes

- **Linux**: X11 only; Wayland is not supported by `rdev`. A running X server is required — use `Xvfb` (`xvfb-run`) in headless environments such as CI. Under XWayland, capture/simulation may be limited or unreliable (XTEST-injected events are not reliably visible to XRECORD), so the loopback integration test is skipped there.
- **macOS**: the app/terminal running Node needs **Accessibility** permission (System Settings → Privacy & Security → Accessibility) to receive key events, and **Input Monitoring** approval where prompted.

## Build

```bash
npm install
npm run build
```

## API

### startListener(callback, onError?)

Listen to keyboard and mouse events. The callback's return value is ignored. Only one native listener can run in a process. A duplicate start throws.

```javascript
startListener(
  (event) => {
    console.log('Type:', event.eventType);
    if (event.keyPress) console.log('Key:', event.keyPress.key);
    if (event.mouseMove) console.log('Position:', event.mouseMove.x, event.mouseMove.y);
  },
  (message) => {
    console.error('Listener failed:', message);
  },
);
```

### stopListener()

Stop the active listener, if any. Returns `true` when a listener was running and is now stopped. Stopping releases the JavaScript callbacks and lets Node exit. `rdev` offers no way to unhook its blocking OS listener, so a new `startListener()` call throws while that native hook remains alive. If the native listener exits with an error, its callbacks are released and a later start may retry. The active listener keeps the Node event loop alive; stopping or a native error releases that hold.

### initSimulation()

Check that input simulation is available in the current environment (for example, that an X display can be reached on Linux). `rdev` manages its own display connections, so no state is retained — this is an explicit availability check. Throws when unavailable.

### simulateEvent(event)

Simulate keyboard/mouse events. Each event requires its matching payload and a finite, non-negative integer timestamp below `2^53` milliseconds. Unknown captured keys/buttons cannot be simulated because their native platform codes are unavailable.

```javascript
simulateEvent({
  eventType: 'KeyPress',
  keyPress: { key: 'KeyA' },
  time: Date.now()
});
```

### getDisplaySize()

Get main display dimensions.

```javascript
const { width, height } = getDisplaySize();
```

## Supported Platforms

- macOS (x64, arm64)
- Windows (x64, ia32; arm64 is build-only, not runtime-tested in CI)
- Linux (x64 gnu + musl, arm64, armv7)

Android is not supported (`rdev` has no Android backend).

## License

MIT
