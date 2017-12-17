//! Wrapper for the `wlr_box` type.
//! Note that we renamed it to `Area` to avoid conflicts with Rust's Box.

use std::ops::{Deref, DerefMut};

use libc::c_int;

use wlroots_sys::{wl_output_transform, wlr_box, wlr_box_closest_point, wlr_box_contains_point,
                  wlr_box_empty, wlr_box_intersection, wlr_box_transform};

#[derive(Debug, Clone, Copy)]
/// Result of applying an intersection of two `Area`s.
pub enum IntersectionResult {
    /// This area is the intersection between the two points.
    Intersection(Area),
    /// There was not an intersection, here is the resulting area anyways.
    NoIntersection(Area)
}

#[derive(Debug, Clone, Copy)]
/// Generic geometry-like struct. Container an origin (x, y) point and bounds
/// (width, height).
pub struct Area(pub wlr_box);

impl Default for Area {
    fn default() -> Area {
        Area(wlr_box { x: 0,
                       y: 0,
                       width: 0,
                       height: 0 })
    }
}

impl Area {
    pub fn new(x: c_int, y: c_int, width: c_int, height: c_int) -> Self {
        Area(wlr_box { x,
                       y,
                       width,
                       height })
    }

    /// Finds the closest point within the box to the given point.
    /// If the (x, y) point lies outside of the box, then it finds the closest
    /// corner and returns that.
    ///
    /// Returned value is in form of (x, y).
    pub fn closest_point(&mut self, x: f64, y: f64) -> (f64, f64) {
        unsafe {
            let (mut dest_x, mut dest_y) = (0.0, 0.0);
            wlr_box_closest_point(&mut self.0, x, y, &mut dest_x, &mut dest_y);
            (dest_x, dest_y)
        }
    }

    /// Gets the intersection of the two areas.
    pub fn intersection(&mut self, other_box: &mut Area) -> IntersectionResult {
        unsafe {
            let mut res = Area::default();
            let is_empty =
                wlr_box_intersection(&mut self.0, &mut other_box.0, &mut (&mut res.0 as *mut _));
            if is_empty {
                IntersectionResult::NoIntersection(res)
            } else {
                IntersectionResult::Intersection(res)
            }
        }
    }

    /// Determines if the box contains the given point.
    pub fn contains_point(&mut self, x: f64, y: f64) -> bool {
        unsafe { wlr_box_contains_point(&mut self.0, x, y) }
    }

    /// Determines if the box is empty (e.g if the bounds give it an area of 0).
    pub fn is_empty(&mut self) -> bool {
        unsafe { wlr_box_empty(&mut self.0) }
    }

    /// Transforms the box coordinates and bounds according to the
    /// output transformation.
    ///
    /// e.g: If it's `WL_OUTPUT_TRANSFORM_90` then it will flip the Area 90° clockwise.
    pub fn transform(&mut self, transform: wl_output_transform) -> Area {
        unsafe {
            let mut res = Area::default();
            wlr_box_transform(&mut self.0, transform, &mut res.0);
            res
        }
    }
}

impl Deref for Area {
    type Target = wlr_box;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Area {
    fn deref_mut(&mut self) -> &mut wlr_box {
        &mut self.0
    }
}