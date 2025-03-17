mod modules;

use crate::modules::timelapse::timelapse;
use clap::Command;
use sqlx::SqlitePool;
use std::env;

fn cli() -> Command {
    Command::new("canvas")
        .about("canvas tools !!")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(Command::new("timelapse").about("Render a timelapse video of the canvas"))
}

#[tokio::main]
async fn main() {
    let matches = cli().get_matches();
    dotenvy::dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = SqlitePool::connect(&database_url).await.unwrap();

    match matches.subcommand() {
        Some(("timelapse", _sub_matches)) => {
            timelapse(pool).await;
        }
        _ => unreachable!(),
    }
}
