use crate::config::CANVAS_SIZES;
use crate::util::canvas::{blank_image};
use crate::util::color::hex_to_rgba;
use image::Rgba;
use sqlx::{query, Pool, Sqlite};
use std::collections::HashMap;
use std::fs::create_dir_all;
use crate::util::db::get_user_map;

pub async fn singleplayer(pool: Pool<Sqlite>) {
    let placements = query!(
        "SELECT p.x, p.y, p.color, du.discord_id, p.mod_action 
         FROM pixel p 
         JOIN discord_user du ON p.user_id = du.user_id 
         WHERE p.created_at > '2025-02-28 17:00:00'"
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    let users: HashMap<u64, String> = get_user_map(pool).await;

    const FINAL_CANVAS_SIZE: (u32, u32) = CANVAS_SIZES[CANVAS_SIZES.len() - 1];
    let mut grid = vec![
        vec![(0, Rgba([0, 0, 0, 0])); FINAL_CANVAS_SIZE.1 as usize];
        FINAL_CANVAS_SIZE.0 as usize
    ];

    for pixel in placements {
        let x = pixel.x as u32;
        let y = pixel.y as u32;
        let color = hex_to_rgba(&pixel.color);
        let discord_id: u64 = pixel.discord_id.parse().unwrap();

        let placement = if pixel.mod_action == 0 {
            (discord_id, color)
        } else {
            (0, Rgba([255, 255, 255, 255]))
        };
        grid[x as usize][y as usize] = placement;
    }

    let mut user_images: HashMap<u64, image::ImageBuffer<Rgba<u8>, Vec<u8>>> = HashMap::new();
    for (x, row) in grid.iter().enumerate() {
        for (y, (user_id, color)) in row.iter().enumerate() {
            if *user_id == 0 {
                continue;
            }

            let user_image = user_images.entry(*user_id).or_insert_with(blank_image);
            user_image.put_pixel(x as u32, y as u32, *color);
        }
    }   
    
    create_dir_all("./output/singleplayer/").expect("Failed to create output directory");
    for image in user_images {
        let username = users.get(&image.0).unwrap();
        image.1.save(format!("output/singleplayer/{username}.png")).unwrap();
    }
}
