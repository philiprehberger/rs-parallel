# Changelog

## 0.1.4 (2026-03-31)

- Standardize README to 3-badge format with emoji Support section
- Update CI checkout action to v5 for Node.js 24 compatibility

## 0.1.3 (2026-03-27)

- Add GitHub issue templates, PR template, and dependabot configuration
- Update README badges and add Support section

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
