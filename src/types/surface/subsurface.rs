//! TODO Documentation

use std::{cell::Cell, rc::Rc};

use libc;
use wayland_sys::server::WAYLAND_SERVER_HANDLE;
use wlroots_sys::wlr_subsurface;

use {compositor,
     surface,
     utils::{self, HandleErr, HandleResult, Handleable}};

pub type Handle = utils::Handle<(), wlr_subsurface, Subsurface>;

#[allow(unused_variables)]
pub trait Handler {
    fn on_destroy(&mut self,
                  compositor_handle: compositor::Handle,
                  subsurface_handle: Handle,
                  surface_handle: surface::Handle) {}
}

wayland_listener!(pub(crate) InternalSubsurface, (Subsurface, Box<Handler>), [
    on_destroy_listener => on_destroy_notify: |this: &mut InternalSubsurface,
                                               data: *mut libc::c_void,|
    unsafe {
        let (ref mut subsurface, ref mut manager) = this.data;
        let subsurface_ptr = data as *mut wlr_subsurface;
        let surface = surface::Handle::from_ptr((*subsurface_ptr).surface);
        let compositor = match compositor::handle() {
            Some(handle) => handle,
            None => return
        };
        manager.on_destroy(compositor, subsurface.weak_reference(), surface);
        Box::from_raw((*subsurface_ptr).data as *mut InternalSubsurface);
    };
]);

#[derive(Debug)]
pub struct Subsurface {
    /// The structe that ensures weak handles to this structure are still alive.
    ///
    /// They contain weak handles, and will safely not use dead memory when this
    /// is freed by wlroots.
    ///
    /// If this is `None`, then this is from an upgraded `surface::Handle`, and
    /// the operations are **unchecked**.
    /// This is means safe operations might fail, but only if you use the unsafe
    /// marked function `upgrade` on a `surface::Handle`.
    liveliness: Rc<Cell<bool>>,
    /// The pointer to the wlroots object that wraps a wl_surface.
    subsurface: *mut wlr_subsurface
}

impl Subsurface {
    pub(crate) unsafe fn new(subsurface: *mut wlr_subsurface) -> Self {
        let liveliness = Rc::new(Cell::new(false));
        Subsurface { subsurface,
                     liveliness }
    }

    pub(crate) unsafe fn as_ptr(&self) -> *mut wlr_subsurface {
        self.subsurface
    }

    /// Get a handle to the surface for this sub surface.
    pub fn surface(&self) -> surface::Handle {
        unsafe { surface::Handle::from_ptr((*self.subsurface).surface) }
    }

    /// Get a handle to the parent surface for this sub surface.
    pub fn parent_surface(&self) -> surface::Handle {
        unsafe { surface::Handle::from_ptr((*self.subsurface).parent) }
    }

    /// Get the cached state of the sub surface.
    pub fn cached_state<'surface>(&'surface self) -> Option<surface::State<'surface>> {
        unsafe {
            if (*self.subsurface).has_cache {
                None
            } else {
                Some(surface::State::new((*self.subsurface).cached))
            }
        }
    }

    /// Determine if the sub surface has a cached state.
    pub fn has_cache(&self) -> bool {
        unsafe { (*self.subsurface).has_cache }
    }

    pub fn synchronized(&self) -> bool {
        unsafe { (*self.subsurface).synchronized }
    }

    pub fn reordered(&self) -> bool {
        unsafe { (*self.subsurface).reordered }
    }
}

impl Handleable<(), wlr_subsurface> for Subsurface {
    #[doc(hidden)]
    unsafe fn from_ptr(subsurface: *mut wlr_subsurface) -> Self {
        let data = (*subsurface).data as *mut InternalSubsurface;
        Subsurface {
            liveliness: (*data).data.0.liveliness.clone(),
            subsurface
        }
    }

    #[doc(hidden)]
    unsafe fn as_ptr(&self) -> *mut wlr_subsurface {
        self.subsurface
    }

    #[doc(hidden)]
    unsafe fn from_handle(handle: &Handle) -> HandleResult<Self> {
        let liveliness = handle.handle
            .upgrade()
            .ok_or_else(|| HandleErr::AlreadyDropped)?;
        Ok(Subsurface { liveliness,
                        subsurface: handle.ptr })
    }

    fn weak_reference(&self) -> Handle {
        Handle { ptr: self.subsurface,
                 handle: Rc::downgrade(&self.liveliness),
                 data: (),
                 _marker: std::marker::PhantomData
        }
    }
}

impl Drop for InternalSubsurface {
    fn drop(&mut self) {
        unsafe {
            ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                          wl_list_remove,
                          &mut (*self.on_destroy_listener()).link as *mut _ as _);
        }
    }
}
