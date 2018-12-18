//! Manager for XDG shell v6 client.

use libc;
use wayland_sys::server::signal::wl_signal_add;
use wlroots_sys::{wlr_xdg_surface_v6, wlr_xdg_surface_v6_role::*};

use {compositor,
     shell::xdg_shell_v6::{self, ShellState},
     surface,
     utils::Handleable};
use super::xdg_shell_v6_handler::XdgShellV6;

pub trait ManagerHandler {
    /// Callback that is triggered when a new XDG shell v6 surface appears.
    fn new_surface(&mut self,
                   compositor_handle: compositor::Handle,
                   xdg_shell_v6_handle: xdg_shell_v6::Handle)
                   -> (Option<Box<xdg_shell_v6::Handler>>, Option<Box<surface::Handler>>);
}

wayland_listener!(pub(crate) Manager, Box<ManagerHandler>, [
    add_listener => add_notify: |this: &mut Manager, data: *mut libc::c_void,|
    unsafe {
        let ref mut manager = this.data;
        let data = data as *mut wlr_xdg_surface_v6;
        let compositor = match compositor::handle() {
            Some(handle) => handle,
            None => return
        };
        wlr_log!(WLR_DEBUG, "New xdg_shell_v6_surface request {:p}", data);
        let state = unsafe {
            match (*data).role {
                WLR_XDG_SURFACE_V6_ROLE_NONE => None,
                WLR_XDG_SURFACE_V6_ROLE_TOPLEVEL => {
                    let toplevel = (*data).__bindgen_anon_1.toplevel;
                    Some(ShellState::TopLevel(xdg_shell_v6::TopLevel::from_shell(data, toplevel)))
                }
                WLR_XDG_SURFACE_V6_ROLE_POPUP => {
                    let popup = (*data).__bindgen_anon_1.popup;
                    Some(ShellState::Popup(xdg_shell_v6::Popup::from_shell(data, popup)))
                }
            }
        };
        let shell_surface = xdg_shell_v6::Surface::new(data, state);

        let (shell_surface_handler, surface_handler) =
            manager.new_surface(compositor, shell_surface.weak_reference());

        let mut shell_surface = XdgShellV6::new((shell_surface, shell_surface_handler));
        let surface_state = (*(*data).surface).data as *mut surface::InternalState;
        if let Some(surface_handler) = surface_handler {
            (*(*surface_state).surface).data().1 = surface_handler;
        }

        wl_signal_add(&mut (*data).events.destroy as *mut _ as _,
                        shell_surface.destroy_listener() as _);
        wl_signal_add(&mut (*(*data).surface).events.commit as *mut _ as _,
                        shell_surface.commit_listener() as _);
        wl_signal_add(&mut (*data).events.ping_timeout as *mut _ as _,
                        shell_surface.ping_timeout_listener() as _);
        wl_signal_add(&mut (*data).events.new_popup as *mut _ as _,
                        shell_surface.new_popup_listener() as _);
        wl_signal_add(&mut (*data).events.map as *mut _ as _,
                        shell_surface.map_listener() as _);
        wl_signal_add(&mut (*data).events.unmap as *mut _ as _,
                        shell_surface.unmap_listener() as _);
        let events = with_handles!([(shell_surface: {shell_surface.surface_mut()})] => {
            match shell_surface.state() {
                None | Some(&mut ShellState::Popup(_)) => None,
                Some(&mut ShellState::TopLevel(ref mut toplevel)) => Some((*toplevel.as_ptr()).events)
            }
        }).expect("Cannot borrow xdg shell surface");
        if let Some(mut events) = events {
            wl_signal_add(&mut events.request_maximize as *mut _ as _,
                            shell_surface.maximize_listener() as _);
            wl_signal_add(&mut events.request_fullscreen as *mut _ as _,
                            shell_surface.fullscreen_listener() as _);
            wl_signal_add(&mut events.request_minimize as *mut _ as _,
                            shell_surface.minimize_listener() as _);
            wl_signal_add(&mut events.request_move as *mut _ as _,
                            shell_surface.move_listener() as _);
            wl_signal_add(&mut events.request_resize as *mut _ as _,
                            shell_surface.resize_listener() as _);
            wl_signal_add(&mut events.request_show_window_menu as *mut _ as _,
                            shell_surface.show_window_menu_listener() as _);
        }

        let shell_data = (*data).data as *mut xdg_shell_v6::SurfaceState;
        (*shell_data).shell = Box::into_raw(shell_surface);
    };
]);
