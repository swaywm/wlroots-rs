//! Matrix math is used to render things on a computer screen. This is common
//! throughout computer graphics, for examples and primers on using matrix
//! math to render things to screens please read an OpengGL tutorial.
//!
//! In wlroots we primarily use a 3x3 matrix of 32 bit floating point values to
//! represent a 2D screen. We also provide basic helper functions to assist in
//! transforming the matrices.

use wlroots_sys::{
    wl_output_transform, wlr_matrix_multiply, wlr_matrix_project_box, wlr_matrix_projection,
    wlr_matrix_rotate, wlr_matrix_scale, wlr_matrix_transform, wlr_matrix_translate, wlr_matrix_transpose
};

use area::Area;

pub const IDENTITY: [f32; 9] = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];

/// Shortcut for the various matrix operations involved in projecting the
/// specified wlr_box onto a given orthographic projection with a given
/// rotation. The result can be applied to each coordinate of the box to
/// get a new coordinate from [-1,1].
pub fn project_box(
    area: Area,
    transform: wl_output_transform,
    rotation: f32,
    projection: [f32; 9]
) -> [f32; 9] {
    unsafe {
        let mut output = [0.0; 9];
        wlr_matrix_project_box(
            output.as_mut_ptr(),
            &area.into(),
            transform,
            rotation,
            projection.as_ptr()
        );
        output
    }
}

/// Translate the 2D matrix to a magnitude of (x, y).
pub fn translate(x: f32, y: f32) -> [f32; 9] {
    let mut output = [0.0; 9];
    unsafe {
        wlr_matrix_translate(output.as_mut_ptr(), x, y);
    }
    output
}

/// Scales the 2D matrix to a magnitude of (x, y).
pub fn scale(x: f32, y: f32) -> [f32; 9] {
    let mut output = [0.0; 9];
    unsafe {
        wlr_matrix_scale(output.as_mut_ptr(), x, y);
    }
    output
}

/// Rotate the matrix by some amount of radians.
pub fn rotate(mut matrix: [f32; 9], radians: f32) -> [f32; 9] {
    unsafe {
        wlr_matrix_rotate(matrix.as_mut_ptr(), radians);
    }
    matrix
}

/// Multiply two matrices together.
pub fn multiply(x: [f32; 9], y: [f32; 9]) -> [f32; 9] {
    let mut output = [0.0; 9];
    unsafe {
        wlr_matrix_multiply(output.as_mut_ptr(), x.as_ptr(), y.as_ptr());
    }
    output
}

/// Transform the matrix based on the given Wayland output transform mode.
pub fn transform(mut matrix: [f32; 9], transform: wl_output_transform) -> [f32; 9] {
    unsafe {
        wlr_matrix_transform(matrix.as_mut_ptr(), transform);
    }
    matrix
}

/// Create a 2D orthographic projection matrix of (width, height) with a
/// specified `wl_output_transform`
pub fn projection(mut matrix: [f32; 9], width: i32, height: i32, transform: wl_output_transform) -> [f32; 9] {
    unsafe {
        wlr_matrix_projection(matrix.as_mut_ptr(), width, height, transform);
    }
    matrix
}

/// Flip the values over the diagonal of a matrix
pub fn transpose(matrix: [f32; 9]) -> [f32; 9] {
    let mut result = [0.0; 9];
    unsafe {
        wlr_matrix_transpose(result.as_mut_ptr(), matrix.as_ptr());
    }
    result
}
