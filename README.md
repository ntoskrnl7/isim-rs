# isim

[![CI](https://github.com/ntoskrnl7/isim-rs/actions/workflows/test.yml/badge.svg)](https://github.com/ntoskrnl7/isim-rs/actions/workflows/test.yml)
[![npm](https://img.shields.io/npm/v/isim.svg)](https://www.npmjs.com/package/isim)

`isim` is a small Node.js library for controlling X11 windows from JavaScript.
It wraps the common desktop automation operations you usually reach for in
`xdotool`: keyboard input, mouse input, window focus, activation, clicks, and
window lookup.

```ts
import { currentDisplay } from 'isim';

const win = currentDisplay.activeWindow;

win.raise();
await win.focus();
win.key.press('ctrl+l');
await win.mouse.move(20, 10);
win.click(1);
```

## Support

`isim` currently targets Linux desktops running X11.

It depends on `libxdo` and X11 libraries under the hood, so it is not a
Wayland-native automation library. It may still work in XWayland scenarios when
the target window is visible to X11.

## Install

```sh
npm install isim
```

If npm cannot find a prebuilt binary for your platform, or if you are working
from source, install the native Linux dependencies first:

```sh
sudo apt-get update
sudo apt-get install -y pkgconf libxdo-dev libxfixes-dev
```

Source builds also require Node.js and Rust.

## Usage

Use the default display:

```ts
import { currentDisplay } from 'isim';

const active = currentDisplay.activeWindow;
const focused = currentDisplay.focusedWindow;
const underMouse = currentDisplay.windowAtMouse;
```

Use an explicit display:

```ts
import { Display } from 'isim';

const display = new Display(':0');
const win = display.getWindow(0x3400007);

win.raise();
await win.activate();
```

Send keyboard input:

```ts
const win = currentDisplay.activeWindow;

await win.focus();
win.key.press('ctrl+shift+t');
```

Send mouse input:

```ts
const win = currentDisplay.windowAtMouse;

win.mouse.down(1);
win.mouse.up(1);
await win.mouse.move(40, 20);
```

Use the lower-level native bindings when you need direct access:

```ts
import { xdo } from 'isim';

const windowId = xdo.getActiveWindow();
xdo.keyPress('Return', windowId);
```

## API

| API | Description |
| --- | --- |
| `new Display(name?)` | Connect to the default X display or a display such as `:0`. |
| `display.activeWindow` | Window currently active according to the window manager. |
| `display.focusedWindow` | Window that currently owns keyboard focus. |
| `display.windowAtMouse` | Window under the mouse pointer. |
| `display.currentWindow` | Wrapper for libxdo's current-window target. |
| `display.currentScreen` | Wrapper for libxdo's current-screen target. |
| `display.getWindow(id)` | Wrap a specific X11 window ID. |
| `display.getScreen(id)` | Wrap a specific screen ID. |
| `window.key.down(key)` | Press and hold a key sequence. |
| `window.key.up(key)` | Release a key sequence. |
| `window.key.press(key)` | Press and release a key sequence. |
| `window.mouse.down(button)` | Press a mouse button. |
| `window.mouse.up(button)` | Release a mouse button. |
| `window.mouse.move(x, y)` | Move relative to the window. |
| `window.click(button)` | Click a mouse button in the window. |
| `window.focus()` | Focus the window and wait for focus. |
| `window.activate()` | Activate the window and wait for activation. |
| `window.raise()` | Raise the window. |
| `window.close()` | Ask the window to close. |
| `window.kill()` | Kill the window. |
| `window.pid` | Resolve the process ID for the window. |
| `xdo` | Raw native addon exports. |

Most synchronous methods return the underlying `libxdo` status code. Methods
that wait for X11 state changes return `Promise<number>`.

## Development

```sh
npm ci
npm run test:node
npm run test:rust
npm run debug
```

Useful scripts:

| Command | Description |
| --- | --- |
| `npm run test:node` | Type-check TypeScript and run JavaScript wrapper tests. |
| `npm run test:rust` | Run the Rust test suite. |
| `npm test` | Run both Node and Rust tests. |
| `npm run debug` | Build a debug native addon. |
| `npm run build` | Build a release native addon. |
| `npm run dryrun` | Start a dry-run release workflow. |
| `npm run release` | Start the release workflow. |
