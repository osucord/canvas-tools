use crate::config::CANVAS_SIZES;
use crate::util::render::{blank_image_borders, pixel_offset, start_ffmpeg};
use image::Rgba;
use palette::encoding::{Linear, Srgb};
use palette::rgb::Rgb;
use palette::{LinSrgb, Mix};
use sqlx::{query_as, Pool, Sqlite};
use std::io::Write;
use std::process::ChildStdin;

struct Placement {
    pub created_at: Option<String>,
    pub x: i64,
    pub y: i64,
}

const COLORS: [(f32, Rgb<Linear<Srgb>>); 4] = [
    (0.0, LinSrgb::new(0.0, 0.0, 0.0)),    // Black
    (0.2, LinSrgb::new(0.5451, 0.0, 0.0)), // DarkRed
    (0.66, LinSrgb::new(1.0, 1.0, 0.0)),   // Yellow
    (1.0, LinSrgb::new(1.0, 1.0, 1.0)),    // White
];

fn convert_color(color: Rgb<Linear<Srgb>>) -> Rgba<u8> {
    Rgba([
        (color.red * 255.0) as u8,
        (color.green * 255.0) as u8,
        (color.blue * 255.0) as u8,
        255,
    ])
}

fn heatmap_color(value: f32) -> Rgba<u8> {
    for i in 0..COLORS.len() - 1 {
        let (t1, c1) = COLORS[i];
        let (t2, c2) = COLORS[i + 1];

        if value >= t1 && value <= t2 {
            let t = (value - t1) / (t2 - t1);
            return convert_color(c1.mix(c2, t));
        }
    }

    convert_color(COLORS.last().unwrap().1)
}

pub async fn agemap(
    pool: Pool<Sqlite>,
    fps: u8,
    pixels_per_frame: i32,
    min_seconds_per_frame: i32,
) {
    let placements = query_as!(Placement,
            "SELECT x, y, strftime('%s', created_at) as created_at FROM pixel WHERE created_at > '2025-02-28 17:00:00'"
        )
        .fetch_all(&pool)
        .await
        .unwrap();

    let (child, mut stdin) = start_ffmpeg(fps, "agemap").expect("failed to start ffmpeg");

    render_timelapse(
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
    placements: &[Placement],
    stdin: &mut ChildStdin,
    pixels_per_frame: i32,
    min_seconds_per_frame: i32,
) {
    let mut canvas_size_idx = 0;
    let mut placement_offset = pixel_offset(canvas_size_idx);
    let mut frame_start_time = 0;
    let mut remaining_pixels: i32 = 0;
    let pixel_lifetime: i32 = 60;
    const FINAL_CANVAS_SIZE: (u32, u32) = CANVAS_SIZES[CANVAS_SIZES.len() - 1];
    let mut pixel_age = vec![vec![0; FINAL_CANVAS_SIZE.1 as usize]; FINAL_CANVAS_SIZE.0 as usize];

    for pixel in placements {
        let x = pixel.x as u32;
        let y = pixel.y as u32;
        let timestamp: i32 = pixel.created_at.as_ref().unwrap().parse().unwrap();

        if remaining_pixels <= 0 && timestamp - frame_start_time >= min_seconds_per_frame {
            pixel_age.iter_mut().for_each(|row| {
                row.iter_mut().for_each(|age| {
                    if *age > 0 {
                        *age -= 1;
                    }
                })
            });

            let mut image = blank_image_borders(canvas_size_idx, true);
            let (width, height) = CANVAS_SIZES[canvas_size_idx];
            for (x, col) in pixel_age.iter().enumerate() {
                if x > width as usize {
                    continue;
                }
                for (y, age) in col.iter().enumerate() {
                    if y > height as usize {
                        continue;
                    }
                    image.put_pixel(
                        x as u32 + placement_offset.0,
                        y as u32 + placement_offset.1,
                        heatmap_color(*age as f32 / pixel_lifetime as f32),
                    );
                }
            }

            let raw_frame = image.as_raw().clone();
            stdin.write_all(&raw_frame).expect("Failed to write frame");
            remaining_pixels = pixels_per_frame;
            frame_start_time = timestamp;
        }

        remaining_pixels -= 1;

        if x > CANVAS_SIZES[canvas_size_idx].0 || y > CANVAS_SIZES[canvas_size_idx].1 {
            canvas_size_idx += 1;
            placement_offset = pixel_offset(canvas_size_idx);
        }

        pixel_age[x as usize][y as usize] = pixel_lifetime;
    }

    if remaining_pixels < pixels_per_frame {
        pixel_age.iter_mut().for_each(|row| {
            row.iter_mut().for_each(|age| {
                if *age > 0 {
                    *age -= 1;
                }
            })
        });

        let mut image = blank_image_borders(canvas_size_idx, true);
        let (width, height) = CANVAS_SIZES[canvas_size_idx];
        for (x, col) in pixel_age.iter().enumerate() {
            if x > width as usize {
                continue;
            }
            for (y, age) in col.iter().enumerate() {
                if y > height as usize {
                    continue;
                }
                image.put_pixel(
                    x as u32 + placement_offset.0,
                    y as u32 + placement_offset.1,
                    heatmap_color(*age as f32 / pixel_lifetime as f32),
                );
            }
        }

        let raw_frame = image.as_raw().clone();
        stdin
            .write_all(&raw_frame)
            .expect("Failed to write last frame");
    }
}
