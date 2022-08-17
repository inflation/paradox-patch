use std::{collections::HashMap, io::Write};

use clap::{Parser, ValueEnum};
use clap_verbosity_flag::Verbosity;
use color_eyre::{eyre::eyre, Report, Result};
use futures::TryStreamExt;
use reqwest::{Client, StatusCode};
use serde::Deserialize;

static URL: &'static str = "https://store.steampowered.com/api/appdetails?appids=";

#[derive(Clone, Copy, ValueEnum)]
#[clap(rename_all = "lower")]
enum GameAbbr {
    Eu4,
    Hoi4,
    Stellaris,
    Ck3,
}

const fn game_id(game: GameAbbr) -> u32 {
    match game {
        GameAbbr::Eu4 => 236850,
        GameAbbr::Hoi4 => 394360,
        GameAbbr::Stellaris => 281990,
        GameAbbr::Ck3 => 1158310,
    }
}

#[derive(Debug, Deserialize)]
struct GameList {
    #[serde(flatten)]
    games: HashMap<String, Game>,
}

#[derive(Debug, Deserialize)]
struct Game {
    data: Data,
}

#[derive(Debug, Deserialize)]
struct Data {
    name: String,
    dlc: Option<Vec<u32>>,
}

#[derive(Parser)]
#[clap(arg_required_else_help(true))]
/// Steam Store DLC List generator
struct Args {
    /// Game Name Abbreviation
    #[clap(value_enum)]
    name: Option<GameAbbr>,
    /// Custom Game ID
    #[clap(long, short)]
    id: Option<u32>,
    /// Dry run, don't write to DLC.txt
    #[clap(long, short)]
    dry_run: bool,
    #[clap(flatten)]
    verbose: Verbosity,
}

async fn fetch_app(client: &Client, id: u32) -> Result<Game> {
    let res = client.get(&format!("{}{}", URL, id)).send().await?;
    let games: GameList = match res.error_for_status() {
        Ok(res) => res.json().await?,
        Err(err) => match err.status() {
            Some(StatusCode::TOO_MANY_REQUESTS) => {
                tracing::error!("Too many requests, try again in 5 mins.");
                std::process::exit(1);
            }
            _ => return Err(Report::new(err)),
        },
    };

    match games.games.into_iter().next() {
        Some((id_str, game)) if id == id_str.parse::<u32>()? => Ok(game),
        _ => Err(eyre!("No game found for id: {}", id)),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let args = Args::parse();

    tracing_subscriber::fmt()
        .with_env_filter(args.verbose.log_level_filter().as_str())
        .init();

    let id = args
        .id
        .or_else(|| args.name.map(game_id))
        .ok_or_else(|| eyre!("No game ID or abbreviation specified. Check -h for help."))?;

    let client = reqwest::Client::new();

    let game = fetch_app(&client, id).await?;
    let mut dlc = game.data.dlc.ok_or_else(|| eyre!("No DLC found"))?;
    dlc.sort();

    let tasks: Vec<_> = futures::stream::FuturesOrdered::from_iter(dlc.into_iter().map(|d| {
        let c = client.clone();
        tokio::spawn(async move {
            let game = fetch_app(&c, d).await?;
            Ok::<_, Report>(format!("{}={}", d, game.data.name))
        })
    }))
    .try_collect()
    .await?;
    let list = tasks.into_iter().collect::<Result<Vec<_>>>()?;

    println!("Found {} DLCs, saving to DLC.txt", list.len());
    tracing::info!("DLC = {list:#?}");

    if args.dry_run {
        tracing::warn!("Dry run, not writing to DLC.txt");
    } else {
        let mut f = std::fs::File::create("DLC.txt")?;
        for line in list {
            writeln!(f, "{}", line)?;
        }
    }

    Ok(())
}
