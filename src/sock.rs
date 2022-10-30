use std::error::Error;
use std::fs;
use std::process::exit;
use std::time::Duration;
use tokio::net::UnixListener;
use tokio::time::timeout;
use crate::{default_job, MelonDB, WAIT_DELAY_MS};

const SOCK_FILE: &str = "/tmp/melonbooks-scraper.sock";

pub(crate) async fn main_loop_sock() -> Result<(), Box<dyn Error>> {
    fs::remove_file(SOCK_FILE)?;
    let listener = UnixListener::bind(SOCK_FILE)?;
    loop {
        match timeout(Duration::from_millis(WAIT_DELAY_MS), listener.accept()).await {
            Ok(Ok((mut stream, _addr))) => {
                print!("connection");
                let mut cmd = String::new();
                stream.read_to_string(&mut cmd).await?;
                handle_cmd(cmd.as_str())?;
            }
            Ok(Err(_sock_err)) => {
                println!("sock_err");
                exit(1);
            }
            Err(_elapsed) => {
                println!("elapsed");
                default_job().await?;
            }
        };
    }
}

fn handle_cmd(cmd: &str) -> std::io::Result<()>{
    print!("{cmd}");
    if cmd == "exit\n" {
        exit(0);
    }
    Ok(())
}