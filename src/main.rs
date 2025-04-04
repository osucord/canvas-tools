mod config;
mod modules;
mod util;

use crate::modules::{
    currentpixels, heatmap, longsession, singleplace, singleplayer, timelapse, usermap, virginmap, maincontributors, agemap
};

use clap::{Arg, Command};
use sqlx::SqlitePool;
use std::env;
use std::fs::create_dir_all;

fn cli() -> Command {
    Command::new("canvas")
        .about("canvas tools !!")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(Command::new("timelapse").about("Render a timelapse video of the canvas"))
        .subcommand(Command::new("virginmap").about("Render a timelapse video of the canvas"))
        .subcommand(
            Command::new("agemap")
                .about("Render a timelapse showing the age of each pixel"),
        )
        .subcommand(Command::new("heatmap").about("Render a heatmap of the canvas"))
        .subcommand(
            Command::new("usermap")
                .about("Render a usermap of the canvas, showing who placed each pixel"),
        )
        .subcommand(
            Command::new("singleplace")
                .about("Render the canvas, without placing pixels over drawn pixels"),
        )
        .subcommand(
            Command::new("singleplayer")
                .about("Render one canvas per user, showing only the pixels they placed."),
        )
        .subcommand(
            Command::new("longsession")
                .about("Show a list of the longest sessions, with a max pause of X seconds.")
                .arg(
                    Arg::new("seconds")
                        .short('s')
                        .long("seconds")
                        .help("Specify the amount of seconds")
                        .default_value("5")
                        .value_parser(clap::value_parser!(i32)),
                ),
        )
        .subcommand(
            Command::new("currentpixels")
                .about("Make a leaderboard counting only the pixels still on the canvas."),
        )
        .subcommand(
            Command::new("maincontributors")
                .about("List the amount of people that were placed most of X% of the pixels")
                .arg(
                    Arg::new("percentage")
                        .short('p')
                        .long("percentage")
                        .help("Specify the percentage")
                        .default_value("90")
                        .value_parser(clap::value_parser!(i32)),
                ),
        )
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
            // TODO: let configure on cmdline.
            let pixels_per_frame = 10;
            let min_seconds_between_frames = 10;
            let frames_per_second = 120;

            timelapse::timelapse(
                pool,
                frames_per_second,
                pixels_per_frame,
                min_seconds_between_frames,
            )
            .await;
        }
        Some(("virginmap", _sub_matches)) => {
            // TODO: let configure on cmdline.
            let pixels_per_frame = 10;
            let min_seconds_between_frames = 10;
            let frames_per_second = 120;

            virginmap::timelapse(
                pool,
                frames_per_second,
                pixels_per_frame,
                min_seconds_between_frames,
            )
            .await;
        }
        Some(("agemap", _sub_matches)) => {
            // TODO: let configure on cmdline.
            let pixels_per_frame = 20;
            let min_seconds_between_frames = 20;
            let frames_per_second = 120;

            agemap::agemap(
                pool,
                frames_per_second,
                pixels_per_frame,
                min_seconds_between_frames,
            )
            .await;
        }
        Some(("heatmap", _sub_matches)) => {
            heatmap::heatmap(pool).await;
        }
        Some(("usermap", _sub_matches)) => {
            usermap::usermap(pool).await;
        }
        Some(("singleplace", _sub_matches)) => {
            singleplace::singleplace(pool).await;
        }
        Some(("singleplayer", _sub_matches)) => {
            singleplayer::singleplayer(pool).await;
        }
        Some(("longsession", sub_matches)) => {
            longsession::longsession(pool, sub_matches.get_one::<i32>("seconds").unwrap()).await;
        }
        Some(("currentpixels", _sub_matches)) => {
            currentpixels::currentpixels(pool).await;
        }
        Some(("maincontributors", sub_matches)) => {
            maincontributors::maincontributors(pool, sub_matches.get_one::<i32>("percentage").unwrap()).await;
        }
        _ => unreachable!(),
    }
}
