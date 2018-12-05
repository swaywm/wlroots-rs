use libc::{int16_t, uint16_t};
use wlroots_sys::{wlr_xwayland_move_event, wlr_xwayland_resize_event,
                  wlr_xwayland_surface_configure_event};

use {xwayland::surface::XWaylandSurfaceHandle,
     utils::edges::Edges};

/// Event for when XWayland surface needs to be configured.
pub struct ConfigureEvent {
    event: *mut wlr_xwayland_surface_configure_event
}

/// Event for when an XWayland surface is moved.
pub struct MoveEvent {
    event: *mut wlr_xwayland_move_event
}

/// Event for when an XWayland surface is resized.
pub struct ResizeEvent {
    event: *mut wlr_xwayland_resize_event
}

impl ConfigureEvent {
    pub(crate) unsafe fn from_ptr(event: *mut wlr_xwayland_surface_configure_event) -> Self {
        ConfigureEvent { event }
    }

    /// Get the surface associated with this configure event.
    pub fn surface(&self) -> Option<XWaylandSurfaceHandle> {
        unsafe {
            if (*self.event).surface.is_null() {
                None
            } else {
                Some(XWaylandSurfaceHandle::from_ptr((*self.event).surface))
            }
        }
    }

    /// Get the coordinates for where the XWayland surface wants to be.
    ///
    /// Return format is (x, y).
    pub fn coords(&self) -> (int16_t, int16_t) {
        unsafe { ((*self.event).x, (*self.event).y) }
    }

    /// Get the dimensions the XWayland surface wants to have.
    ///
    /// Return format is (width, height).
    pub fn dimensions(&self) -> (uint16_t, uint16_t) {
        unsafe { ((*self.event).width, (*self.event).height) }
    }
}

impl MoveEvent {
    pub(crate) unsafe fn from_ptr(event: *mut wlr_xwayland_move_event) -> Self {
        MoveEvent { event }
    }

    /// Get the surface associated with this move event.
    pub fn surface(&self) -> Option<XWaylandSurfaceHandle> {
        unsafe {
            if (*self.event).surface.is_null() {
                None
            } else {
                Some(XWaylandSurfaceHandle::from_ptr((*self.event).surface))
            }
        }
    }
}

impl ResizeEvent {
    pub(crate) unsafe fn from_ptr(event: *mut wlr_xwayland_resize_event) -> Self {
        ResizeEvent { event }
    }

    /// Get the surface associated with this resize event.
    pub fn surface(&self) -> Option<XWaylandSurfaceHandle> {
        unsafe {
            if (*self.event).surface.is_null() {
                None
            } else {
                Some(XWaylandSurfaceHandle::from_ptr((*self.event).surface))
            }
        }
    }

    /// Get the resize edge information for the resize action.
    pub fn edges(&self) -> Edges {
        unsafe {
            let edges_bits = (*self.event).edges;
            match Edges::from_bits(edges_bits) {
                Some(edges) => edges,
                None => panic!("got invalid edges: {}", edges_bits)
            }
        }
    }
}
