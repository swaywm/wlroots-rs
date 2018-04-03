//! TODO Documentation

use Area;
use wlroots_sys::{wl_output_transform, wlr_matrix_identity, wlr_matrix_multiply,
                  wlr_matrix_project_box, wlr_matrix_projection, wlr_matrix_rotate,
                  wlr_matrix_scale, wlr_matrix_transform, wlr_matrix_translate,
                  wlr_matrix_transpose};

/// Modifies the matrix to become the identity matrix.
pub fn matrix_identity(output: &mut [f32; 9]) {
    unsafe { wlr_matrix_identity(output.as_mut_ptr()) }
}

/// Translate the matrix in x, and y.
pub fn matrix_translate(x: f32, y: f32) -> [f32; 9] {
    let mut output = [0.0; 9];
    unsafe {
        wlr_matrix_translate(output.as_mut_ptr(), x, y);
    }
    output
}

/// Scale the output in the x, and y.
pub fn matrix_scale(x: f32, y: f32) -> [f32; 9] {
    let mut output = [0.0; 9];
    unsafe {
        wlr_matrix_scale(output.as_mut_ptr(), x, y);
    }
    output
}

/// Rotate the matrix by some amount of radians.
pub fn matrix_rotate(mut matrix: [f32; 9], radians: f32) -> [f32; 9] {
    unsafe {
        wlr_matrix_rotate(matrix.as_mut_ptr(), radians);
    }
    matrix
}

/// Multiply two matrices together.
pub fn matrix_multiply(x: [f32; 9], y: [f32; 9]) -> [f32; 9] {
    let mut output = [0.0; 9];
    unsafe {
        wlr_matrix_multiply(output.as_mut_ptr(), x.as_ptr(), y.as_ptr());
    }
    output
}

/// Transform the matrix based on the given Wayland output transform mode.
pub fn matrix_transform(mut matrix: [f32; 9], transform: wl_output_transform) -> [f32; 9] {
    unsafe {
        wlr_matrix_transform(matrix.as_mut_ptr(), transform);
    }
    matrix
}

/// Transform the matrix based on the given Wayland output transform mode and
/// the width and height of a texture.
pub fn matrix_projection(mut matrix: [f32; 9],
                         width: i32,
                         height: i32,
                         transform: wl_output_transform)
                         -> [f32; 9] {
    unsafe {
        wlr_matrix_projection(matrix.as_mut_ptr(), width, height, transform);
    }
    matrix
}

pub fn matrix_transpose(matrix: [f32; 9]) -> [f32; 9] {
    let mut result = [0.0; 9];
    unsafe {
        wlr_matrix_transpose(result.as_mut_ptr(), matrix.as_ptr());
    }
    result
}

pub fn project_box(area: Area,
                   transform: wl_output_transform,
                   rotation: f32,
                   projection: [f32; 9])
                   -> [f32; 9] {
    unsafe {
        let mut output = [0.0; 9];
        wlr_matrix_project_box(output.as_mut_ptr(),
                               &area.into(),
                               transform,
                               rotation,
                               projection.as_ptr());
        output
    }
}
