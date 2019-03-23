use crate::cursor::xcursor;

/// An image that can be attached to a `Cursor` or `OutputCursor`.
#[derive(Debug, Default, PartialEq)]
pub struct Image<'buffer> {
    pub pixels: &'buffer [u8],
    pub stride: i32,
    pub width: u32,
    pub height: u32,
    pub hotspot_x: i32,
    pub hotspot_y: i32,
    pub delay: u32,
    pub scale: f32
}

impl<'buffer> Image<'buffer> {
    pub fn new(
        pixels: &'buffer [u8],
        stride: i32,
        width: u32,
        height: u32,
        hotspot_x: i32,
        hotspot_y: i32,
        scale: f32,
        delay: u32
    ) -> Image<'buffer> {
        Image {
            pixels,
            stride,
            width,
            height,
            hotspot_x,
            hotspot_y,
            scale,
            delay
        }
    }
}

impl<'buffer> From<xcursor::Image<'buffer>> for Image<'buffer> {
    fn from(image: xcursor::Image<'buffer>) -> Self {
        let xcursor::Image {
            buffer,
            width,
            height,
            hotspot_x,
            hotspot_y,
            delay,
            ..
        } = image;
        Image {
            pixels: buffer,
            width,
            height,
            stride: width as i32 * 4,
            scale: 1.0,
            hotspot_x: hotspot_x as _,
            hotspot_y: hotspot_y as _,
            delay
        }
    }
}
