use std::{ptr, time::Duration};
use wlroots_sys::{timespec, wlr_output, wlr_output_damage, wlr_output_damage_add,
                  wlr_output_damage_add_box, wlr_output_damage_add_whole,
                  wlr_output_damage_create, wlr_output_damage_destroy,
                  wlr_output_damage_make_current, wlr_output_damage_swap_buffers,
                  pixman_region32_t};

use Area;

#[derive(Debug)]
/// Tracks damage for an output.
///
/// When a `frame` event is emitted, `make_current` should be
/// called. If necessary, the output should be repainted and
/// `swap_buffers` should be called.
///
/// No rendering should happen outside a `frame` event handler.
pub struct OutputDamage {
    damage: *mut wlr_output_damage
}

impl OutputDamage {
    /// Makes a new `OutputDamage` bound to the given Output.
    ///
    /// # Safety
    /// This function is unsafe because the `OutputDamage` should not outlive the
    /// past in `Output`.
    pub(crate) unsafe fn new(output: *mut wlr_output) -> Self {
        unsafe {
            let damage = wlr_output_damage_create(output);
            if damage.is_null() {
                panic!("Damage was none")
            }
            OutputDamage { damage }
        }
    }

    pub(crate) unsafe fn from_ptr(damage: *mut wlr_output_damage) -> Self {
        OutputDamage { damage }
    }

    pub(crate) unsafe fn as_ptr(&self) -> *mut wlr_output_damage {
        self.damage
    }

    /// Just like `std::clone::Clone` but unsafe.
    ///
    /// # Unsafety
    /// You can create a reference leak using this method very easily,
    /// and it could make it so that weak handles to an output are dangling.
    ///
    /// This exists due to an issue in output_manager.rs that might be fixed
    /// with NLL, so if this is no longer necessary it should be removed asap.
    pub(crate) unsafe fn clone(&self) -> Self {
        OutputDamage { damage: self.damage }
    }

    /// Makes the output rendering context current.
    /// Returns `true` if `wlr_output_damage_swap_buffers` needs to be called.
    ///
    ///The region of the output that needs to be repainted is added to `damage`.
    pub fn make_current(&mut self, damage: &mut Option<*mut pixman_region32_t>) -> bool {
        unsafe {
            let mut res = false;
            let damage = damage.unwrap_or_else(|| ptr::null_mut());
            wlr_output_damage_make_current(self.damage, &mut res, damage);
            res
        }
    }

    /// Swaps the output buffers.
    ///
    /// If the time of the frame isn't known, set `when` to `None`.
    ///
    /// Swapping buffers schedules a `frame` event.
    pub fn swap_buffers(&mut self,
                        when: Option<Duration>,
                        damage: Option<*mut pixman_region32_t>)
                        -> bool {
        unsafe {
            let when = when.map(|duration| {
                                    timespec { tv_sec: duration.as_secs() as i64,
                                               tv_nsec: duration.subsec_nanos() as i64 }
                                });
            let when_ptr =
                when.map(|mut duration| &mut duration as *mut _).unwrap_or_else(|| ptr::null_mut());
            let damage = damage.unwrap_or_else(|| ptr::null_mut());
            wlr_output_damage_swap_buffers(self.damage, when_ptr, damage)
        }
    }

    /// Accumulates damage and schedules a `frame` event.
    pub fn add(&mut self, damage: *mut pixman_region32_t) {
        unsafe {
            wlr_output_damage_add(self.damage, damage);
        }
    }

    /// Damages the whole output and schedules a `frame` event.
    pub fn add_whole(&mut self) {
        unsafe { wlr_output_damage_add_whole(self.damage) }
    }

    /// Accumulates damage from an `Area` and schedules a `frame` event.
    pub fn add_area(&mut self, mut area: Area) {
        unsafe { wlr_output_damage_add_box(self.damage, &mut area.0) }
    }
}

impl Drop for OutputDamage {
    fn drop(&mut self) {
        wlr_log!(L_DEBUG, "Dropped OutputDamage {:p}", self.damage);
        unsafe {
            wlr_output_damage_destroy(self.damage);
        }
    }
}
