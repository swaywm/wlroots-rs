use std::{cell::Cell, rc::{Rc, Weak}, hash::{Hash, Hasher}, panic};

use wlroots_sys::wlr_drag_icon;

use {surface, utils::{HandleErr, HandleResult}};
pub use manager::drag_icon_handler::*;

#[derive(Debug)]
pub struct DragIcon {
    liveliness: Rc<Cell<bool>>,
    drag_icon: *mut wlr_drag_icon
}

impl DragIcon {
    #[allow(dead_code)]
    pub(crate) unsafe fn new(drag_icon: *mut wlr_drag_icon) -> Self {
        let liveliness = Rc::new(Cell::new(false));
        let state = Box::new(DragIconState { handle: Rc::downgrade(&liveliness) });
        (*drag_icon).data = Box::into_raw(state) as *mut _;
        DragIcon {
            liveliness,
            drag_icon
        }
    }

    /// Get a handle for the surface associated with this drag icon
    pub fn surface(&mut self) -> surface::Handle {
        unsafe {
            let surface = (*self.drag_icon).surface;
            if surface.is_null() {
                panic!("drag icon had a null surface!");
            }
            surface::Handle::from_ptr(surface)
        }
    }

    /// Whether or not to display the drag icon
    pub fn mapped(&mut self) -> bool {
        unsafe { (*self.drag_icon).mapped }
    }

    /// If this is a touch-driven dnd operation, the id of the touch point that started it
    pub fn touch_id(&mut self) -> i32 {
        unsafe { (*(*self.drag_icon).drag).touch_id }
    }

    /// Creates a weak reference to a `DragIcon`.
    pub fn weak_reference(&self) -> Handle {
        Handle {
            handle: Rc::downgrade(&self.liveliness),
            drag_icon: self.drag_icon
        }
    }

    unsafe fn from_handle(handle: &Handle) -> HandleResult<Self> {
        let liveliness = handle.handle
                               .upgrade()
                               .ok_or_else(|| HandleErr::AlreadyDropped)?;
        Ok(DragIcon { liveliness,
                      drag_icon: handle.as_ptr() })
    }
}

pub(crate) struct DragIconState {
    handle: Weak<Cell<bool>>
}

#[derive(Debug, Clone)]
pub struct Handle {
    handle: Weak<Cell<bool>>,
    drag_icon: *mut wlr_drag_icon
}

impl Eq for Handle {}

impl PartialEq for Handle {
    fn eq(&self, rhs: &Self) -> bool {
        self.drag_icon == rhs.drag_icon
    }
}

impl Hash for Handle {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.drag_icon.hash(state);
    }
}

impl Handle {
    #[allow(unused)]
    pub(crate) unsafe fn from_ptr(drag_icon: *mut wlr_drag_icon) -> Self {
        if drag_icon.is_null() {
            panic!("drag icon was null");
        }
        let data = (*drag_icon).data as *mut DragIconState;
        if data.is_null() {
            panic!("Cannot construct handle from drag icon that has not been set up!");
        }

        let handle = (*data).handle.clone();

        Handle {
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
                                          wlr_log!(WLR_ERROR,
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
