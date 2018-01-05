use wlroots_sys::{wlr_seat_keyboard_grab, wlr_seat_pointer_grab, wlr_seat_touch_grab};

pub struct PointerGrab {
    grab: *mut wlr_seat_pointer_grab
}

pub struct KeyboardGrab {
    grab: *mut wlr_seat_keyboard_grab
}

pub struct TouchGrab {
    grab: *mut wlr_seat_touch_grab
}

impl PointerGrab {
    pub unsafe fn as_ptr(&self) -> *mut wlr_seat_pointer_grab {
        self.grab
    }

    pub unsafe fn from_ptr(grab: *mut wlr_seat_pointer_grab) -> Self {
        PointerGrab { grab }
    }
}

impl KeyboardGrab {
    pub unsafe fn as_ptr(&self) -> *mut wlr_seat_keyboard_grab {
        self.grab
    }

    pub unsafe fn from_ptr(grab: *mut wlr_seat_keyboard_grab) -> Self {
        KeyboardGrab { grab }
    }
}

impl TouchGrab {
    pub unsafe fn as_ptr(&self) -> *mut wlr_seat_touch_grab {
        self.grab
    }

    pub unsafe fn from_ptr(grab: *mut wlr_seat_touch_grab) -> Self {
        TouchGrab { grab }
    }
}
