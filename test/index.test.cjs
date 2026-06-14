const assert = require('node:assert/strict');
const Module = require('node:module');
const path = require('node:path');
const test = require('node:test');

const indexPath = path.resolve(__dirname, '../lib/index.cjs');
const loadPath = path.resolve(__dirname, '../lib/load.cjs');

function createMockAddon() {
  const calls = [];

  const record = (name, value) => (...args) => {
    calls.push({ name, args });
    return value;
  };

  const recordAsync = (name, value) => async (...args) => {
    calls.push({ name, args });
    return value;
  };

  return {
    addon: {
      keyDown: record('keyDown', 10),
      keyUp: record('keyUp', 11),
      keyPress: record('keyPress', 12),
      mouseDown: record('mouseDown', 20),
      mouseUp: record('mouseUp', 21),
      mouseMoveRelativeToWindow: recordAsync('mouseMoveRelativeToWindow', 30),
      mouseMoveRelative: recordAsync('mouseMoveRelative', 31),
      mouseMove: recordAsync('mouseMove', 32),
      hideCursor: record('hideCursor', undefined),
      showCursor: record('showCursor', undefined),
      clickWindow: record('clickWindow', 40),
      focusWindow: recordAsync('focusWindow', 50),
      activateWindow: recordAsync('activateWindow', 51),
      raiseWindow: record('raiseWindow', 52),
      closeWindow: record('closeWindow', 53),
      killWindow: record('killWindow', 54),
      getPIDWindow: record('getPIDWindow', 4242),
      reparentWindow: record('reparentWindow', 60),
      getFocusedWindow: record('getFocusedWindow', 101),
      getActiveWindow: record('getActiveWindow', 102),
      getRootWindow: record('getRootWindow', 103),
      getWindowAtMouse: record('getWindowAtMouse', 104),
    },
    calls,
  };
}

function loadIndexWith(addon) {
  delete require.cache[indexPath];
  delete require.cache[loadPath];

  const originalLoad = Module._load;
  Module._load = function mockedLoad(request, parent, isMain) {
    if (request === './load.cjs' && parent?.filename === indexPath) {
      return addon;
    }

    return originalLoad.call(this, request, parent, isMain);
  };

  try {
    return require(indexPath);
  } finally {
    Module._load = originalLoad;
  }
}

test('Display creates windows and screens bound to the same display', async () => {
  const { addon, calls } = createMockAddon();
  const { Display, Window, Screen } = loadIndexWith(addon);

  const display = new Display(':99');
  const window = display.getWindow(123);
  const screen = display.getScreen(2);

  assert.ok(window instanceof Window);
  assert.ok(screen instanceof Screen);
  assert.equal(window.display, display);
  assert.equal(screen.display, display);

  assert.equal(window.key.press('ctrl+l', 25), 12);
  assert.equal(await screen.mouse.move(10, 20), 32);

  assert.deepEqual(calls, [
    { name: 'keyPress', args: ['ctrl+l', 123, ':99', 25] },
    { name: 'mouseMove', args: [10, 20, 2, ':99'] },
  ]);
});

test('current display accessors preserve the display name', () => {
  const { addon, calls } = createMockAddon();
  const { Display } = loadIndexWith(addon);

  const display = new Display(':7');

  assert.equal(display.focusedWindow.id, 101);
  assert.equal(display.activeWindow.id, 102);
  assert.equal(display.windowAtMouse.id, 104);
  assert.equal(display.currentWindow.click(1), 40);

  assert.deepEqual(calls, [
    { name: 'getFocusedWindow', args: [':7'] },
    { name: 'getActiveWindow', args: [':7'] },
    { name: 'getWindowAtMouse', args: [':7'] },
    { name: 'clickWindow', args: [1, undefined, ':7'] },
  ]);
});

test('Window methods forward IDs, display names, and defaults', async () => {
  const { addon, calls } = createMockAddon();
  const { Display } = loadIndexWith(addon);

  const window = new Display(':1').getWindow(9001);
  const parent = new Display(':1').getWindow(77);

  assert.equal(window.key.down('shift'), 10);
  assert.equal(window.mouse.down(1), 20);
  assert.equal(window.pid, 4242);
  assert.equal(window.raise(), 52);
  assert.equal(window.close(), 53);
  assert.equal(window.kill(), 54);
  window.reparent(parent);
  assert.equal(await window.focus(), 50);
  assert.equal(await window.activate(), 51);
  assert.equal(await window.mouse.move(3, 4), 30);

  assert.deepEqual(calls, [
    { name: 'keyDown', args: ['shift', 9001, ':1', 0] },
    { name: 'mouseDown', args: [1, 9001, ':1'] },
    { name: 'getPIDWindow', args: [9001, ':1'] },
    { name: 'raiseWindow', args: [9001, ':1'] },
    { name: 'closeWindow', args: [9001, ':1'] },
    { name: 'killWindow', args: [9001, ':1'] },
    { name: 'reparentWindow', args: [9001, 77, ':1'] },
    { name: 'focusWindow', args: [9001, ':1'] },
    { name: 'activateWindow', args: [9001, ':1'] },
    { name: 'mouseMoveRelativeToWindow', args: [3, 4, 9001, ':1'] },
  ]);
});

test('xdo exposes the native addon surface', () => {
  const { addon } = createMockAddon();
  const { xdo } = loadIndexWith(addon);

  assert.equal(xdo.keyPress('a'), 12);
  assert.equal(xdo.showCursor(), undefined);
});
