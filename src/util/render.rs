use std::process::{Child, ChildStdin, Command};

use aformat::aformat;
use image::{ImageBuffer, Rgba};
use to_arraystring::ToArrayString;

use crate::config::CANVAS_SIZES;

const IMAGE_SIZE: (u32, u32) = (960, 540);
const VIDEO_SCALE: u32 = 2;
const WHITE: Rgba<u8> = Rgba([255, 255, 255, 0]);
pub const BLACK: Rgba<u8> = Rgba([0, 0, 0, 255]);

pub fn start_ffmpeg(fps: u8) -> std::io::Result<(Child, ChildStdin)> {
    #[rustfmt::skip]
    let mut child = Command::new("ffmpeg")
        .args([
            "-framerate", &fps.to_arraystring(),
            "-f", "rawvideo",
            "-pix_fmt", "rgba",
            "-video_size", &aformat!("{}x{}", IMAGE_SIZE.0, IMAGE_SIZE.1),
            "-i", "pipe:0",
            "-c:v", "libx264",
            "-pix_fmt", "yuv420p",
            "-preset", "veryslow",
            "-y",
            "-vf", &aformat!("scale={}:{}:flags=neighbor", IMAGE_SIZE.0 * VIDEO_SCALE, IMAGE_SIZE.1 * VIDEO_SCALE),
            "-crf", "24",
            "-tune", "animation",
            "-keyint_min", "64",
            "./output/timelapse.mp4",
        ])
        .stdin(std::process::Stdio::piped())
        .spawn()?;

    let stdin = child.stdin.take().expect("Failed to open stdin");

    Ok((child, stdin))
}

pub fn pixel_offset(canvas_size_idx: usize) -> (u32, u32) {
    (
        (IMAGE_SIZE.0 - CANVAS_SIZES[canvas_size_idx].0) / 2,
        (IMAGE_SIZE.1 - CANVAS_SIZES[canvas_size_idx].1) / 2,
    )
}

pub fn blank_image_borders(canvas_size_idx: usize) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    blank_image_borders_with_colour(
        canvas_size_idx,
        Rgba([0, 0, 0, 255]),
        Rgba([255, 255, 255, 255]),
    )
}

pub fn blank_image_borders_with_colour(
    canvas_size_idx: usize,
    in_bounds: Rgba<u8>,
    out_of_bounds: Rgba<u8>,
) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let mut image = image::ImageBuffer::new(IMAGE_SIZE.0, IMAGE_SIZE.1);
    let (width, height) = CANVAS_SIZES[canvas_size_idx];
    let (x_offset, y_offset) = pixel_offset(canvas_size_idx);
    for x in 0..image.width() {
        for y in 0..image.height() {
            if x < x_offset || y < y_offset || x > x_offset + width || y > y_offset + height {
                image.put_pixel(x, y, out_of_bounds);
            } else {
                image.put_pixel(x, y, in_bounds);
            }
        }
    }

    image
}

pub fn extend_canvas(
    image: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    canvas_size_idx: usize,
) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    extend_canvas_with_colour(image, canvas_size_idx, WHITE, BLACK)
}

pub fn extend_canvas_with_colour(
    image: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    canvas_size_idx: usize,
    in_bounds: Rgba<u8>,
    out_of_bounds: Rgba<u8>,
) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let old_offset = pixel_offset(canvas_size_idx - 1);
    let (old_width, old_height) = CANVAS_SIZES[canvas_size_idx - 1];
    let new_offset = pixel_offset(canvas_size_idx);
    let mut new_image = blank_image_borders_with_colour(canvas_size_idx, in_bounds, out_of_bounds);

    for x in 0..old_width {
        for y in 0..old_height {
            let old_x = x + old_offset.0;
            let old_y = y + old_offset.1;
            new_image.put_pixel(
                x + new_offset.0,
                y + new_offset.1,
                *image.get_pixel(old_x, old_y),
            );
        }
    }

    new_image
}
