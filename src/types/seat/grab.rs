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

#[allow(dead_code)]
impl PointerGrab {
    pub(crate) unsafe fn as_ptr(&self) -> *mut wlr_seat_pointer_grab {
        self.grab
    }

    pub(crate) unsafe fn from_ptr(grab: *mut wlr_seat_pointer_grab) -> Self {
        PointerGrab { grab }
    }
}

#[allow(dead_code)]
impl KeyboardGrab {
    pub(crate) unsafe fn as_ptr(&self) -> *mut wlr_seat_keyboard_grab {
        self.grab
    }

    pub(crate) unsafe fn from_ptr(grab: *mut wlr_seat_keyboard_grab) -> Self {
        KeyboardGrab { grab }
    }
}

#[allow(dead_code)]
impl TouchGrab {
    pub(crate) unsafe fn as_ptr(&self) -> *mut wlr_seat_touch_grab {
        self.grab
    }

    pub(crate) unsafe fn from_ptr(grab: *mut wlr_seat_touch_grab) -> Self {
        TouchGrab { grab }
    }
}
