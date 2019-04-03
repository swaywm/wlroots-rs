//! Handles the subsurface list for surfaces.
//!
//! This handler will live in the Surface struct, behind a box
//! and is not exposed to the compositor writer.
//!
//! It ensures the list of subsurfaces is kept up to date and invalidates
//! the subsurface handles when they are destroyed.

use std::fmt;

use crate::libc;
use wlroots_sys::wlr_subsurface;

use crate::{
    surface::subsurface::{self, Subsurface},
    utils::Handleable
};

wayland_listener!(pub SubsurfaceManager, Vec<Subsurface>, [
    subsurface_created_listener => subsurface_created_notify:
                                   |this: &mut SubsurfaceManager, data: *mut libc::c_void,|
    unsafe {
        let subsurfaces = &mut this.data;
        let subsurface = Subsurface::new(data as *mut wlr_subsurface);
        subsurfaces.push(subsurface)
    };
]);

impl SubsurfaceManager {
    pub(crate) fn subsurfaces(&self) -> Vec<subsurface::Handle> {
        self.data.iter().map(|surface| surface.weak_reference()).collect()
    }
}

impl fmt::Debug for SubsurfaceManager {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.data)
    }
}
