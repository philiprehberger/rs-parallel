# rs-parallel

[![CI](https://github.com/philiprehberger/rs-parallel/actions/workflows/ci.yml/badge.svg)](https://github.com/philiprehberger/rs-parallel/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/philiprehberger-parallel.svg)](https://crates.io/crates/philiprehberger-parallel)
[![Last updated](https://img.shields.io/github/last-commit/philiprehberger/rs-parallel)](https://github.com/philiprehberger/rs-parallel/commits/main)

Easy parallel iteration — par_map, par_filter, par_for_each with progress and error collection

## Installation

```toml
[dependencies]
philiprehberger-parallel = "0.1.3"
```

## Usage

```rust
use philiprehberger_parallel::ParIter;

// Parallel map (order preserved)
let squares: Vec<i32> = (0..1000).par_map(|x| x * x);

// Parallel filter
let evens: Vec<i32> = (0..100).par_filter(|x| x % 2 == 0);

// Parallel for_each
use std::sync::atomic::{AtomicUsize, Ordering};
let sum = AtomicUsize::new(0);
vec![1usize, 2, 3].par_for_each(|x| { sum.fetch_add(x, Ordering::Relaxed); });

// Parallel map with error collection
let result: Result<Vec<i32>, Vec<String>> = vec![1, 2, 3]
    .par_map_results(|x| if x > 0 { Ok(x * 2) } else { Err("negative".into()) });

// Parallel flat_map
let pairs: Vec<i32> = vec![1, 2, 3].par_flat_map(|x| vec![x, x * 10]);

// Parallel any / all / count
let has_even = vec![1, 2, 3].par_any(|x| *x % 2 == 0);
let all_positive = vec![1, 2, 3].par_all(|x| *x > 0);
let even_count = vec![1, 2, 3, 4].par_count(|x| *x % 2 == 0);
```

### Custom thread pool

```rust
use philiprehberger_parallel::{ParConfig, ParIterWith};

let config = ParConfig::new().threads(4);
let result: Vec<i32> = (0..1000).par_map_with(&config, |x| x * x);
```

### Standalone functions

```rust
use philiprehberger_parallel::{par_map, par_filter, par_for_each, par_map_results, par_chunks};

let squares = par_map(0..100, |x| x * x);
let evens = par_filter(0..100, |x| x % 2 == 0);
let sums = par_chunks(vec![1, 2, 3, 4, 5, 6], 2, |chunk| chunk.iter().sum::<i32>());
```

## API

| Function / Method | Description |
|---|---|
| `par_map(f)` | Apply `f` to each item in parallel, preserving order |
| `par_filter(f)` | Keep items where `f` returns `true`, preserving order |
| `par_for_each(f)` | Execute `f` on each item in parallel |
| `par_map_results(f)` | Apply fallible `f`, returning `Ok(results)` or `Err(errors)` |
| `par_flat_map(f)` | Map and flatten in parallel |
| `par_any(f)` | `true` if any item satisfies predicate |
| `par_all(f)` | `true` if all items satisfy predicate |
| `par_count(f)` | Count items satisfying predicate |
| `par_chunks(items, size, f)` | Process items in parallel chunks |
| `ParConfig::new()` | Create default config |
| `.threads(n)` | Set thread count |
| `.chunk_size(n)` | Set chunk size |

All `ParIter` methods have `_with` variants on `ParIterWith` that accept a `&ParConfig`.

## Development

```bash
cargo test
cargo clippy -- -D warnings
```

## Support

If you find this project useful:

⭐ [Star the repo](https://github.com/philiprehberger/rs-parallel)

🐛 [Report issues](https://github.com/philiprehberger/rs-parallel/issues?q=is%3Aissue+is%3Aopen+label%3Abug)

💡 [Suggest features](https://github.com/philiprehberger/rs-parallel/issues?q=is%3Aissue+is%3Aopen+label%3Aenhancement)

❤️ [Sponsor development](https://github.com/sponsors/philiprehberger)

🌐 [All Open Source Projects](https://philiprehberger.com/open-source-packages)

💻 [GitHub Profile](https://github.com/philiprehberger)

🔗 [LinkedIn Profile](https://www.linkedin.com/in/philiprehberger)

## License

[MIT](LICENSE)
