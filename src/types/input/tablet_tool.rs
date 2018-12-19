//! TODO Documentation
use std::{cell::Cell, rc::Rc};

use errors::{HandleErr, HandleResult};
use wlroots_sys::{wlr_input_device, wlr_tablet, wlr_tablet_tool_axes};

use {input::{self, InputState}, utils::{self, Handleable}};
pub use manager::tablet_tool_handler::*;
pub use events::tablet_tool_events as event;

pub type Handle = utils::Handle<*mut wlr_input_device, wlr_tablet, TabletTool>;

#[derive(Debug)]
pub struct TabletTool {
    /// The structure that ensures weak handles to this structure are still alive.
    ///
    /// They contain weak handles, and will safely not use dead memory when this
    /// is freed by wlroots.
    ///
    /// If this is `None`, then this is from an upgraded `tablet_tool::Handle`, and
    /// the operations are **unchecked**.
    /// This is means safe operations might fail, but only if you use the unsafe
    /// marked function `upgrade` on a `tablet_tool::Handle`.
    liveliness: Rc<Cell<bool>>,
    /// The device that refers to this tablet tool.
    device: input::Device,
    /// Underlying tablet state
    tool: *mut wlr_tablet
}

bitflags! {
    pub struct Axis: u32 {
        const WLR_TABLET_TOOL_AXIS_X =
            wlr_tablet_tool_axes::WLR_TABLET_TOOL_AXIS_X as u32;
        const WLR_TABLET_TOOL_AXIS_Y =
            wlr_tablet_tool_axes::WLR_TABLET_TOOL_AXIS_Y as u32;
        const WLR_TABLET_TOOL_AXIS_DISTANCE =
            wlr_tablet_tool_axes::WLR_TABLET_TOOL_AXIS_DISTANCE as u32;
        const WLR_TABLET_TOOL_AXIS_PRESSURE =
            wlr_tablet_tool_axes::WLR_TABLET_TOOL_AXIS_PRESSURE as u32;
        const WLR_TABLET_TOOL_AXIS_TILT_X =
            wlr_tablet_tool_axes::WLR_TABLET_TOOL_AXIS_TILT_X as u32;
        const WLR_TABLET_TOOL_AXIS_TILT_Y =
            wlr_tablet_tool_axes::WLR_TABLET_TOOL_AXIS_TILT_Y as u32;
    }
}

impl TabletTool {
    /// Tries to convert an input device to a TabletTool
    ///
    /// Returns None if it is of a different type of input variant.
    ///
    /// # Safety
    /// This creates a totally new TabletTool (e.g with its own reference count)
    /// so only do this once per `wlr_input_device`!
    pub(crate) unsafe fn new_from_input_device(device: *mut wlr_input_device) -> Option<Self> {
        use wlroots_sys::wlr_input_device_type::*;
        match (*device).type_ {
            WLR_INPUT_DEVICE_TABLET_TOOL => {
                let tool = (*device).__bindgen_anon_1.tablet;
                let liveliness = Rc::new(Cell::new(false));
                let handle = Rc::downgrade(&liveliness);
                let state = Box::new(InputState { handle,
                                                  device: input::Device::from_ptr(device) });
                (*tool).data = Box::into_raw(state) as *mut _;
                Some(TabletTool { liveliness,
                                  device: input::Device::from_ptr(device),
                                  tool })
            }
            _ => None
        }
    }

    /// Gets the wlr_input_device associated with this TabletTool.
    pub fn input_device(&self) -> &input::Device {
        &self.device
    }
}

impl Drop for TabletTool {
    fn drop(&mut self) {
        if Rc::strong_count(&self.liveliness) != 1 {
            return
        }
        wlr_log!(WLR_DEBUG, "Dropped TabletTool {:p}", self.tool);
        unsafe {
            let _ = Box::from_raw((*self.tool).data as *mut InputState);
        }
        let weak_count = Rc::weak_count(&self.liveliness);
        if weak_count > 0 {
            wlr_log!(WLR_DEBUG,
                     "Still {} weak pointers to TabletTool {:p}",
                     weak_count,
                     self.tool);
        }
    }
}

impl Handleable<*mut wlr_input_device, wlr_tablet> for TabletTool {
    #[doc(hidden)]
    unsafe fn from_ptr(tool: *mut wlr_tablet) -> Self {
        let data = Box::from_raw((*tool).data as *mut InputState);
        let handle = data.handle.clone();
        let device = data.device.clone();
        (*tool).data = Box::into_raw(data) as *mut _;
        TabletTool { liveliness: handle.upgrade().unwrap(),
                     device,
                     tool }
    }

    #[doc(hidden)]
    unsafe fn as_ptr(&self) -> *mut wlr_tablet {
        self.tool
    }

    #[doc(hidden)]
    unsafe fn from_handle(handle: &Handle) -> HandleResult<Self> {
        let liveliness = handle.handle
            .upgrade()
            .ok_or(HandleErr::AlreadyDropped)?;
        Ok(TabletTool { liveliness,
                        // NOTE Rationale for cloning:
                        // If we already dropped we don't reach this point.
                        device: input::Device { device: handle.data },
                        tool: handle.as_ptr()
        })
    }

    fn weak_reference(&self) -> Handle {
        Handle { ptr: self.tool,
                 handle: Rc::downgrade(&self.liveliness),
                 // NOTE Rationale for cloning:
                 // Since we have a strong reference already,
                 // the input must still be alive.
                 data: unsafe { self.device.as_ptr() },
                 _marker: std::marker::PhantomData
        }
    }
}
