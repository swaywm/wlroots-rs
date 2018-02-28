//! Events for XDG shell v6

use wlroots_sys::{wlr_xdg_toplevel_v6_move_event, wlr_xdg_toplevel_v6_resize_event,
                  wlr_xdg_toplevel_v6_set_fullscreen_event,
                  wlr_xdg_toplevel_v6_show_window_menu_event};

use {OutputHandle, XdgV6ShellSurfaceHandle};

/// Event that triggers when the surface has been moved in coordinate space.
#[derive(Debug, PartialEq, Eq)]
pub struct MoveEvent {
    event: *mut wlr_xdg_toplevel_v6_move_event
}

/// Event that triggers when the suface has been resized.
#[derive(Debug, PartialEq, Eq)]
pub struct ResizeEvent {
    event: *mut wlr_xdg_toplevel_v6_resize_event
}

/// Event that is triggered when the surface toggles between being fullscreen
/// or not.
#[derive(Debug, PartialEq, Eq)]
pub struct SetFullscreenEvent {
    event: *mut wlr_xdg_toplevel_v6_set_fullscreen_event
}

/// Event that is triggered when the surface shows the window menu.
#[derive(Debug, PartialEq, Eq)]
pub struct ShowWindowMenuEvent {
    event: *mut wlr_xdg_toplevel_v6_show_window_menu_event
}

impl MoveEvent {
    pub(crate) unsafe fn from_ptr(event: *mut wlr_xdg_toplevel_v6_move_event) -> Self {
        MoveEvent { event }
    }

    /// Get a handle to the surface associated with this event.
    pub fn surface(&self) -> XdgV6ShellSurfaceHandle {
        unsafe { XdgV6ShellSurfaceHandle::from_ptr((*self.event).surface) }
    }

    // TODO Get seat client

    pub fn serial(&self) -> u32 {
        unsafe { (*self.event).serial }
    }
}

impl ResizeEvent {
    pub(crate) unsafe fn from_ptr(event: *mut wlr_xdg_toplevel_v6_resize_event) -> Self {
        ResizeEvent { event }
    }

    /// Get a handle to the surface associated with this event.
    pub fn surface(&self) -> XdgV6ShellSurfaceHandle {
        unsafe { XdgV6ShellSurfaceHandle::from_ptr((*self.event).surface) }
    }

    // TODO Get seat client

    pub fn serial(&self) -> u32 {
        unsafe { (*self.event).serial }
    }

    pub fn edges(&self) -> u32 {
        unsafe { (*self.event).edges }
    }
}

impl SetFullscreenEvent {
    pub(crate) unsafe fn from_ptr(event: *mut wlr_xdg_toplevel_v6_set_fullscreen_event) -> Self {
        SetFullscreenEvent { event }
    }

    /// Get a handle to the surface associated with this event.
    pub fn surface(&self) -> XdgV6ShellSurfaceHandle {
        unsafe { XdgV6ShellSurfaceHandle::from_ptr((*self.event).surface) }
    }

    /// Determine if the event is to trigger fullscreen or to stop being
    /// fullscreen.
    pub fn fullscreen(&self) -> bool {
        unsafe { (*self.event).fullscreen }
    }

    /// Get a handle to the output that this fullscreen event refers to.
    pub fn output(&self) -> OutputHandle {
        unsafe { OutputHandle::from_ptr((*self.event).output) }
    }
}

impl ShowWindowMenuEvent {
    pub(crate) unsafe fn from_ptr(event: *mut wlr_xdg_toplevel_v6_show_window_menu_event) -> Self {
        ShowWindowMenuEvent { event }
    }

    /// Get a handle to the surface associated with this event.
    pub fn surface(&self) -> XdgV6ShellSurfaceHandle {
        unsafe { XdgV6ShellSurfaceHandle::from_ptr((*self.event).surface) }
    }

    // TODO seat client

    pub fn serial(&self) -> u32 {
        unsafe { (*self.event).serial }
    }

    /// Get the coordinates for where this show menu event takes place.
    ///
    /// Return value is in (x, y) format.
    pub fn coords(&self) -> (u32, u32) {
        unsafe { ((*self.event).x, (*self.event).y) }
    }
}
