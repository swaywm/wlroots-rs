//! Some events refer to actions that occurred on certain "edges" of windows.
//! This is represented as a bit flag since multiple edges (including none)
//! could be affected.

use wlroots_sys::wlr_edges;

bitflags! {
    /// A bit flag representing which edge was affected by an event.
    pub struct Edges: u32 {
        const WLR_EDGE_NONE = wlr_edges::WLR_EDGE_NONE as u32;
        const WLR_EDGE_TOP = wlr_edges::WLR_EDGE_TOP as u32;
        const WLR_EDGE_BOTTOM = wlr_edges::WLR_EDGE_BOTTOM as u32;
        const WLR_EDGE_LEFT = wlr_edges::WLR_EDGE_LEFT as u32;
        const WLR_EDGE_RIGHT = wlr_edges::WLR_EDGE_RIGHT as u32;
    }
}
