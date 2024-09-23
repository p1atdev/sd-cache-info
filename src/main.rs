use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use futures::{stream, StreamExt, TryStreamExt};
use image::ImageReader;
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde::Serialize;

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

    let read_dir = std::fs::read_dir(&input_dir)?
        .filter_map(Result::ok)
        .collect::<Vec<_>>();
    let files_len = read_dir.len();

    println!("Found {} files!", files_len);
    println!("Filtering files...");

    let paths = read_dir
        .par_iter()
        .progress_with(get_progress_bar(files_len as u64)?)
        .filter_map(|entry| {
            let img_path = entry.path();
            let txt_path = img_path.with_extension("txt"); // caption file

            img_path
                .extension()
                .and_then(|ext| ext.to_str())
                .map_or(None, |img_ext| {
                    // must a txt file with the same name exists
                    if SUPPORTED_FILE_TYPES.contains(&img_ext) && txt_path.exists() {
                        Some(img_path.to_path_buf())
                    } else {
                        None
                    }
                })
        })
        .collect::<Vec<_>>();

    let paths_len = paths.len();

    println!("Found {} images with captions!", paths_len);
    println!("Caching metadata...");

    let progress = get_progress_bar(paths_len as u64)?;
    let metas = progress
        .wrap_stream(stream::iter(paths))
        .map(|path| async move {
            let image_path = path.clone();
            let txt_path = path.with_extension("txt");

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
        .buffer_unordered(threads)
        .try_collect::<Vec<_>>()
        .await?;
    progress.finish();

    println!("Metadata cached!");
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
