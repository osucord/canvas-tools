use image::{ImageBuffer, Rgba};
use crate::config::CANVAS_SIZES;

const FINAL_CANVAS_SIZE: (u32, u32) = CANVAS_SIZES[CANVAS_SIZES.len() - 1];

pub fn blank_image() -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let mut image = image::ImageBuffer::new(FINAL_CANVAS_SIZE.0, FINAL_CANVAS_SIZE.1);
    for x in 0..image.width() {
        for y in 0..image.height() {
            image.put_pixel(x, y, Rgba([255, 255, 255, 255]));
        }
    }

    image
}