use libxdo_sys::{
    xdo_activate_window, xdo_click_window, xdo_focus_window, xdo_free, xdo_get_active_window,
    xdo_get_current_desktop, xdo_get_desktop_for_window, xdo_get_focused_window,
    xdo_get_mouse_location, xdo_get_pid_window, xdo_get_window_at_mouse, xdo_get_window_name,
    xdo_kill_window, xdo_mouse_down, xdo_mouse_up, xdo_move_mouse, xdo_move_mouse_relative,
    xdo_move_mouse_relative_to_window, xdo_new, xdo_reparent_window, xdo_send_keysequence_window,
    xdo_send_keysequence_window_down, xdo_send_keysequence_window_up, xdo_wait_for_mouse_move_from,
    xdo_wait_for_window_active, xdo_wait_for_window_focus,
};
use neon::prelude::*;
use std::{ffi::CString, sync::Arc};

struct Xdo(*mut libxdo_sys::xdo_t);

unsafe impl Send for Xdo {}
unsafe impl Sync for Xdo {}

impl Drop for Xdo {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe {
                xdo_free(self.0);
            }
        }
    }
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    macro_rules! display_to_xdo {
        ($display:expr, $cx:expr) => {
            Xdo(unsafe {
                match $display {
                    Some(display) => {
                        if display.is_a::<JsUndefined, _>(&mut $cx)
                            || display.is_a::<JsNull, _>(&mut $cx)
                        {
                            xdo_new(std::ptr::null())
                        } else {
                            let display = display
                                .downcast_or_throw::<JsString, _>(&mut $cx)?
                                .value(&mut $cx);
                            let display = match CString::new(display) {
                                Ok(display) => display,
                                Err(err) => {
                                    return $cx.throw_error(err.to_string());
                                }
                            };
                            let xdo = xdo_new(display.as_ptr() as _);
                            if xdo.is_null() {
                                return $cx
                                    .throw_error(format!("Can't open display: {:?}", display));
                            }
                            xdo
                        }
                    }
                    None => xdo_new(std::ptr::null()),
                }
            })
        };
    }

    macro_rules! register_key_event {
        ($name:ident, $xdo_key_fn:expr) => {
            cx.export_function(stringify!($name), |mut cx| {
                let key_seq: Handle<JsString> = cx.argument(0)?;
                let window = cx.argument_opt(1);
                let display = cx.argument_opt(2);
                let delay = cx.argument_opt(3);

                let key_seq = CString::new(key_seq.value(&mut cx))
                    .map_err(|err| cx.throw_error(err.to_string()).unwrap())?;

                let window = match window {
                    Some(window) => {
                        if window.is_a::<JsUndefined, _>(&mut cx)
                            || window.is_a::<JsNull, _>(&mut cx)
                        {
                            libxdo_sys::CURRENTWINDOW
                        } else {
                            window
                                .downcast_or_throw::<JsNumber, _>(&mut cx)?
                                .value(&mut cx) as _
                        }
                    }
                    None => libxdo_sys::CURRENTWINDOW,
                };

                let xdo = display_to_xdo!(display, cx);

                let delay = match delay {
                    Some(delay) => delay
                        .downcast_or_throw::<JsNumber, _>(&mut cx)?
                        .value(&mut cx) as _,
                    None => 0,
                };

                Ok(cx.number(unsafe { $xdo_key_fn(xdo.0, window, key_seq.as_ptr(), delay) }))
            })?;
        };
    }

    macro_rules! register_mouse_event {
        ($name:ident, $xdo_fn:expr) => {
            cx.export_function(stringify!($name), |mut cx| {
                let button: Handle<JsNumber> = cx.argument(0)?;
                let window = cx.argument_opt(1);
                let display = cx.argument_opt(2);

                let button = button.value(&mut cx);

                let window = match window {
                    Some(window) => {
                        if window.is_a::<JsUndefined, _>(&mut cx)
                            || window.is_a::<JsNull, _>(&mut cx)
                        {
                            libxdo_sys::CURRENTWINDOW
                        } else {
                            window
                                .downcast_or_throw::<JsNumber, _>(&mut cx)?
                                .value(&mut cx) as _
                        }
                    }
                    None => libxdo_sys::CURRENTWINDOW,
                };

                let xdo = display_to_xdo!(display, cx);

                Ok(cx.number(unsafe { $xdo_fn(xdo.0, window, button as _) }))
            })?;
        };
    }

    register_key_event!(keyDown, xdo_send_keysequence_window_down);
    register_key_event!(keyUp, xdo_send_keysequence_window_up);
    register_key_event!(keyPress, xdo_send_keysequence_window);

    register_mouse_event!(mouseDown, xdo_mouse_down);
    register_mouse_event!(mouseUp, xdo_mouse_up);

    cx.export_function("mouseMoveRelativeToWindow", |mut cx| {
        let x: Handle<JsNumber> = cx.argument(0)?;
        let y: Handle<JsNumber> = cx.argument(1)?;
        let window = cx.argument_opt(2);
        let display = cx.argument_opt(3);

        let x = x.value(&mut cx);
        let y = y.value(&mut cx);

        let window = match window {
            Some(window) => {
                if window.is_a::<JsUndefined, _>(&mut cx) || window.is_a::<JsNull, _>(&mut cx) {
                    None
                } else {
                    Some(
                        window
                            .downcast_or_throw::<JsNumber, _>(&mut cx)?
                            .value(&mut cx) as _,
                    )
                }
            }
            None => None,
        };

        let xdo = Arc::new(display_to_xdo!(display, cx));

        let mut origin_x = 0;
        let mut origin_y = 0;
        let mut origin_screen = 0;
        let ret = unsafe {
            xdo_get_mouse_location(xdo.0, &mut origin_x, &mut origin_y, &mut origin_screen)
        };
        if ret != 0 {
            return cx.throw_error(format!("failed to xdo_get_mouse_location : {}", ret));
        }

        let ret = unsafe {
            xdo_move_mouse_relative_to_window(
                xdo.0,
                window.unwrap_or(libxdo_sys::CURRENTWINDOW),
                x as _,
                y as _,
            )
        };
        if ret != 0 {
            return cx.throw_error(format!(
                "failed to xdo_move_mouse_relative_to_window : {}",
                ret
            ));
        }

        let promise = cx
            .task(move || unsafe { xdo_wait_for_mouse_move_from(xdo.0, origin_x, origin_y) })
            .promise(move |mut cx, _| Ok(cx.number(ret)));

        Ok(promise)
    })?;

    cx.export_function("mouseMoveRelative", |mut cx| {
        let x: Handle<JsNumber> = cx.argument(0)?;
        let y: Handle<JsNumber> = cx.argument(1)?;
        let display = cx.argument_opt(2);

        let x = x.value(&mut cx);
        let y = y.value(&mut cx);

        let xdo = Arc::new(display_to_xdo!(display, cx));

        let mut origin_x = 0;
        let mut origin_y = 0;
        let mut origin_screen = 0;
        let ret = unsafe {
            xdo_get_mouse_location(xdo.0, &mut origin_x, &mut origin_y, &mut origin_screen)
        };
        if ret != 0 {
            return cx.throw_error(format!("failed to xdo_get_mouse_location : {}", ret));
        }

        let ret = unsafe { xdo_move_mouse_relative(xdo.0, x as _, y as _) };
        if ret != 0 {
            return cx.throw_error(format!(
                "failed to xdo_move_mouse_relative_to_window : {}",
                ret
            ));
        }

        let promise = cx
            .task(move || unsafe { xdo_wait_for_mouse_move_from(xdo.0, origin_x, origin_y) })
            .promise(move |mut cx, _| Ok(cx.number(ret)));

        Ok(promise)
    })?;

    cx.export_function("mouseMove", |mut cx| {
        let x: Handle<JsNumber> = cx.argument(0)?;
        let y: Handle<JsNumber> = cx.argument(1)?;
        let screen = cx.argument_opt(2);
        let display = cx.argument_opt(3);

        let x = x.value(&mut cx);
        let y = y.value(&mut cx);

        let screen = match screen {
            Some(screen) => {
                if screen.is_a::<JsUndefined, _>(&mut cx) || screen.is_a::<JsNull, _>(&mut cx) {
                    None
                } else {
                    Some(
                        screen
                            .downcast_or_throw::<JsNumber, _>(&mut cx)?
                            .value(&mut cx) as _,
                    )
                }
            }
            None => None,
        }
        .unwrap_or(0 /* CURRENTSCREEN*/);

        let xdo = Arc::new(display_to_xdo!(display, cx));

        let mut origin_x = 0;
        let mut origin_y = 0;
        let mut origin_screen = 0;
        let ret = unsafe {
            xdo_get_mouse_location(xdo.0, &mut origin_x, &mut origin_y, &mut origin_screen)
        };
        if ret != 0 {
            return cx.throw_error(format!("failed to xdo_get_mouse_location : {}", ret));
        }

        let ret = unsafe { xdo_move_mouse(xdo.0, x as _, y as _, screen) };
        if ret != 0 {
            return cx.throw_error(format!(
                "failed to xdo_move_mouse_relative_to_window : {}",
                ret
            ));
        }

        let promise = cx
            .task(move || unsafe { xdo_wait_for_mouse_move_from(xdo.0, origin_x, origin_y) })
            .promise(move |mut cx, _| Ok(cx.number(ret)));

        Ok(promise)
    })?;

    cx.export_function("clickWindow", |mut cx| {
        let button: Handle<JsNumber> = cx.argument(0)?;
        let window = cx.argument_opt(1);
        let display = cx.argument_opt(2);

        let button = button.value(&mut cx);

        let window = match window {
            Some(window) => {
                if window.is_a::<JsUndefined, _>(&mut cx) || window.is_a::<JsNull, _>(&mut cx) {
                    None
                } else {
                    Some(
                        window
                            .downcast_or_throw::<JsNumber, _>(&mut cx)?
                            .value(&mut cx) as _,
                    )
                }
            }
            None => None,
        };

        let xdo = Arc::new(display_to_xdo!(display, cx));

        Ok(cx.number(unsafe {
            xdo_click_window(
                xdo.0,
                window.unwrap_or(libxdo_sys::CURRENTWINDOW),
                button as _,
            )
        }))
    })?;

    cx.export_function("activateWindow", |mut cx| {
        let window_id: Handle<JsNumber> = cx.argument(0)?;
        let display = cx.argument_opt(1);
        let window_id = window_id.value(&mut cx);

        let xdo = Arc::new(display_to_xdo!(display, cx));

        let ret = unsafe { xdo_activate_window(xdo.0, window_id as _) };

        let promise = cx
            .task(move || unsafe {
                xdo_wait_for_window_active(xdo.0, window_id as _, 1);
            })
            .promise(move |mut cx, _| Ok(cx.number(ret)));

        Ok(promise)
    })?;

    cx.export_function("focusWindow", |mut cx| {
        let window_id: Handle<JsNumber> = cx.argument(0)?;
        let display = cx.argument_opt(1);
        let window_id = window_id.value(&mut cx);

        let xdo = Arc::new(display_to_xdo!(display, cx));

        let ret = unsafe { xdo_focus_window(xdo.0, window_id as _) };

        let promise = cx
            .task(move || unsafe {
                xdo_wait_for_window_focus(xdo.0, window_id as _, 1);
            })
            .promise(move |mut cx, _| Ok(cx.number(ret)));

        Ok(promise)
    })?;

    cx.export_function("killWindow", |mut cx| {
        let window_id: Handle<JsNumber> = cx.argument(0)?;
        let display = cx.argument_opt(1);
        let window_id = window_id.value(&mut cx);

        let xdo = Arc::new(display_to_xdo!(display, cx));

        let ret = unsafe { xdo_kill_window(xdo.0, window_id as _) };

        Ok(cx.number(ret))
    })?;

    cx.export_function("getPIDWindow", |mut cx| {
        let window_id: Handle<JsNumber> = cx.argument(0)?;
        let display = cx.argument_opt(1);
        let window_id = window_id.value(&mut cx);

        let xdo = Arc::new(display_to_xdo!(display, cx));

        let ret = unsafe { xdo_get_pid_window(xdo.0, window_id as _) };

        if (ret == -1) {
            return cx.throw_error(format!("invalid pid : ({})", ret));
        }
        Ok(cx.number(ret))
    })?;

    cx.export_function("getWindowAtMouse", |mut cx| {
        let display = cx.argument_opt(0);

        let xdo = Arc::new(display_to_xdo!(display, cx));

        let mut window = 0;
        let ret = unsafe { xdo_get_window_at_mouse(xdo.0, &mut window) };
        if ret != 0 {
            return cx.throw_error(format!("failed to get window at mouse : ({})", ret));
        }
        Ok(cx.number(window as f64))
    })?;

    cx.export_function("getFocusedWindow", |mut cx| {
        let display = cx.argument_opt(0);

        let xdo = Arc::new(display_to_xdo!(display, cx));

        let mut window = 0;
        let ret = unsafe { xdo_get_focused_window(xdo.0, &mut window) };
        if ret != 0 {
            return cx.throw_error(format!("failed to focused window : ({})", ret));
        }
        Ok(cx.number(window as f64))
    })?;

    cx.export_function("getActiveWindow", |mut cx| {
        let display = cx.argument_opt(0);

        let xdo = Arc::new(display_to_xdo!(display, cx));

        let mut window = 0;
        let ret = unsafe { xdo_get_active_window(xdo.0, &mut window) };
        if ret != 0 {
            return cx.throw_error(format!("failed to active window : ({})", ret));
        }
        Ok(cx.number(window as f64))
    })?;

    Ok(())
}
