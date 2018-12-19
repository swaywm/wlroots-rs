//! Handles the subsurface list for surfaces.
//!
//! This handler will live in the Surface struct, behind a box
//! and is not exposed to the compositor writer.
//!
//! It ensures the list of subsurfaces is kept up to date and invalidates
//! the subsurface handles when they are destroyed.

use std::fmt;

use libc;
use wayland_sys::server::signal::wl_signal_add;
use wlroots_sys::wlr_subsurface;

use {surface::subsurface::{self, Subsurface}, utils::Handleable};

wayland_listener!(pub SubsurfaceManager, Vec<Subsurface>, [
    subsurface_created_listener => subsurface_created_notify:
                                   |this: &mut SubsurfaceManager, data: *mut libc::c_void,|
    unsafe {
        wl_signal_add(&mut (*(data as *mut wlr_subsurface)).events.destroy as *mut _ as _,
                      this.subsurface_destroyed_listener() as _);
        let subsurfaces = &mut this.data;
        let subsurface = Subsurface::new(data as *mut wlr_subsurface);
        subsurfaces.push(subsurface)
    };
    subsurface_destroyed_listener => subsurface_destroyed_listner:
                                     |this: &mut SubsurfaceManager, data: *mut libc::c_void,|
    unsafe {
        let subsurfaces = &mut this.data;
        let subsurface = data as *mut wlr_subsurface;
        if let Some(index) = subsurfaces.iter().position(|cur| cur.as_ptr() == subsurface) {
            subsurfaces.remove(index);
        }
    };
]);

impl SubsurfaceManager {
    pub(crate) fn subsurfaces(&self) -> Vec<subsurface::Handle> {
        self.data.iter()
            .map(|surface| surface.weak_reference())
            .collect()
    }
}

impl fmt::Debug for SubsurfaceManager {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.data)
    }
}
