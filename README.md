
# sd-cache-info

Fast cache subset metadata for [sd-scripts](https://github.com/kohya-ss/sd-scripts).

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
❯ time sd-cache-info ./data/1k
Input directory: "./data/1k"
Found 2000 files
██████████████████████████████████████████████████████████ 2000/2000
Found 1000 images with captions
██████████████████████████████████████████████████████████ 1000/1000
Saving metadata cache...
Metadata cache saved to "./data/1k/metadata_cache.json"

real    0m0.128s
user    0m0.089s
sys     0m1.432s
```
