//! Wrapper for the `wlr_box` type.
//! Note that we renamed it to `Area` to avoid conflicts with Rust's Box.

use libc::{c_double, c_float, c_int};

use wlroots_sys::{
    wl_output_transform, wlr_box, wlr_box_closest_point, wlr_box_contains_point, wlr_box_empty,
    wlr_box_intersection, wlr_box_rotated_bounds, wlr_box_transform
};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
/// Result of applying an intersection of two `Area`s.
pub enum IntersectionResult {
    /// This area is the intersection between the two points.
    Intersection(Area),
    /// There was not an intersection
    NoIntersection
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub struct Origin {
    pub x: c_int,
    pub y: c_int
}

impl Origin {
    pub fn new(x: c_int, y: c_int) -> Self {
        Origin { x, y }
    }
}

impl Into<Area> for Origin {
    fn into(self) -> Area {
        Area::new(self, Size::default())
    }
}

#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
pub struct Size {
    pub width: c_int,
    pub height: c_int
}

impl Size {
    pub fn new(width: c_int, height: c_int) -> Self {
        Size { width, height }
    }
}

impl Into<Area> for Size {
    fn into(self) -> Area {
        Area::new(Origin::default(), self)
    }
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
/// Generic geometry-like struct. Container an origin (x, y) point and bounds
/// (width, height).
pub struct Area {
    pub origin: Origin,
    pub size: Size
}

impl Into<wlr_box> for Area {
    fn into(self) -> wlr_box {
        wlr_box {
            x: self.origin.x,
            y: self.origin.y,
            width: self.size.width,
            height: self.size.height
        }
    }
}

impl Area {
    pub fn new(origin: Origin, size: Size) -> Self {
        Area { origin, size }
    }

    /// Construct an Area from a `wlr_box`.
    pub fn from_box(wlr_box: wlr_box) -> Self {
        Area {
            origin: Origin {
                x: wlr_box.x,
                y: wlr_box.y
            },
            size: Size {
                width: wlr_box.width,
                height: wlr_box.height
            }
        }
    }

    /// Makes a new `Area` with width and height set to the values in the given
    /// `Size`.
    pub fn with_size(self, size: Size) -> Self {
        Area { size, ..self }
    }

    /// Makes a new `Area` with x and y set to the value in the given `Origin`.
    pub fn with_origin(self, origin: Origin) -> Self {
        Area { origin, ..self }
    }

    /// Finds the closest point within the box to the given point.
    /// If the (x, y) point lies outside of the box, then it finds the closest
    /// corner and returns that.
    ///
    /// Returned value is in form of (x, y).
    pub fn closest_point(self, x: c_double, y: c_double) -> (c_double, c_double) {
        unsafe {
            let (mut dest_x, mut dest_y) = (0.0, 0.0);
            wlr_box_closest_point(&mut self.into(), x, y, &mut dest_x, &mut dest_y);
            (dest_x, dest_y)
        }
    }

    /// Gets the intersection of the two areas.
    pub fn intersection(self, other_box: Area) -> IntersectionResult {
        unsafe {
            let res = Area::default();
            let is_empty = wlr_box_intersection(&mut res.into(), &self.into(), &other_box.into());
            if is_empty {
                IntersectionResult::NoIntersection
            } else {
                IntersectionResult::Intersection(res)
            }
        }
    }

    /// Determines if the box contains the given point.
    pub fn contains_point(self, x: c_double, y: c_double) -> bool {
        unsafe { wlr_box_contains_point(&mut self.into(), x, y) }
    }

    /// Determines if the box is empty (e.g if the bounds give it an area of 0).
    pub fn is_empty(self) -> bool {
        unsafe { wlr_box_empty(&mut self.into()) }
    }

    /// Transforms the box coordinates and bounds according to the
    /// output transformation.
    ///
    /// e.g: If it's `WL_OUTPUT_TRANSFORM_90` then it will flip the Area 90°
    /// clockwise.
    pub fn transform(self, transform: wl_output_transform, width: c_int, height: c_int) -> Area {
        unsafe {
            let res = Area::default();
            wlr_box_transform(&mut res.into(), &mut self.into(), transform, width, height);
            res
        }
    }

    /// Creates the smallest box that contains the box rotated about its center.
    pub fn rotated_bounds(self, rotation: c_float) -> Area {
        unsafe {
            let dest = Area::default();
            wlr_box_rotated_bounds(&mut dest.into(), &self.into(), rotation);
            dest
        }
    }
}
