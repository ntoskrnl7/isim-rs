// This module is the CJS entry point for the library.

// The Rust addon.
import * as addon from './load.cjs';

// Use this declaration to assign types to the addon's exports,
// which otherwise by default are `any`.

export class Window {
  readonly display: Display;
  readonly id?: number;

  constructor(id?: number, display?: Display) {
    this.display = display ? display : new Display();
    this.id = id;
  }

  get key() {
    return {
      down: (keySeq: string, delay?: number) => {
        return addon.keyDown(keySeq, this.id, this.display.name, delay ? delay : 0);
      },
      up: (keySeq: string, delay?: number) => {
        return addon.keyUp(keySeq, this.id, this.display.name, delay ? delay : 0);
      },
      press: (keySeq: string, delay?: number) => {
        return addon.keyPress(keySeq, this.id, this.display.name, delay ? delay : 0);
      },
    };
  }

  get mouse() {
    return {
      down: (button: number) => {
        return addon.mouseDown(button, this.id, this.display.name);
      },
      up: (button: number) => {
        return addon.mouseUp(button, this.id, this.display.name);
      },
      move: async (x: number, y: number) => {
        return await addon.mouseMoveRelativeToWindow(x, y, this.id, this.display.name);
      }
    }
  }

  get pid() {
    if (this.id) {
      return addon.getPIDWindow(this.id, this.display.name);
    }
  }

  click(button: number) {
    return addon.clickWindow(button, this.id, this.display.name);
  }

  reparent(parent: Window) {
    if (this.id && parent.id) {
      addon.reparentWindow(this.id, parent.id, this.display.name);
    }
  }

  raise() {
    if (this.id) {
      return addon.raiseWindow(this.id, this.display.name);
    }
  }

  async focus() {
    if (this.id) {
      return await addon.focusWindow(this.id, this.display.name);
    }
  }

  async activate() {
    if (this.id) {
      return await addon.activateWindow(this.id, this.display.name);
    }
  }

  kill() {
    if (this.id) {
      return addon.killWindow(this.id, this.display.name);
    }
  }

  close() {
    if (this.id) {
      return addon.closeWindow(this.id, this.display.name);
    }
  }
};

export class Screen {
  readonly display: Display;
  readonly id?: number;

  constructor(id?: number, display?: Display) {
    this.display = display ? display : new Display();
    this.id = id;
  }

  get mouse() {
    return {
      move: (x: number, y: number) => {
        return addon.mouseMove(x, y, this.id, this.display.name);
      }
    }
  }
};

export class Display {
  public readonly name?: string;
  constructor(name?: string) {
    this.name = name;
  }

  get windowAtMouse() {
    return new Window(addon.getWindowAtMouse(this.name), this);
  }

  get currentScreen() {
    return new Screen();
  }

  get focusedWindow() {
    return new Window(addon.getFocusedWindow(this.name), this);
  }

  get activeWindow() {
    return new Window(addon.getActiveWindow(this.name), this);
  }

  get currentWindow() {
    return new Window();
  }

  getWindow(id: number): Window {
    return new Window(id, this);
  }

  getScreen(id: number): Screen {
    return new Screen(id);
  }

  get mouse() {
    return {
      move: (x: number, y: number) => {
        return addon.mouseMoveRelative(x, y, this.name);
      }
    }
  }
};

export const currentDisplay = new Display();

declare module "./load.cjs" {
  function keyDown(keySeq: string, window?: number, display?: string, delay?: number): number;
  function keyUp(keySeq: string, window?: number, display?: string, delay?: number): number;
  function keyPress(keySeq: string, window?: number, display?: string, delay?: number): number;

  function mouseDown(button: number, window?: number, display?: string): number;
  function mouseUp(button: number, window?: number, display?: string): number;
  function mouseMoveRelativeToWindow(x: number, y: number, window?: number, display?: string): Promise<number>;
  function mouseMoveRelative(x: number, y: number, display?: string): Promise<number>;
  function mouseMove(x: number, y: number, screen?: number, display?: string): Promise<number>;

  function clickWindow(button: number, window?: number, display?: string): number;
  function focusWindow(window: number, display?: string): Promise<number>;
  function activateWindow(window: number, display?: string): Promise<number>;
  function raiseWindow(window: number, display?: string): Promise<number>;
  function closeWindow(window: number, display?: string): Promise<number>;
  function killWindow(window: number, display?: string): Promise<number>;
  function getPIDWindow(window: number, display?: string): number;
  function reparentWindow(window: number, parent: number, display?: string): number;

  function getFocusedWindow(display?: string): number;
  function getActiveWindow(display?: string): number;

  function getWindowAtMouse(display?: string): number;
}

export const xdo = {
  ...addon
};