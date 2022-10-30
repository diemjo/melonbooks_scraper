extern crate core;

use tokio::task::spawn_blocking;
use crate::cli::Args;
use clap::Parser;
use lazy_static::lazy_static;
use crate::model::Availability::{Available, Preorder};
use crate::config::AppConfig;

#[cfg(feature = "sock")]
mod sock;

mod job;
mod db;
mod common;
mod web;
mod model;
mod cli;
mod config;

const WAIT_DELAY_MS: u64 = 14_400_000;

lazy_static! {
    pub static ref CONFIGURATION: AppConfig = AppConfig::load_config();
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    let args: Args = Args::parse();
    if args.daemon {
        #[cfg(feature = "sock")]
        {
            sock::main_loop_sock().await?;
            Ok(())
        }

        #[cfg(not(feature = "sock"))]
        {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(WAIT_DELAY_MS));
            loop {
                interval.tick().await;
                spawn_blocking(move || job::default_job()).await??;
            }
        }
    }
    else if args.load_new {
        spawn_blocking(move || job::load_products(args.also_new_unavailable)).await??;
    }
    else if args.refresh {
        spawn_blocking(move || job::update_products(vec![Available, Preorder])).await??;
    }
    else if args.add_artist.is_some(){
        spawn_blocking(move || job::add_artist(args.add_artist.unwrap().as_str(), args.site.unwrap().as_str())).await??;
    }
    else if args.remove_artist.is_some() {
        spawn_blocking(move || job::remove_artist(args.remove_artist.unwrap().as_str(), args.site.unwrap().as_str())).await??;
    }
    Ok(())
}
