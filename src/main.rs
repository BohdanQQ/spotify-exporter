mod commands;

use crate::commands::make_command;
use clap::{ArgEnum, Parser, Subcommand};
use rspotify::{prelude::*, scopes, AuthCodeSpotify, Credentials, OAuth};
use std::fs::File;
use std::process::exit;

/// Exports Spotify User Data
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Args {
    /// Action
    #[clap(subcommand)]
    command: Commands,

    /// Output path
    #[clap(short, long)]
    output: String,

    /// Output format
    #[clap(short, long, arg_enum)]
    format: OutputType,

    /// Be verbose
    #[clap(short, long)]
    verbose: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum, Debug)]
pub enum OutputType {
    Markdown,
    HTML,
    MarkdownWWW,
    JSON,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum, Debug)]
pub enum CmdRange {
    Short,
    Medium,
    Long,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// List User's top tracks
    TopTracks {
        /// The time range to collect
        #[clap(arg_enum)]
        time: CmdRange,

        /// Number of items to retrieve
        #[clap(short, long, default_value_t = 50)]
        count: u8,
    },
}

#[tokio::main]
async fn main() {
    let cli = Args::parse();
    println!("{:#?}", cli);
    let creds = Credentials::from_env().unwrap();

    let scopes = scopes!(
        "user-top-read",
        "user-read-recently-played",
        "user-library-read",
        "user-read-currently-playing",
        "user-read-playback-state",
        "user-read-playback-position",
        "playlist-read-collaborative",
        "playlist-read-private"
    );
    let oauth = OAuth::from_env(scopes).unwrap();

    let mut spotify = AuthCodeSpotify::new(creds, oauth);

    let url = spotify.get_authorize_url(false).unwrap();
    spotify.prompt_for_token(&url).unwrap();

    let mut command = make_command(cli.command);

    let file = match File::create(cli.output) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("{}", e);
            exit(1)
        }
    };

    match command.execute(&spotify) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("{}", e);
            exit(1)
        }
    };

    match command.output(&file, &cli.format) {
        Ok(()) => (),
        Err(e) => {
            eprintln!("{}", e);
            exit(1)
        }
    }
}
