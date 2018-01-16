//! Wrapper for the `wlr_box` type.
//! Note that we renamed it to `Area` to avoid conflicts with Rust's Box.

use std::ops::{Deref, DerefMut};

use libc::{c_double, c_int};

use wlroots_sys::{wl_output_transform, wlr_box, wlr_box_closest_point, wlr_box_contains_point,
                  wlr_box_empty, wlr_box_intersection, wlr_box_transform};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
/// Result of applying an intersection of two `Area`s.
pub enum IntersectionResult {
    /// This area is the intersection between the two points.
    Intersection(Area),
    /// There was not an intersection, here is the resulting area anyways.
    NoIntersection(Area)
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Origin {
    pub x: c_int,
    pub y: c_int
}

impl Default for Origin {
    fn default() -> Self {
        Origin { x: 0, y: 0 }
    }
}

impl Into<Area> for Origin {
    fn into(self) -> Area {
        Area::new(self, Size::default())
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Size {
    pub width: c_int,
    pub height: c_int
}

impl Default for Size {
    fn default() -> Self {
        Size { width: 0,
               height: 0 }
    }
}

impl Into<Area> for Size {
    fn into(self) -> Area {
        Area::new(Origin::default(), self)
    }
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
    pub fn new(origin: Origin, size: Size) -> Self {
        Area(wlr_box { x: origin.x,
                       y: origin.y,
                       width: size.width,
                       height: size.height })
    }

    /// Finds the closest point within the box to the given point.
    /// If the (x, y) point lies outside of the box, then it finds the closest
    /// corner and returns that.
    ///
    /// Returned value is in form of (x, y).
    pub fn closest_point(&mut self, x: c_double, y: c_double) -> (c_double, c_double) {
        unsafe {
            let (mut dest_x, mut dest_y) = (0.0, 0.0);
            wlr_box_closest_point(&mut self.0, x, y, &mut dest_x, &mut dest_y);
            (dest_x, dest_y)
        }
    }

    /// Gets the intersection of the two areas.
    pub fn intersection(&self, other_box: &Area) -> IntersectionResult {
        unsafe {
            let mut res = Area::default();
            let is_empty =
                wlr_box_intersection(&self.0, &other_box.0, &mut res.0 as *mut _);
            if is_empty {
                IntersectionResult::NoIntersection(res)
            } else {
                IntersectionResult::Intersection(res)
            }
        }
    }

    /// Determines if the box contains the given point.
    pub fn contains_point(&mut self, x: c_double, y: c_double) -> bool {
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

impl PartialEq for Area {
    fn eq(&self, other: &Area) -> bool {
        self.x == other.x && self.y == other.y && self.height == other.height
        && self.width == other.width
    }
}

impl Eq for Area {}
