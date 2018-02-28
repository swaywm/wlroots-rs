//! Handler for XDG shell v6 clients.

use libc;

use wlroots_sys::wlr_xdg_surface_v6;

use {Surface, XdgV6Popup, XdgV6ShellSurface};
use compositor::{Compositor, COMPOSITOR_PTR};
use xdg_shell_v6_events::{MoveEvent, ResizeEvent, SetFullscreenEvent, ShowWindowMenuEvent};

/// Handles events from the client XDG v6 shells.
pub trait XdgV6ShellHandler {
    /// Called when the wayland shell is destroyed (e.g by the user)
    fn destroy(&mut self, &mut Compositor, &mut Surface, &mut XdgV6ShellSurface) {}

    /// Called when the ping request timed out.
    ///
    /// This usually indicates something is wrong with the client.
    fn ping_timeout(&mut self, &mut Compositor, &mut Surface, &mut XdgV6ShellSurface) {}

    /// Called when a new popup appears in the xdg tree.
    fn new_popup(&mut self,
                 &mut Compositor,
                 &mut Surface,
                 &mut XdgV6ShellSurface,
                 &mut XdgV6Popup) {
    }

    /// Called when there is a request to maximize the XDG surface.
    fn maximize_request(&mut self, &mut Compositor, &mut Surface, &mut XdgV6ShellSurface) {}

    /// Called when there is a request to minimize the XDG surface.
    fn minimize_request(&mut self, &mut Compositor, &mut Surface, &mut XdgV6ShellSurface) {}

    /// Called when there is a request to move the shell surface somewhere else.
    fn move_request(&mut self,
                    &mut Compositor,
                    &mut Surface,
                    &mut XdgV6ShellSurface,
                    &mut MoveEvent) {
    }

    /// Called when there is a request to resize the shell surface.
    fn resize_request(&mut self,
                      &mut Compositor,
                      &mut Surface,
                      &mut XdgV6ShellSurface,
                      &mut ResizeEvent) {
    }

    /// Called when there is a request to make the shell surface fullscreen.
    fn fullscreen_request(&mut self,
                          &mut Compositor,
                          &mut Surface,
                          &mut XdgV6ShellSurface,
                          &mut SetFullscreenEvent) {
    }

    /// Called when there is a request to show the window menu.
    fn show_window_menu_request(&mut self,
                                &mut Compositor,
                                &mut Surface,
                                &mut XdgV6ShellSurface,
                                &mut ShowWindowMenuEvent) {
    }
}

wayland_listener!(XdgV6Shell, (XdgV6ShellSurface, Surface, Box<XdgV6ShellHandler>), [
    ping_timeout_listener => ping_timeout_notify: |this: &mut XdgV6Shell,
                                                   _data: *mut libc::c_void,|
    unsafe {
        let (ref mut shell_surface, ref mut surface, ref mut manager) = this.data;
        let compositor = &mut *COMPOSITOR_PTR;
        shell_surface.set_lock(true);
        surface.set_lock(true);
        manager.ping_timeout(compositor, surface, shell_surface);
        shell_surface.set_lock(false);
        surface.set_lock(false);
    };
    new_popup_listener => new_popup_notify: |this: &mut XdgV6Shell, popup: *mut libc::c_void,|
    unsafe {
        let (ref mut shell_surface, ref mut surface, ref mut manager) = this.data;
        let compositor = &mut *COMPOSITOR_PTR;
        let mut popup = XdgV6Popup::from_ptr(popup as _);
        shell_surface.set_lock(true);
        surface.set_lock(true);
        manager.new_popup(compositor, surface, shell_surface, &mut popup);
        shell_surface.set_lock(false);
        surface.set_lock(false);
    };
    maximize_listener => maximize_notify: |this: &mut XdgV6Shell, _event: *mut libc::c_void,|
    unsafe {
        let (ref mut shell_surface, ref mut surface, ref mut manager) = this.data;
        let compositor = &mut *COMPOSITOR_PTR;
        shell_surface.set_lock(true);
        surface.set_lock(true);
        manager.maximize_request(compositor, surface, shell_surface);
        shell_surface.set_lock(false);
        surface.set_lock(false);
    };
    fullscreen_listener => fullscreen_notify: |this: &mut XdgV6Shell, event: *mut libc::c_void,|
    unsafe {
        let (ref mut shell_surface, ref mut surface, ref mut manager) = this.data;
        let compositor = &mut *COMPOSITOR_PTR;
        let mut event = SetFullscreenEvent::from_ptr(event as _);
        shell_surface.set_lock(true);
        surface.set_lock(true);
        manager.fullscreen_request(compositor, surface, shell_surface, &mut event);
        shell_surface.set_lock(false);
        surface.set_lock(false);
    };
    minimize_listener => minimize_notify: |this: &mut XdgV6Shell, _event: *mut libc::c_void,|
    unsafe {
        let (ref mut shell_surface, ref mut surface, ref mut manager) = this.data;
        let compositor = &mut *COMPOSITOR_PTR;
        shell_surface.set_lock(true);
        surface.set_lock(true);
        manager.minimize_request(compositor, surface, shell_surface);
        shell_surface.set_lock(false);
        surface.set_lock(false);
    };
    move_listener => move_notify: |this: &mut XdgV6Shell, event: *mut libc::c_void,| unsafe {
        let (ref mut shell_surface, ref mut surface, ref mut manager) = this.data;
        let compositor = &mut *COMPOSITOR_PTR;
        let mut event = MoveEvent::from_ptr(event as _);
        shell_surface.set_lock(true);
        surface.set_lock(true);
        manager.move_request(compositor, surface, shell_surface, &mut event);
        shell_surface.set_lock(false);
        surface.set_lock(false);
    };
    resize_listener => resize_notify: |this: &mut XdgV6Shell, event: *mut libc::c_void,| unsafe {
        let (ref mut shell_surface, ref mut surface, ref mut manager) = this.data;
        let compositor = &mut *COMPOSITOR_PTR;
        let mut event = ResizeEvent::from_ptr(event as _);
        shell_surface.set_lock(true);
        surface.set_lock(true);
        manager.resize_request(compositor, surface, shell_surface, &mut event);
        shell_surface.set_lock(false);
        surface.set_lock(false);
    };
    show_window_menu_listener => show_window_menu_notify: |this: &mut XdgV6Shell,
                                                           event: *mut libc::c_void,| unsafe {
        let (ref mut shell_surface, ref mut surface, ref mut manager) = this.data;
        let compositor = &mut *COMPOSITOR_PTR;
        let mut event = ShowWindowMenuEvent::from_ptr(event as _);
        shell_surface.set_lock(true);
        surface.set_lock(true);
        manager.show_window_menu_request(compositor, surface, shell_surface, &mut event);
        shell_surface.set_lock(false);
        surface.set_lock(false);
    };
]);

impl XdgV6Shell {
    pub(crate) unsafe fn surface_ptr(&self) -> *mut wlr_xdg_surface_v6 {
        self.data.0.as_ptr()
    }

    pub(crate) fn surface_mut(&mut self) -> &mut XdgV6ShellSurface {
        &mut self.data.0
    }
}
