use crate::config::CANVAS_SIZES;
use crate::util::color::hex_to_rgba;
use hsv::hsv_to_rgb;
use image::{ImageBuffer, Rgba};
use sqlx::{query, Pool, Sqlite};
use std::collections::HashMap;

const FINAL_CANVAS_SIZE: (u32, u32) = CANVAS_SIZES[CANVAS_SIZES.len() - 1];

fn blank_image() -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let mut image = image::ImageBuffer::new(FINAL_CANVAS_SIZE.0, FINAL_CANVAS_SIZE.1);
    for x in 0..image.width() {
        for y in 0..image.height() {
            image.put_pixel(x, y, Rgba([255, 255, 255, 255]));
        }
    }

    image
}

pub async fn usermap(pool: Pool<Sqlite>) {
    let placements = query!(
        "SELECT p.x, p.y, du.discord_id, p.mod_action 
         FROM pixel p 
         JOIN discord_user du ON p.user_id = du.user_id 
         WHERE p.created_at > '2025-02-28 17:00:00'"
    )
    .fetch_all(&pool)
    .await
    .unwrap();

    let mut image = blank_image();
    let mut user_colors = match std::fs::read_to_string("db/user_colors.json") {
        Ok(json) => serde_json::from_str::<HashMap<u64, String>>(&json).unwrap(),
        Err(_) => HashMap::new(),
    };

    for pixel in placements {
        let x = pixel.x as u32;
        let y = pixel.y as u32;
        let discord_id: u64 = pixel.discord_id.parse().unwrap();

        if pixel.mod_action == 0 {
            let color = match user_colors.get(&discord_id) {
                Some(color) => hex_to_rgba(color),
                None => {
                    let h = discord_id as f64 % 360.0;
                    let s = ((discord_id as f64 % 40.0) + 60.0) / 100.0;
                    let v = ((discord_id as f64 % 50.0) + 50.0) / 100.0;
                    let (r, g, b) = hsv_to_rgb(h, s, v);
                    let color = Rgba([r, g, b, 255]);
                    let hex = format!("#{:02x}{:02x}{:02x}", r, g, b);
                    user_colors.insert(discord_id, hex);
                    color
                }
            };
            image.put_pixel(x, y, color);
        } else {
            image.put_pixel(x, y, Rgba([255, 255, 255, 255]));
        }
    }

    let json = serde_json::to_string(&user_colors).unwrap();
    std::fs::write("db/user_colors.json", json).unwrap();
    image.save("output/usermap.png").unwrap();
}
