use image::{ImageBuffer, Rgba};
use sqlx::{query, query_scalar, SqlitePool};
use std::env;
use std::process::Command;

const PIXELS_PER_FRAME: i32 = 20;
const MIN_SECONDS_BETWEEN_FRAMES: i32 = 20;
const FRAMES_PER_SECOND: i32 = 60;
const CANVAS_SIZES: [(u32, u32); 3] = [(500, 281), (500, 540), (960, 540)];
const IMAGE_SIZE: (u32, u32) = (960, 540);

fn hex_to_rgba(hex: &str) -> Rgba<u8> {
    let r = u8::from_str_radix(&hex[1..3], 16).unwrap();
    let g = u8::from_str_radix(&hex[3..5], 16).unwrap();
    let b = u8::from_str_radix(&hex[5..7], 16).unwrap();
    Rgba([r, g, b, 255])
}

fn pixel_offset(canvas_size_idx: usize) -> (u32, u32) {
    (
        (IMAGE_SIZE.0 - CANVAS_SIZES[canvas_size_idx].0) / 2,
        (IMAGE_SIZE.1 - CANVAS_SIZES[canvas_size_idx].1) / 2,
    )
}

fn blank_image(canvas_size_idx: usize) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
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
    let mut new_image = blank_image(canvas_size_idx);

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

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = SqlitePool::connect(&database_url).await.unwrap();

    let placements =
        query!("SELECT x, y, color, strftime('%s', created_at) as created_at FROM pixel WHERE created_at > '2025-02-28 17:00:00'")
            .fetch_all(&pool)
            .await
            .unwrap();
    let pixel_count =
        query_scalar!("SELECT COUNT(*) FROM pixel WHERE created_at > '2025-02-28 17:00:00'")
            .fetch_one(&pool)
            .await
            .unwrap();

    let mut canvas_size_idx = 0;
    let mut placement_offset = pixel_offset(canvas_size_idx);
    let mut image = blank_image(canvas_size_idx);

    let mut frame_start_time = 0;
    let mut remaining_pixels = 0;
    let mut frame = 0;
    for (i, pixel) in placements.iter().enumerate() {
        let x = pixel.x as u32;
        let y = pixel.y as u32;
        let timestamp: i32 = pixel.created_at.clone().unwrap().parse().unwrap();

        if remaining_pixels <= 0 && timestamp - frame_start_time > MIN_SECONDS_BETWEEN_FRAMES {
            image.save(format!("output/image-{frame:05}.png")).unwrap();
            remaining_pixels = PIXELS_PER_FRAME;
            frame += 1;
            frame_start_time = timestamp;
        }

        if i % 1000 == 0 {
            println!("\rRendering frames: {}%", i * 100 / pixel_count as usize);
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
        frame += 1;
        image.save(format!("output/image-{frame:05}.png")).unwrap();
    }

    println!("Rendering...");

    let output = Command::new("ffmpeg")
        .args([
            "-framerate",
            &FRAMES_PER_SECOND.to_string(),
            "-f",
            "image2",
            "-i",
            "./output/image-%05d.png",
            "-c:v",
            "libx264",
            "-pix_fmt",
            "yuv420p",
            "-preset",
            "veryslow",
            "-y",
            "-vf",
            "scale=1920:1080:flags=neighbor", // 960x540 * 2
            "-crf",
            "24",
            "-tune",
            "animation",
            "-keyint_min",
            "64",
            "./output/video.mp4",
        ])
        .status()
        .expect("Failed to execute ffmpeg command");

    eprintln!("{}", output);
    println!("Done!");
}
