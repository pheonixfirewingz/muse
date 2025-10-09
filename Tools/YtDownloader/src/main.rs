mod yt_audio;

use clap::{Parser, Subcommand};
use colored::Colorize;
use anyhow::{Result, Context};
use std::path::PathBuf;
use yt_audio::{YoutubeExplode, DownloadResult};
use std::env;
use std::sync::Arc;
use tokio::sync::Semaphore;
use futures::stream::{FuturesUnordered, StreamExt};

#[derive(Parser)]
#[command(name = "yt-audio")]
#[command(author, version, about = "Search and download best-quality YouTube audio", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Search {
        query: String,
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },
    Download {
        urls: Vec<String>,
        #[arg(short, long)]
        output: Option<PathBuf>,
        #[arg(long, default_value_t = 8)]
        max_concurrent: usize,
        #[arg(long)]
        quiet: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let yt = Arc::new(YoutubeExplode::new(true)?);
    yt.check_installed().await.context("YoutubeExplode CLI not found or failed to run!")?;
    println!("{} {}", "Using YoutubeExplode at:".green(), yt.path().unwrap().display());

    match cli.command {
        Commands::Search { query, limit } => {
            println!("{} '{}'", "Searching for:".cyan(), query);
            let results = yt.search(&query, limit).await?;
            match serde_json::from_str::<Vec<serde_json::Value>>(&results) {
                Ok(videos) => {
                    println!("\n{} {} results\n", "Found".green(), videos.len());
                    for (i, video) in videos.iter().enumerate() {
                        if let Some(title) = video.get("Title").and_then(|v| v.as_str()) {
                            println!("{}. {}", (i + 1).to_string().yellow(), title.bold());
                        }
                        if let Some(author) = video.get("Author").and_then(|v| v.as_str()) {
                            println!("   {}: {}", "Author".cyan(), author);
                        }
                        if let Some(id) = video.get("Id").and_then(|v| v.as_str()) {
                            println!("   {}: https://youtube.com/watch?v={}", "URL".cyan(), id);
                        }
                        if let Some(duration) = video.get("Duration").and_then(|v| v.as_str()) {
                            println!("   {}: {}", "Duration".cyan(), duration);
                        }
                        println!();
                    }
                }
                Err(_) => println!("{}", results),
            }
        }
        Commands::Download { urls, output, max_concurrent, quiet } => {
            if urls.is_empty() {
                eprintln!("{}", "Error: Provide at least one YouTube URL.".red());
                std::process::exit(1);
            }

            yt.set_logging(!quiet);

            let output_dir = output.unwrap_or_else(|| {
                env::current_exe().ok().and_then(|p| p.parent().map(|p| p.to_path_buf()))
                    .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
            });

            println!("{} {} (max {} concurrent)\n", "Starting downloads to:".green(), output_dir.display(), max_concurrent);

            let semaphore = Arc::new(Semaphore::new(max_concurrent));
            let mut futures = FuturesUnordered::new();

            for url in urls {
                let yt_clone = yt.clone();
                let sem_clone = semaphore.clone();
                let output_dir_clone = output_dir.clone();

                let fut = async move {
                    let _permit = sem_clone.acquire().await.expect("Semaphore closed");
                    if !quiet { println!("▶ Downloading: {}", url); }

                    let result: DownloadResult = yt_clone.download_audio(&url, Some(&output_dir_clone))
                        .await.unwrap_or(DownloadResult { success: false, file_path: None, error_message: Some("Unknown error".to_string()) });

                    if result.success {
                        println!("{} {}", "✅ Done:".green().bold(), result.file_path.unwrap().display());
                    } else {
                        eprintln!("{} {} -> {}", "❌ Failed:".red().bold(), url, result.error_message.unwrap_or("Unknown error".into()));
                    }
                };

                futures.push(tokio::spawn(fut));
            }

            while futures.next().await.is_some() {}
            println!("\n{}", "All downloads completed.".green().bold());
        }
    }

    Ok(())
}
