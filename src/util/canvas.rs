use image::{ImageBuffer, Rgba};
use crate::config::CANVAS_SIZES;

const FINAL_CANVAS_SIZE: (u32, u32) = CANVAS_SIZES[CANVAS_SIZES.len() - 1];

pub fn white_image() -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    single_color_image(Rgba([255, 255, 255, 255]))
}

pub fn blank_image() -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    single_color_image(Rgba([0, 0, 0, 0]))
}

fn single_color_image(color: Rgba<u8>) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let mut image = image::ImageBuffer::new(FINAL_CANVAS_SIZE.0, FINAL_CANVAS_SIZE.1);
    for x in 0..image.width() {
        for y in 0..image.height() {
            image.put_pixel(x, y, color);
        }
    }

    image
}