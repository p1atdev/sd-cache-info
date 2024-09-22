
# sd-cache-info

Fast caching metadata of dataset subset for [sd-scripts](https://github.com/kohya-ss/sd-scripts).

## Install

```bash
cargo install --git https://github.com/p1atdev/sd-cache-info
```

## Usage

```bash
sd-cache-info ./path/to/cache
```

Example:

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
