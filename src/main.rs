mod modules;
mod config;
mod util;

use clap::Command;
use sqlx::SqlitePool;
use std::env;
use std::fs::create_dir_all;
use crate::modules::{heatmap, timelapse, usermap, singleplace, singleplayer};

fn cli() -> Command {
    Command::new("canvas")
        .about("canvas tools !!")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(Command::new("timelapse").about("Render a timelapse video of the canvas"))
        .subcommand(Command::new("heatmap").about("Render a heatmap of the canvas"))
        .subcommand(Command::new("usermap").about("Render a usermap of the canvas, showing who placed each pixel"))
        .subcommand(Command::new("singleplace").about("Render the canvas, without placing pixels over drawn pixels"))
        .subcommand(Command::new("singleplayer").about("Render one canvas per user, showing only the pixels they placed."))
}

#[tokio::main]
async fn main() {
    let matches = cli().get_matches();
    dotenvy::dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = SqlitePool::connect(&database_url).await.unwrap();
    create_dir_all("./output").expect("Failed to create output directory");

    match matches.subcommand() {
        Some(("timelapse", _sub_matches)) => {
            timelapse::timelapse(pool).await;
        },
        Some(("heatmap", _sub_matches)) => {
            heatmap::heatmap(pool).await;
        },
        Some(("usermap", _sub_matches)) => {
            usermap::usermap(pool).await;
        },
        Some(("singleplace", _sub_matches)) => {
            singleplace::singleplace(pool).await;
        },
        Some(("singleplayer", _sub_matches)) => {
            singleplayer::singleplayer(pool).await;
        }
        _ => unreachable!(),
    }
}
