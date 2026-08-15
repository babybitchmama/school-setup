use std::ffi::CString;
use std::os::raw::{c_int, c_ulong};
use x11::xlib;

pub fn is_inkscape_focused() -> bool {
    unsafe {
        let display = xlib::XOpenDisplay(std::ptr::null());
        if display.is_null() {
            return false;
        }

        let screen = xlib::XDefaultScreen(display);
        let root = xlib::XRootWindow(display, screen);

        let net_active_window = CString::new("_NET_ACTIVE_WINDOW").unwrap();
        let atom_active = xlib::XInternAtom(display, net_active_window.as_ptr(), xlib::True);

        if atom_active == 0 {
            xlib::XCloseDisplay(display);
            return false;
        }

        let mut actual_type: c_ulong = 0;
        let mut actual_format: c_int = 0;
        let mut nitems: c_ulong = 0;
        let mut bytes_after: c_ulong = 0;
        let mut prop_return: *mut u8 = std::ptr::null_mut();

        let status = xlib::XGetWindowProperty(
            display,
            root,
            atom_active,
            0,
            1,
            xlib::False,
            xlib::XA_WINDOW,
            &mut actual_type,
            &mut actual_format,
            &mut nitems,
            &mut bytes_after,
            &mut prop_return,
        );

        if status != xlib::Success.into() || prop_return.is_null() || nitems == 0 {
            if !prop_return.is_null() {
                xlib::XFree(prop_return as *mut std::os::raw::c_void);
            }
            xlib::XCloseDisplay(display);
            return false;
        }

        let active_window = *(prop_return as *const c_ulong);
        xlib::XFree(prop_return as *mut std::os::raw::c_void);

        if active_window == 0 {
            xlib::XCloseDisplay(display);
            return false;
        }

        let is_ink = check_window_class(display, active_window);
        xlib::XCloseDisplay(display);
        is_ink
    }
}

unsafe fn check_window_class(display: *mut xlib::Display, window: c_ulong) -> bool {
    let wm_class = CString::new("WM_CLASS").unwrap();
    let atom_wm_class = xlib::XInternAtom(display, wm_class.as_ptr(), xlib::True);

    if atom_wm_class == 0 {
        return false;
    }

    let mut actual_type: c_ulong = 0;
    let mut actual_format: c_int = 0;
    let mut nitems: c_ulong = 0;
    let mut bytes_after: c_ulong = 0;
    let mut prop_return: *mut u8 = std::ptr::null_mut();

    let status = xlib::XGetWindowProperty(
        display,
        window,
        atom_wm_class,
        0,
        1024,
        xlib::False,
        xlib::XA_STRING,
        &mut actual_type,
        &mut actual_format,
        &mut nitems,
        &mut bytes_after,
        &mut prop_return,
    );

    if status != xlib::Success.into() || prop_return.is_null() {
        if !prop_return.is_null() {
            xlib::XFree(prop_return as *mut std::os::raw::c_void);
        }
        return false;
    }

    let slice = std::slice::from_raw_parts(prop_return, nitems as usize);
    let class_string = String::from_utf8_lossy(slice);
    xlib::XFree(prop_return as *mut std::os::raw::c_void);

    class_string.to_lowercase().contains("inkscape")
}
