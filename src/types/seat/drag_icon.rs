use manager::DragIconHandler;

use wlroots_sys::wlr_drag_icon;
use std::cell::Cell;
use std::rc::{Rc, Weak};

pub struct DragIcon {
    liveliness: Rc<Cell<bool>>,
    drag_icon: *mut wlr_drag_icon
}

impl DragIcon {
    pub(crate) unsafe fn new(drag_icon: *mut wlr_drag_icon) -> Self {
        let liveliness = Rc::new(Cell::new(false));
        let state = Box::new(DragIconState {
            handle: Rc::downgrade(&liveliness),
            drag_icon
        });
        (*drag_icon).data = Box::into_raw(state) as *mut _;
        DragIcon {
            liveliness,
            drag_icon
        }
    }
}

pub struct DragIconHandle {
}

pub(crate) struct DragIconState {
    pub(crate) handle: Weak<Cell<bool>>,
    pub(crate) drag_icon: *mut wlr_drag_icon
}
