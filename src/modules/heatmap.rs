use sqlx::{query, Pool, Sqlite};
use crate::config::CANVAS_SIZES;
use palette::{LinSrgb, Mix};
use palette::encoding::{Linear, Srgb};
use palette::rgb::Rgb;

const MAX_HEAT: i32 = 20;
const COLORS: [(f32, Rgb<Linear<Srgb>>); 4] = [
        (0.0, LinSrgb::new(0.0, 0.0, 0.0)), // Black
        (0.2, LinSrgb::new(0.5451, 0.0, 0.0)), // DarkRed
        (0.66, LinSrgb::new(1.0, 1.0, 0.0)), // Yellow
        (1.0, LinSrgb::new(1.0, 1.0, 1.0)), // White
    ];

fn convert_color(color: Rgb<Linear<Srgb>>) -> image::Rgb<u8> {
    image::Rgb([(color.red * 255.0) as u8, (color.green * 255.0) as u8, (color.blue * 255.0) as u8])
}

fn heatmap_color(value: f32) -> image::Rgb<u8> {

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

pub async fn heatmap(pool: Pool<Sqlite>) {
    let placements = query!("SELECT x, y FROM pixel WHERE created_at > '2025-02-28 17:00:00'")
        .fetch_all(&pool)
        .await
        .unwrap();

    const FINAL_CANVAS_SIZE: (u32, u32) = CANVAS_SIZES[CANVAS_SIZES.len() - 1];
    let mut image = image::ImageBuffer::new(FINAL_CANVAS_SIZE.0, FINAL_CANVAS_SIZE.1);
    let mut heat_matrix = vec![vec![0; FINAL_CANVAS_SIZE.1 as usize]; FINAL_CANVAS_SIZE.0 as usize];

    for pixel in placements {
        let x = pixel.x as u32;
        let y = pixel.y as u32;
        heat_matrix[x as usize][y as usize] += 1;
    }

    let hottest = heat_matrix.iter().map(|row| row.iter().max().unwrap()).max().unwrap();
    let hottest_capped = if hottest > &MAX_HEAT { MAX_HEAT } else { *hottest };

    for x in 0..FINAL_CANVAS_SIZE.0 {
        for y in 0..FINAL_CANVAS_SIZE.1 {
            let heat = heat_matrix[x as usize][y as usize] as f32 / hottest_capped as f32;
            image.put_pixel(x, y, heatmap_color(heat));
        }
    }
    
    image.save("output/heatmap.png").unwrap();
}
