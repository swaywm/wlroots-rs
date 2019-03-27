//! Handler for drag icons

use crate::{compositor, seat::drag_icon};

/// Handles events from the wlr drag icon
#[allow(unused_variables)]
pub trait Handler {
    /// Called when the drag icon is ready to be displayed.
    fn on_map(&mut self, compositor_handle: compositor::Handle, drag_icon_handle: drag_icon::Handle);

    /// Called when the drag icon should no longer be displayed
    fn on_unmap(&mut self, compositor_handle: compositor::Handle, drag_icon_handle: drag_icon::Handle);

    /// Called when the drag icon is about to be destroyed.
    fn destroyed(&mut self, compositor_handle: compositor::Handle, drag_icon_handle: drag_icon::Handle);
}
