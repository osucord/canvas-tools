use sqlx::{query, Pool, Sqlite};
use std::collections::HashMap;

pub async fn get_user_map(pool: Pool<Sqlite>) -> HashMap<u64, String> {
    query!("SELECT discord_id, discord_username FROM discord_user")
        .fetch_all(&pool)
        .await
        .unwrap()
        .into_iter()
        .map(|u| (u.discord_id.parse().unwrap(), u.discord_username))
        .collect()
}
