use crate::config::CANVAS_SIZES;
use crate::util::db::get_user_map;
use crate::util::io::print_write;
use sqlx::{query, Pool, Sqlite};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufWriter;
use std::io::Write;

pub async fn currentpixels(pool: Pool<Sqlite>) {
    let placements = query!(
        "SELECT p.x, p.y, du.discord_id as user_id, mod_action
        FROM pixel as p
        JOIN discord_user du ON p.user_id = du.user_id
        JOIN user u ON du.user_id = u.user_id
        WHERE p.created_at > '2025-02-28 17:00:00' AND u.is_banned = 0"
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    let users: HashMap<u64, String> = get_user_map(pool).await;

    const FINAL_CANVAS_SIZE: (u32, u32) = CANVAS_SIZES[CANVAS_SIZES.len() - 1];
    let mut user_grid = vec![vec![0; FINAL_CANVAS_SIZE.1 as usize]; FINAL_CANVAS_SIZE.0 as usize];
    let mut user_counts: HashMap<u64, i32> = HashMap::new();

    for pixel in placements {
        let user_id: u64 = pixel.user_id.parse().unwrap();
        let x = pixel.x as u32;
        let y = pixel.y as u32;

        user_grid[x as usize][y as usize] = if pixel.mod_action == 1 { 0 } else { user_id };
    }
    for row in user_grid {
        for user_id in row {
            if user_id != 0 {
                *user_counts.entry(user_id).or_insert(0) += 1;
            }
        }
    }

    let mut user_count_lb = user_counts.into_iter().collect::<Vec<(u64, i32)>>();
    user_count_lb.sort_by(|(_, a), (_, b)| b.cmp(a));
    let output = File::create("output/currentpixels.txt").unwrap();
    let mut writer = BufWriter::new(output);
    print_write(&mut writer, "Most pixels placed on current canvas:");
    for (i, entry) in user_count_lb.iter().enumerate() {
        let (user_id, pixels) = entry;
        let username = users.get(user_id).unwrap();
        let index = i + 1;
        print_write(
            &mut writer,
            format!("{index:02}. {username}: {pixels} pixels.").as_str(),
        );
    }
    writer.flush().unwrap();
}
