# Changelog

## 0.1.2 (2026-03-22)

- Fix README formatting

## 0.1.1 (2026-03-22)

- Fix README and CI compliance

## 0.1.0 (2026-03-19)

- `ParIter` extension trait with `par_map`, `par_filter`, `par_for_each`, `par_map_results`, `par_flat_map`, `par_any`, `par_all`, `par_count`
- `ParIterWith` trait for custom thread pool configuration via `ParConfig`
- `ParConfig` builder for setting thread count and chunk size
- Standalone functions: `par_map`, `par_filter`, `par_for_each`, `par_map_results`
- `par_chunks` function for processing items in parallel chunks
