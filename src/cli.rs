use clap::{Parser, ArgGroup};

#[derive(Debug, Parser)]
#[clap(name = "MelonbooksScraper", about = "MelonbooksScraper CLI")]
#[clap(group(ArgGroup::new("action").args(&["daemon", "load_new", "refresh", "add_artist", "remove_artist"]).required(true)))]
pub struct Args {
    #[clap(short, long)]
    pub daemon: bool,
    #[clap(short, long)]
    pub load_new: bool,
    #[clap(long)]
    pub also_new_unavailable: bool,
    #[clap(short, long)]
    pub refresh: bool,
    #[clap(long, requires="site")]
    pub add_artist: Option<String>,
    #[clap(long, requires="site")]
    pub remove_artist: Option<String>,
    #[clap(long)]
    pub site: Option<String>
}