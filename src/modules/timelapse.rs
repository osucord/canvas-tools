use crate::config::CANVAS_SIZES;
use crate::util::color::hex_to_rgba;
use crate::util::render::{blank_image_borders, extend_canvas, pixel_offset, start_ffmpeg};
use image::{ImageBuffer, Rgba};
use sqlx::{query_as, Pool, Sqlite};
use std::io::Write;
use std::process::ChildStdin;

struct Placement {
    pub color: String,
    pub created_at: Option<String>,
    pub x: i64,
    pub y: i64,
}

pub async fn timelapse(
    pool: Pool<Sqlite>,
    fps: u8,
    pixels_per_frame: i32,
    min_seconds_per_frame: i32,
) {
    let placements = query_as!(Placement,
            "SELECT x, y, color, strftime('%s', created_at) as created_at FROM pixel WHERE created_at > '2025-02-28 17:00:00'"
        )
        .fetch_all(&pool)
        .await
        .unwrap();

    let mut image = blank_image_borders(0, false);

    let (child, mut stdin) = start_ffmpeg(fps, "timelapse").expect("failed to start ffmpeg");

    render_timelapse(
        &mut image,
        &placements,
        &mut stdin,
        pixels_per_frame,
        min_seconds_per_frame,
    )
    .await;

    drop(stdin);
    child.wait_with_output().expect("Failed to wait on child");

    println!("Done!");
}

async fn render_timelapse(
    image: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    placements: &[Placement],
    stdin: &mut ChildStdin,
    pixels_per_frame: i32,
    min_seconds_per_frame: i32,
) {
    let mut canvas_size_idx = 0;
    let mut placement_offset = pixel_offset(canvas_size_idx);
    let mut frame_start_time = 0;
    let mut remaining_pixels = 0_i32;

    for pixel in placements {
        let x = pixel.x as u32;
        let y = pixel.y as u32;
        let timestamp: i32 = pixel.created_at.as_ref().unwrap().parse().unwrap();

        if remaining_pixels <= 0 && timestamp - frame_start_time >= min_seconds_per_frame {
            let raw_frame = image.as_raw().clone();
            stdin.write_all(&raw_frame).expect("Failed to write frame");
            remaining_pixels = pixels_per_frame;
            frame_start_time = timestamp;
        }

        remaining_pixels -= 1;

        if x > CANVAS_SIZES[canvas_size_idx].0 || y > CANVAS_SIZES[canvas_size_idx].1 {
            canvas_size_idx += 1;
            *image = extend_canvas(image, canvas_size_idx);
            placement_offset = pixel_offset(canvas_size_idx);
        }

        image.put_pixel(
            x + placement_offset.0,
            y + placement_offset.1,
            hex_to_rgba(&pixel.color),
        );
    }

    if remaining_pixels < pixels_per_frame {
        let raw_frame = image.as_raw().clone();
        stdin
            .write_all(&raw_frame)
            .expect("Failed to write last frame");
    }
}
