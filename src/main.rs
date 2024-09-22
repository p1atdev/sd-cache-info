use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use anyhow::Result;
use clap::{Arg, Parser};
use futures::StreamExt;
use image::ImageReader;
use indicatif::ProgressBar;
use rayon::iter::{ParallelBridge, ParallelIterator};
use serde::Serialize;
use walkdir::WalkDir;

const SUPPORTED_FILE_TYPES: [&str; 4] = ["jpg", "jpeg", "png", "webp"];

#[derive(Debug, Clone, Parser)]
struct Cli {
    /// The input directory to search for images
    input_dir: PathBuf,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct SubsetInfo {
    caption: String,
    resolution: (u32, u32),
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let input_dir = cli.input_dir;

    if tokio::fs::metadata(&input_dir).await?.is_dir() {
        println!("Input directory: {:?}", input_dir);
    } else {
        println!("Input directory does not exist: {:?}", input_dir);
        return Ok(());
    }

    let files_len = std::fs::read_dir(&input_dir)?.count();

    println!("Found {} files", files_len);

    let progress = ProgressBar::new(files_len as u64);
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

    let progress = ProgressBar::new(paths_len as u64);

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
        .buffer_unordered(num_cpus::get())
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
