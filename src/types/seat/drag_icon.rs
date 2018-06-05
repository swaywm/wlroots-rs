use wlroots_sys::wlr_drag_icon;
use std::cell::Cell;
use std::rc::{Rc, Weak};
use std::panic;
use errors::{HandleErr, HandleResult};
use DragIconHandler;

#[derive(Debug)]
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

    unsafe fn from_handle(handle: &DragIconHandle) -> HandleResult<Self> {
        let liveliness = handle.handle
                               .upgrade()
                               .ok_or_else(|| HandleErr::AlreadyDropped)?;
        Ok(DragIcon { liveliness,
                      drag_icon: handle.as_ptr() })
    }

    pub fn weak_reference(&self) -> DragIconHandle {
        DragIconHandle {
            handle: Rc::downgrade(&self.liveliness),
            drag_icon: self.drag_icon
        }
    }
}

pub struct DragIconHandle {
    handle: Weak<Cell<bool>>,
    drag_icon: *mut wlr_drag_icon
}

impl DragIconHandle {
    pub(crate) unsafe fn from_ptr(drag_icon: *mut wlr_drag_icon) -> Self {
        if drag_icon.is_null() {
            panic!("drag icon was null");
        }
        let data = (*drag_icon).data as *mut DragIconState;
        if data.is_null() {
            panic!("Cannot construct handle from drag icon that has not been set up!");
        }

        let handle = (*data).handle.clone();

        DragIconHandle {
            handle,
            drag_icon
        }
    }

    pub(crate) unsafe fn upgrade(&self) -> HandleResult<DragIcon> {
        self.handle.upgrade()
            .ok_or(HandleErr::AlreadyDropped)
            // NOTE
            // We drop the Rc here because having two would allow a dangling
            // pointer to exist!
            .and_then(|check| {
                let drag_icon = DragIcon::from_handle(self)?;
                if check.get() {
                    return Err(HandleErr::AlreadyBorrowed)
                }
                check.set(true);
                Ok(drag_icon)
            })
    }

    pub fn run<F, R>(&self, runner: F) -> HandleResult<R>
        where F: FnOnce(&mut DragIcon) -> R
    {
        let mut drag_icon = unsafe { self.upgrade()? };
        let res = panic::catch_unwind(panic::AssertUnwindSafe(|| runner(&mut drag_icon)));
        self.handle.upgrade().map(|check| {
                                      // Sanity check that it hasn't been tampered with.
                                      if !check.get() {
                                          wlr_log!(L_ERROR,
                                                   "After running DragIcon callback, \
                                                    mutable lock was false for: {:?}",
                                                   drag_icon);
                                          panic!("Lock in incorrect state!");
                                      }
                                      check.set(false);
                                  });
        match res {
            Ok(res) => Ok(res),
            Err(err) => panic::resume_unwind(err)
        }
    }

    unsafe fn as_ptr(&self) -> *mut wlr_drag_icon {
        self.drag_icon
    }
}

pub(crate) struct DragIconState {
    pub(crate) handle: Weak<Cell<bool>>,
    pub(crate) drag_icon: *mut wlr_drag_icon
}
