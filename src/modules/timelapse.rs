use image::{ImageBuffer, Rgba};
use sqlx::{query, Pool, Sqlite};
use std::io::Write;
use std::process::{Command, Stdio};
use crate::config::CANVAS_SIZES;
use crate::util::color::hex_to_rgba;

const PIXELS_PER_FRAME: i32 = 2000;
const MIN_SECONDS_BETWEEN_FRAMES: i32 = 20;
const FRAMES_PER_SECOND: i32 = 60;
const IMAGE_SIZE: (u32, u32) = (960, 540);
const VIDEO_SCALE: u32 = 2;

fn pixel_offset(canvas_size_idx: usize) -> (u32, u32) {
    (
        (IMAGE_SIZE.0 - CANVAS_SIZES[canvas_size_idx].0) / 2,
        (IMAGE_SIZE.1 - CANVAS_SIZES[canvas_size_idx].1) / 2,
    )
}

fn blank_image_borders(canvas_size_idx: usize) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let mut image = image::ImageBuffer::new(IMAGE_SIZE.0, IMAGE_SIZE.1);
    let (width, height) = CANVAS_SIZES[canvas_size_idx];
    let (x_offset, y_offset) = pixel_offset(canvas_size_idx);
    for x in 0..image.width() {
        for y in 0..image.height() {
            if x < x_offset || y < y_offset || x > x_offset + width || y > y_offset + height {
                image.put_pixel(x, y, Rgba([0, 0, 0, 255]));
            } else {
                image.put_pixel(x, y, Rgba([255, 255, 255, 255]));
            }
        }
    }

    image
}

fn extend_canvas(
    image: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    canvas_size_idx: usize,
) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let old_offset = pixel_offset(canvas_size_idx - 1);
    let (old_width, old_height) = CANVAS_SIZES[canvas_size_idx - 1];
    let new_offset = pixel_offset(canvas_size_idx);
    let mut new_image = blank_image_borders(canvas_size_idx);

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

pub async fn timelapse(pool: Pool<Sqlite>) {
    let placements =
        query!("SELECT x, y, color, strftime('%s', created_at) as created_at FROM pixel WHERE created_at > '2025-02-28 17:00:00'")
            .fetch_all(&pool)
            .await
            .unwrap();

    let mut canvas_size_idx = 0;
    let mut placement_offset = pixel_offset(canvas_size_idx);
    let mut image = blank_image_borders(canvas_size_idx);

    let mut frame_start_time = 0;
    let mut remaining_pixels = 0;

    println!("Rendering video...");
    #[rustfmt::skip]
    let mut output = Command::new("ffmpeg")
        .args([
            "-framerate", &FRAMES_PER_SECOND.to_string(),
            "-f", "rawvideo",
            "-pix_fmt", "rgba",
            "-video_size", &format!("{}x{}", IMAGE_SIZE.0, IMAGE_SIZE.1),
            "-i", "pipe:0",
            "-c:v", "libx264",
            "-pix_fmt", "yuv420p",
            "-preset", "veryslow",
            "-y",
            "-vf", &format!("scale={}:{}:flags=neighbor", IMAGE_SIZE.0 * VIDEO_SCALE, IMAGE_SIZE.1 * VIDEO_SCALE),
            "-crf", "24",
            "-tune", "animation",
            "-keyint_min", "64",
            "./output/timelapse.mp4",
        ])
        .stdin(Stdio::piped())
        .spawn()
        .expect("Failed to execute ffmpeg command");
    let stdin = output.stdin.as_mut().expect("Failed to open stdin");

    for pixel in placements {
        let x = pixel.x as u32;
        let y = pixel.y as u32;
        let timestamp: i32 = pixel.created_at.clone().unwrap().parse().unwrap();

        if remaining_pixels <= 0 && timestamp - frame_start_time > MIN_SECONDS_BETWEEN_FRAMES {
            let raw_frame = image.as_raw().clone();
            stdin.write_all(&raw_frame).expect("Failed to write frame");
            remaining_pixels = PIXELS_PER_FRAME;
            frame_start_time = timestamp;
        }

        remaining_pixels -= 1;

        if x > CANVAS_SIZES[canvas_size_idx].0 || y > CANVAS_SIZES[canvas_size_idx].1 {
            canvas_size_idx += 1;
            image = extend_canvas(&image, canvas_size_idx);
            placement_offset = pixel_offset(canvas_size_idx);
        }

        image.put_pixel(
            x + placement_offset.0,
            y + placement_offset.1,
            hex_to_rgba(&pixel.color),
        );
    }

    if remaining_pixels != 0 {
        let raw_frame = image.as_raw().clone();
        stdin.write_all(&raw_frame).expect("Failed to write frame");
    }

    let _ = stdin;
    let _ = output.wait_with_output().expect("Failed to wait on child");

    println!("Done!");
}
