use sqlx::{query, Pool, Sqlite};
use crate::config::CANVAS_SIZES;
use crate::util;
use crate::util::canvas::blank_image;

pub async fn singleplace(pool: Pool<Sqlite>) {
    let placements = query!("SELECT x, y, color FROM pixel WHERE created_at > '2025-02-28 17:00:00'")
        .fetch_all(&pool)
        .await
        .unwrap();

    const FINAL_CANVAS_SIZE: (u32, u32) = CANVAS_SIZES[CANVAS_SIZES.len() - 1];
    let mut image = blank_image();
    let mut placed = vec![vec![false; FINAL_CANVAS_SIZE.1 as usize]; FINAL_CANVAS_SIZE.0 as usize];

    for pixel in placements {
        let x = pixel.x as u32;
        let y = pixel.y as u32;
        if placed[x as usize][y as usize] {
            continue;
        }
        
        let color = util::color::hex_to_rgba(&pixel.color);
        image.put_pixel(x, y, color);
        placed[x as usize][y as usize] = true;
    }

    image.save("output/singleplace.png").unwrap();
}
