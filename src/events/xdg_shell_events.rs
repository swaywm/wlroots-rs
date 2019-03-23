//! Events for stable XDG shell

use wlroots_sys::{
    wlr_xdg_toplevel_move_event, wlr_xdg_toplevel_resize_event, wlr_xdg_toplevel_set_fullscreen_event,
    wlr_xdg_toplevel_show_window_menu_event
};

use {output, shell::xdg_shell, utils::edges::Edges};

/// Event that triggers when the surface has been moved in coordinate space.
#[derive(Debug, PartialEq, Eq)]
pub struct Move {
    event: *mut wlr_xdg_toplevel_move_event
}

/// Event that triggers when the suface has been resized.
#[derive(Debug, PartialEq, Eq)]
pub struct Resize {
    event: *mut wlr_xdg_toplevel_resize_event
}

/// Event that is triggered when the surface toggles between being fullscreen
/// or not.
#[derive(Debug, PartialEq, Eq)]
pub struct SetFullscreen {
    event: *mut wlr_xdg_toplevel_set_fullscreen_event
}

/// Event that is triggered when the surface shows the window menu.
#[derive(Debug, PartialEq, Eq)]
pub struct ShowWindowMenu {
    event: *mut wlr_xdg_toplevel_show_window_menu_event
}

impl Move {
    pub(crate) unsafe fn from_ptr(event: *mut wlr_xdg_toplevel_move_event) -> Self {
        Move { event }
    }

    /// Get a handle to the surface associated with this event.
    pub fn surface(&self) -> xdg_shell::Handle {
        unsafe { xdg_shell::Handle::from_ptr((*self.event).surface) }
    }

    // TODO Get seat client

    pub fn serial(&self) -> u32 {
        unsafe { (*self.event).serial }
    }
}

impl Resize {
    pub(crate) unsafe fn from_ptr(event: *mut wlr_xdg_toplevel_resize_event) -> Self {
        Resize { event }
    }

    /// Get a handle to the surface associated with this event.
    pub fn surface(&self) -> xdg_shell::Handle {
        unsafe { xdg_shell::Handle::from_ptr((*self.event).surface) }
    }

    // TODO Get seat client

    pub fn serial(&self) -> u32 {
        unsafe { (*self.event).serial }
    }

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

impl SetFullscreen {
    pub(crate) unsafe fn from_ptr(event: *mut wlr_xdg_toplevel_set_fullscreen_event) -> Self {
        SetFullscreen { event }
    }

    /// Get a handle to the surface associated with this event.
    pub fn surface(&self) -> xdg_shell::Handle {
        unsafe { xdg_shell::Handle::from_ptr((*self.event).surface) }
    }

    /// Determine if the event is to trigger fullscreen or to stop being
    /// fullscreen.
    pub fn fullscreen(&self) -> bool {
        unsafe { (*self.event).fullscreen }
    }

    /// Get a handle to the output that this fullscreen event refers to.
    pub fn output(&self) -> output::Handle {
        unsafe { output::Handle::from_ptr((*self.event).output) }
    }
}

impl ShowWindowMenu {
    pub(crate) unsafe fn from_ptr(event: *mut wlr_xdg_toplevel_show_window_menu_event) -> Self {
        ShowWindowMenu { event }
    }

    /// Get a handle to the surface associated with this event.
    pub fn surface(&self) -> xdg_shell::Handle {
        unsafe { xdg_shell::Handle::from_ptr((*self.event).surface) }
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
