use crate::util::io::print_write;
use sqlx::{query, query_scalar, Pool, Sqlite};
use std::fs::File;
use std::io::BufWriter;
use std::io::Write;

pub async fn maincontributors(pool: Pool<Sqlite>, percentage: &i32) {
    let user_counts = query!(
        "SELECT COUNT(*) as count, du.discord_username as username
        FROM pixel as p
        JOIN discord_user du ON p.user_id = du.user_id
        WHERE p.mod_action = 0 AND p.created_at > '2025-02-28 17:00:00'
        GROUP BY du.user_id
        ORDER BY COUNT(*) DESC"
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    let pixel_count = query_scalar!("SELECT COUNT(*) FROM pixel WHERE mod_action = 0 AND created_at > '2025-02-28 17:00:00'").fetch_one(&pool).await.unwrap();
    let mut pixel_cap = (pixel_count as f32 * (*percentage as f32 / 100.0)) as i64;

    let output = File::create("output/maincontributors.txt").unwrap();
    let mut writer = BufWriter::new(output);
    print_write(&mut writer, format!("Users who contributed to {percentage}% of the pixels:").as_str());
    for (i, user) in user_counts.iter().enumerate() {
        let pixels = &user.count;
        let username = &user.username;
        pixel_cap -= user.count;
        if pixel_cap <= 0 {
            break;
        }
        let index = i + 1;
        print_write(
            &mut writer,
            format!("{index:02}. {username}: {pixels} pixels.").as_str(),
        );
    }

    writer.flush().unwrap();
}
