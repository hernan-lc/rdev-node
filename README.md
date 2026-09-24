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

- **Linux X11**: Uses `rdev` and requires a running X server. Use `Xvfb` (`xvfb-run`) in headless environments.
- **Linux Wayland**: Captures physical keyboard and pointer events through `/dev/input/event*` and injects keys, buttons, and wheel events through `/dev/uinput`. This works below the compositor and requires one-time device permissions; see below. Display size comes from the compositor's xdg-output protocol, including in sessions without XWayland. Absolute pointer coordinates still use XWayland, so a session without XWayland cannot start the full listener or simulate `MouseMove`. `MouseMove` simulation first uses relative virtual-device motion, then falls back to an XWayland pointer warp if necessary; whether that warp affects native Wayland surfaces depends on the compositor. The optional captured window name is unavailable on this backend.
- **macOS**: the app/terminal running Node needs **Accessibility** permission (System Settings → Privacy & Security → Accessibility) to receive key events, and **Input Monitoring** approval where prompted.

### One-time Wayland device access

On Linux, check `id -nG`, `ls -l /dev/input/event*`, and `ls -l /dev/uinput`. If the event devices are owned by the `input` group, add your user once and log out and back in:

```bash
sudo usermod -aG input "$USER"
```

If `/dev/uinput` is not writable by your user after logging back in, install an explicit udev rule:

```bash
printf 'KERNEL=="uinput", GROUP="input", MODE="0660"\n' | sudo tee /etc/udev/rules.d/70-rdev-node-uinput.rules
sudo udevadm control --reload-rules
sudo udevadm trigger --name-match=/dev/uinput
```

Verify access with `test -r /dev/input/event0` and `test -w /dev/uinput`, using a real event device path on your system. Group membership and udev rules persist across runs, so there is no repeated portal approval prompt or restore token. Membership in `input` grants access to all physical keystrokes and pointer input for that account; grant it only to trusted users. The package never changes system permissions automatically.

## Build

```bash
npm install
npm run build
```

## API

### startListener(callback, onError?)

Listen to keyboard and mouse events. The callback's return value is ignored. Only one active listener can run in a process. A duplicate start throws. On Wayland, missing device access produces a clear error at start.

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

Stop the active listener, if any. Returns `true` when a listener was running and is now stopped. Stopping releases the JavaScript callbacks and lets Node exit. On Wayland, the device reader exits and `startListener()` can run again. On X11, `rdev` offers no way to unhook its blocking OS listener, so a new start throws while that hook remains alive. If the native listener exits with an error, its callbacks are released and a later start may retry. The active listener keeps the Node event loop alive; stopping or a native error releases that hold.

### initSimulation()

Check that input simulation is available. X11 checks display access. Wayland creates a reusable `/dev/uinput` virtual device and throws if device access is unavailable.

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
