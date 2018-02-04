//! Events for Wayland shells

use wlroots_sys::{wl_shell_surface_fullscreen_method, wl_shell_surface_resize,
                  wlr_wl_shell_surface_maximize_event, wlr_wl_shell_surface_move_event,
                  wlr_wl_shell_surface_resize_event, wlr_wl_shell_surface_set_fullscreen_event};

use {OutputHandle, WlShellSurfaceHandle};

/// Event that triggers when the surface has been moved in coordinate space.
#[derive(Debug, Eq, PartialEq)]
pub struct MoveEvent {
    event: *mut wlr_wl_shell_surface_move_event
}

/// Event that triggers when the surface are has been resized.
#[derive(Debug, Eq, PartialEq)]
pub struct ResizeEvent {
    event: *mut wlr_wl_shell_surface_resize_event
}

/// Event that triggers when the surface area has been requested to render on
/// the entire screen.
#[derive(Debug, Eq, PartialEq)]
pub struct FullscreenEvent {
    event: *mut wlr_wl_shell_surface_set_fullscreen_event
}

/// Event that triggers when the shell is requested to be maximized.
#[derive(Debug, Eq, PartialEq)]
pub struct MaximizeEvent {
    event: *mut wlr_wl_shell_surface_maximize_event
}

// TODO Get seat client
impl MoveEvent {
    pub(crate) unsafe fn from_ptr(event: *mut wlr_wl_shell_surface_move_event) -> Self {
        MoveEvent { event }
    }
    /// Gets the surface that is being moved.
    pub fn surface(&mut self) -> WlShellSurfaceHandle {
        unsafe { WlShellSurfaceHandle::from_ptr((*self.event).surface) }
    }

    /// TODO Document
    pub fn serial(&self) -> u32 {
        unsafe { (*self.event).serial }
    }
}

// TODO Get seat client
impl ResizeEvent {
    pub(crate) unsafe fn from_ptr(event: *mut wlr_wl_shell_surface_resize_event) -> Self {
        ResizeEvent { event }
    }

    /// Gets the surface that is being resized.
    pub fn surface(&mut self) -> WlShellSurfaceHandle {
        unsafe { WlShellSurfaceHandle::from_ptr((*self.event).surface) }
    }

    /// TODO Document
    pub fn serial(&self) -> u32 {
        unsafe { (*self.event).serial }
    }

    /// Get which edge(s) of the surface were resized from this event.
    pub fn edges(&self) -> wl_shell_surface_resize {
        unsafe { (*self.event).edges }
    }
}

impl FullscreenEvent {
    pub(crate) unsafe fn from_ptr(event: *mut wlr_wl_shell_surface_set_fullscreen_event) -> Self {
        FullscreenEvent { event }
    }

    /// Gets the surface that wants to be a fullscreen
    pub fn surface(&mut self) -> WlShellSurfaceHandle {
        unsafe { WlShellSurfaceHandle::from_ptr((*self.event).surface) }
    }

    /// Get the method that should be used to make the surface fullscreen.
    pub fn method(&self) -> wl_shell_surface_fullscreen_method {
        unsafe { (*self.event).method }
    }

    /// TODO Document
    pub fn framerate(&self) -> u32 {
        unsafe { (*self.event).framerate }
    }

    /// Get the output that the surface wants to be fullscreen on.
    pub fn output(&self) -> OutputHandle {
        unsafe { OutputHandle::from_ptr((*self.event).output) }
    }
}

impl MaximizeEvent {
    pub(crate) unsafe fn from_ptr(event: *mut wlr_wl_shell_surface_maximize_event) -> Self {
        MaximizeEvent { event }
    }

    /// Gets the surface that wants to be maximized.
    pub fn surface(&mut self) -> WlShellSurfaceHandle {
        unsafe { WlShellSurfaceHandle::from_ptr((*self.event).surface) }
    }

    /// Get the output that the surface wants to be maximized on.
    pub fn output(&self) -> OutputHandle {
        unsafe { OutputHandle::from_ptr((*self.event).output) }
    }
}
