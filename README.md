
# sd-cache-info

Fast caching metadata of dataset subset for [sd-scripts](https://github.com/kohya-ss/sd-scripts).

## Install

```bash
cargo install --git https://github.com/p1atdev/sd-cache-info
```

## Usage

```bash
sd-cache-info ./data/my-photo-10k
```

This will create a metadata cache file `metadata_cache.json` in the input directory. (e.g. `./data/my-photo-10k/metadata_cache.json`)


## Example

```bash
❯ time sd-cache-info ./data/10k 
Input directory: "./data/10k"
Checking for all files...
Found 20000 files!
Filtering files...
  [00:00:00] [██████████████████████████████████████████████████████]   20000/20000  Found 10000 images with captions
Caching metadata...
  [00:11:10] [██████████████████████████████████████████████████████]   10000/10000  Saving metadata cache...
Metadata cache saved to "./data/10k/metadata_cache.json"

real	11m10.793s
user	0m1.080s
sys	0m6.725s
```


## How to use the cached metadata

Set `cache_info = true` in your dataset config file.

```toml
[general]
shuffle_caption = false
caption_extension = ".txt"

enable_bucket = true

[[datasets]]
resolution = 1024
max_bucket_reso = 2048
min_bucket_reso = 512

batch_size = 4

[[datasets.subsets]]
image_dir = "/workspace/data/my-photo-10k"
cache_info = true # <== HERE
```

Then specify `--dataset_config "/path/to/dataset.toml"` or write `dataset_config = "/path/to/dataset.toml"` in your training config file.

