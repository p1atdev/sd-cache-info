use std::{collections::HashMap, path::PathBuf};

use anyhow::Result;
use clap::Parser;
use futures::StreamExt;
use image::ImageReader;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::iter::{ParallelBridge, ParallelIterator};
use serde::Serialize;
use walkdir::WalkDir;

// image extensions
const SUPPORTED_FILE_TYPES: [&str; 4] = ["jpg", "jpeg", "png", "webp"];

// progress bar style
const PROGRESS_BAR_TEMPLATE: &str =
    "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos:>7}/{len:7}";
// showing eta will slow down the progress bar

#[derive(Debug, Clone, Parser)]
struct Cli {
    /// The input directory to search for images
    input_dir: PathBuf,

    #[arg(short, long, default_value_t = num_cpus::get())]
    threads: usize,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct SubsetInfo {
    caption: String,
    resolution: (u32, u32),
}

fn get_progress_bar(size: u64) -> Result<ProgressBar> {
    let style = ProgressStyle::with_template(PROGRESS_BAR_TEMPLATE)?;
    Ok(ProgressBar::new(size).with_style(style))
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let input_dir = cli.input_dir;
    let threads = cli.threads;

    if tokio::fs::metadata(&input_dir).await?.is_dir() {
        println!("Input directory: {:?}", input_dir);
    } else {
        println!("Input directory does not exist: {:?}", input_dir);
        return Ok(());
    }

    println!("Checking for all files...");

    let files_len = std::fs::read_dir(&input_dir)?.count();

    println!("Found {} files!", files_len);
    println!("Filtering files...");

    let progress = get_progress_bar(files_len as u64)?;
    let paths = progress
        .wrap_iter(WalkDir::new(&input_dir).into_iter())
        .par_bridge() // イテレータを並列処理する
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.file_type().is_file()
                && entry
                    .path()
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map_or(false, |ext_str| SUPPORTED_FILE_TYPES.contains(&ext_str))
                    &&
                    // must a txt file with the same name exists
                    entry
                        .path()
                        .with_extension("txt")
                        .exists()
        })
        .map(|entry| entry.path().to_path_buf())
        .collect::<Vec<_>>();
    progress.finish();

    let paths_len = paths.len();

    println!("Found {} images with captions", paths_len);
    println!("Caching metadata...");

    let progress = get_progress_bar(paths_len as u64)?;
    let metas = progress
        .wrap_stream(futures::stream::iter(paths))
        .map(|path| {
            let image_path = path.clone();
            let txt_path = path.with_extension("txt");

            tokio::spawn(async move {
                let image = ImageReader::open(&image_path)?;
                let txt = tokio::fs::read_to_string(txt_path).await?;

                let resolution = image.into_dimensions()?;
                let caption = txt.trim().to_string();

                Result::<_>::Ok((
                    image_path.canonicalize()?,
                    SubsetInfo {
                        caption,
                        resolution,
                    },
                ))
            })
        })
        .buffer_unordered(threads)
        .map(|res| res?)
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .collect::<HashMap<_, _>>();
    progress.finish();

    println!("Saving metadata cache...");

    let metadata_cache_path = &input_dir.join("metadata_cache.json");

    let mut writer = std::io::BufWriter::new(
        std::fs::File::options()
            .write(true)
            .create(true)
            .truncate(true)
            .open(metadata_cache_path)?,
    );
    serde_json::to_writer(&mut writer, &metas)?;

    println!("Metadata cache saved to {:?}", metadata_cache_path);

    Ok(())
}
