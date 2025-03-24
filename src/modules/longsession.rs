use crate::util::db::get_user_map;
use sqlx::{query, Pool, Sqlite};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufWriter;
use std::io::Write;

const MIN_PIXELS: i32 = 50;

fn print_write(writer: &mut BufWriter<File>, text: &str) {
    println!("{text}");
    writeln!(writer, "{}", text).unwrap();
}

pub async fn longsession(pool: Pool<Sqlite>, seconds: &i32) {
    let placements = query!(
        "SELECT du.discord_id as user_id, strftime('%s', p.created_at) as created_at, mod_action
        FROM pixel as p
        JOIN discord_user du ON p.user_id = du.user_id
        WHERE p.created_at > '2025-02-28 17:00:00'"
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    let users: HashMap<u64, String> = get_user_map(pool).await;
    let mut active_sessions: HashMap<u64, (i32, i32, i32)> = HashMap::new();
    let mut sessions: Vec<(u64, i32, i32)> = vec![];

    for pixel in placements {
        if pixel.mod_action == 1 {
            continue;
        }
        let timestamp: i32 = pixel.created_at.clone().unwrap().parse().unwrap();
        let user_id: u64 = pixel.user_id.parse().unwrap();
        let current_session = active_sessions
            .entry(user_id)
            .or_insert_with(|| (timestamp, timestamp, 0));

        let (session_start, session_last, pixels) = current_session;
        let pixel_duration = timestamp - *session_last;
        if pixel_duration > *seconds {
            if *pixels > MIN_PIXELS {
                sessions.push((user_id, *session_start, *pixels));
            }
            *current_session = (timestamp, timestamp, 1);
        } else {
            *current_session = (*session_start, timestamp, *pixels + 1);
        }
    }

    // Store remaining sessions
    for (user, session) in active_sessions.iter() {
        let (session_start, _, pixels) = *session;
        if pixels > MIN_PIXELS {
            sessions.push((*user, session_start, pixels));
        }
    }

    sessions.sort_by(|a, b| b.2.cmp(&a.2));
    let output = File::create("output/longsession.txt").unwrap();
    let mut writer = BufWriter::new(output);
    print_write(
        &mut writer,
        format!("Longest sessions, with no pauses longer than {seconds} seconds:").as_str(),
    );
    for (i, session) in sessions.iter().enumerate() {
        let (user_id, session_start, pixels) = session;
        let username = users.get(user_id).unwrap();
        let index = i + 1;
        print_write(
            &mut writer,
            format!("{index:02}. {username}: {pixels} pixels, started at <t:{session_start}>")
                .as_str(),
        );
    }
    writer.flush().unwrap();
}
