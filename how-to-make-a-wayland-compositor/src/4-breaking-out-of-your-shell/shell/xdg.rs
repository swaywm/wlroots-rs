use crate::{CompositorState, Shells, view::View};

use wlroots::{wlroots_dehandle,
              compositor,
              surface,
              shell::xdg_shell};

struct SurfaceHandler;

impl surface::Handler for SurfaceHandler {}

struct XdgShellHandler;

impl xdg_shell::Handler for XdgShellHandler {
    #[wlroots_dehandle]
    fn map_request(&mut self,
                   compositor_handle: compositor::Handle,
                   _surface_handle: surface::Handle,
                   shell_handle: xdg_shell::Handle) {
        #[dehandle] let compositor = compositor_handle;
        let CompositorState { shells: Shells { ref mut mapped_shells, .. }, ..} =
            compositor.downcast();
        mapped_shells.push_back(shell_handle.clone().into());
    }

    #[wlroots_dehandle]
    fn unmap_request(&mut self,
                     compositor_handle: compositor::Handle,
                     _surface_handle: surface::Handle,
                     shell_handle: xdg_shell::Handle) {
        #[dehandle] let compositor = compositor_handle;
        let CompositorState { shells: Shells { ref mut mapped_shells, ..}, .. } =
            compositor.downcast();
        let shell_handle = shell_handle.into();
        mapped_shells.retain(|shell| *shell != shell_handle);
    }

    #[wlroots_dehandle]
    fn destroyed(&mut self,
                 compositor_handle: compositor::Handle,
                 shell_handle: xdg_shell::Handle) {
        #[dehandle] let compositor = compositor_handle;
        let CompositorState { shells: Shells { ref mut mapped_shells, ref mut views }, .. } =
            compositor.downcast();
        let shell_handle = shell_handle.into();
        mapped_shells.retain(|shell| *shell != shell_handle);
        views.remove(&shell_handle);
    }
}


#[wlroots_dehandle]
pub fn xdg_new_surface(compositor_handle: compositor::Handle,
                       shell_handle: xdg_shell::Handle)
                       -> (Option<Box<xdg_shell::Handler>>, Option<Box<surface::Handler>>) {
    #[dehandle] let compositor = compositor_handle;
    let CompositorState { shells: Shells { ref mut views, .. }, .. } =
        compositor.downcast();
    views.insert(shell_handle.clone().into(), View::new(shell_handle));
    (Some(Box::new(XdgShellHandler) as _), Some(Box::new(SurfaceHandler) as _))
}
