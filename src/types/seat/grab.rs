use wlroots_sys::{wlr_seat_keyboard_grab, wlr_seat_pointer_grab, wlr_seat_touch_grab};

pub struct Pointer {
    grab: *mut wlr_seat_pointer_grab
}

pub struct Keyboard {
    grab: *mut wlr_seat_keyboard_grab
}

pub struct Touch {
    grab: *mut wlr_seat_touch_grab
}

#[allow(dead_code)]
impl Pointer {
    pub(crate) unsafe fn as_ptr(&self) -> *mut wlr_seat_pointer_grab {
        self.grab
    }

    pub(crate) unsafe fn from_ptr(grab: *mut wlr_seat_pointer_grab) -> Self {
        Pointer { grab }
    }
}

#[allow(dead_code)]
impl Keyboard {
    pub(crate) unsafe fn as_ptr(&self) -> *mut wlr_seat_keyboard_grab {
        self.grab
    }

    pub(crate) unsafe fn from_ptr(grab: *mut wlr_seat_keyboard_grab) -> Self {
        Keyboard { grab }
    }
}

#[allow(dead_code)]
impl Touch {
    pub(crate) unsafe fn as_ptr(&self) -> *mut wlr_seat_touch_grab {
        self.grab
    }

    pub(crate) unsafe fn from_ptr(grab: *mut wlr_seat_touch_grab) -> Self {
        Touch { grab }
    }
}
