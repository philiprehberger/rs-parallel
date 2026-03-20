//! # philiprehberger-parallel
//!
//! Easy parallel iteration utilities built on top of [rayon](https://crates.io/crates/rayon).
//!
//! Provides a `ParIter` extension trait that adds parallel combinators (`par_map`,
//! `par_filter`, `par_for_each`, etc.) to any `IntoIterator`, a `ParIterWith` trait
//! for custom thread pool configuration, and standalone functions for when you don't
//! want to import a trait.
//!
//! # Quick Start
//!
//! ```rust
//! use philiprehberger_parallel::ParIter;
//!
//! let squares: Vec<i32> = (0..100).par_map(|x| x * x);
//! let evens: Vec<i32> = (0..100).par_filter(|x| x % 2 == 0);
//! ```
//!
//! # With custom configuration
//!
//! ```rust
//! use philiprehberger_parallel::{ParConfig, ParIterWith};
//!
//! let config = ParConfig::new().threads(4);
//! let results: Vec<i32> = (0..100).par_map_with(&config, |x| x * x);
//! ```

use rayon::iter::{
    IntoParallelIterator, ParallelIterator,
};

/// Configuration for parallel operations.
///
/// Controls the number of threads and chunk size used by parallel iterators.
/// When no options are set, rayon's default thread pool is used.
///
/// # Example
///
/// ```rust
/// use philiprehberger_parallel::ParConfig;
///
/// let config = ParConfig::new().threads(4).chunk_size(100);
/// ```
#[derive(Debug, Clone, Default)]
pub struct ParConfig {
    num_threads: Option<usize>,
    chunk_size: Option<usize>,
}

impl ParConfig {
    /// Create a new `ParConfig` with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the number of threads to use.
    ///
    /// If not set, rayon's default thread pool (typically one thread per CPU core) is used.
    pub fn threads(mut self, n: usize) -> Self {
        self.num_threads = Some(n);
        self
    }

    /// Set the chunk size for parallel processing.
    ///
    /// This controls how items are divided among threads when using chunk-based operations.
    pub fn chunk_size(mut self, n: usize) -> Self {
        self.chunk_size = Some(n);
        self
    }

    /// Run a closure within a custom thread pool configured by this `ParConfig`.
    ///
    /// If `num_threads` is set, builds a dedicated rayon thread pool.
    /// Otherwise, runs the closure on the global thread pool.
    fn run<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R + Send,
        R: Send,
    {
        if let Some(threads) = self.num_threads {
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(threads)
                .build()
                .expect("failed to build rayon thread pool");
            pool.install(f)
        } else {
            f()
        }
    }
}

/// Extension trait that adds parallel combinators to any `IntoIterator`.
///
/// All methods collect the iterator into a `Vec` first, then process items
/// in parallel using rayon. Results maintain input order where applicable
/// (e.g., `par_map`, `par_filter`).
pub trait ParIter: IntoIterator {
    /// Apply a function to each item in parallel, returning results in input order.
    ///
    /// # Example
    ///
    /// ```rust
    /// use philiprehberger_parallel::ParIter;
    ///
    /// let squares: Vec<i32> = vec![1, 2, 3].par_map(|x| x * x);
    /// assert_eq!(squares, vec![1, 4, 9]);
    /// ```
    fn par_map<F, R>(self, f: F) -> Vec<R>
    where
        F: Fn(Self::Item) -> R + Sync + Send,
        R: Send;

    /// Filter items in parallel, preserving input order.
    ///
    /// # Example
    ///
    /// ```rust
    /// use philiprehberger_parallel::ParIter;
    ///
    /// let evens: Vec<i32> = vec![1, 2, 3, 4].par_filter(|x| x % 2 == 0);
    /// assert_eq!(evens, vec![2, 4]);
    /// ```
    fn par_filter<F>(self, f: F) -> Vec<Self::Item>
    where
        F: Fn(&Self::Item) -> bool + Sync + Send,
        Self::Item: Send;

    /// Execute a function on each item in parallel.
    ///
    /// # Example
    ///
    /// ```rust
    /// use philiprehberger_parallel::ParIter;
    /// use std::sync::atomic::{AtomicUsize, Ordering};
    ///
    /// let counter = AtomicUsize::new(0);
    /// vec![1, 2, 3].par_for_each(|_| { counter.fetch_add(1, Ordering::Relaxed); });
    /// assert_eq!(counter.load(Ordering::Relaxed), 3);
    /// ```
    fn par_for_each<F>(self, f: F)
    where
        F: Fn(Self::Item) + Sync + Send;

    /// Apply a fallible function in parallel, collecting all `Ok` values or all `Err` values.
    ///
    /// Returns `Ok(results)` if every item succeeds, or `Err(errors)` containing
    /// all the errors if any item fails.
    ///
    /// # Example
    ///
    /// ```rust
    /// use philiprehberger_parallel::ParIter;
    ///
    /// let result: Result<Vec<i32>, Vec<String>> = vec![1, 2, 3]
    ///     .par_map_results(|x| if x > 0 { Ok(x * 2) } else { Err("negative".into()) });
    /// assert_eq!(result, Ok(vec![2, 4, 6]));
    /// ```
    fn par_map_results<F, T, E>(self, f: F) -> Result<Vec<T>, Vec<E>>
    where
        F: Fn(Self::Item) -> Result<T, E> + Sync + Send,
        T: Send,
        E: Send;

    /// Apply a function that returns an iterator, flattening results in parallel.
    ///
    /// # Example
    ///
    /// ```rust
    /// use philiprehberger_parallel::ParIter;
    ///
    /// let result: Vec<i32> = vec![1, 2, 3].par_flat_map(|x| vec![x, x * 10]);
    /// assert_eq!(result, vec![1, 10, 2, 20, 3, 30]);
    /// ```
    fn par_flat_map<F, I, R>(self, f: F) -> Vec<R>
    where
        F: Fn(Self::Item) -> I + Sync + Send,
        I: IntoIterator<Item = R>,
        R: Send;

    /// Return `true` if any item satisfies the predicate (parallel short-circuiting).
    ///
    /// # Example
    ///
    /// ```rust
    /// use philiprehberger_parallel::ParIter;
    ///
    /// assert!(vec![1, 2, 3].par_any(|x| *x == 2));
    /// assert!(!vec![1, 2, 3].par_any(|x| *x == 5));
    /// ```
    fn par_any<F>(self, f: F) -> bool
    where
        F: Fn(&Self::Item) -> bool + Sync + Send;

    /// Return `true` if all items satisfy the predicate (parallel short-circuiting).
    ///
    /// # Example
    ///
    /// ```rust
    /// use philiprehberger_parallel::ParIter;
    ///
    /// assert!(vec![2, 4, 6].par_all(|x| *x % 2 == 0));
    /// assert!(!vec![1, 2, 3].par_all(|x| *x % 2 == 0));
    /// ```
    fn par_all<F>(self, f: F) -> bool
    where
        F: Fn(&Self::Item) -> bool + Sync + Send;

    /// Count items that satisfy the predicate in parallel.
    ///
    /// # Example
    ///
    /// ```rust
    /// use philiprehberger_parallel::ParIter;
    ///
    /// assert_eq!(vec![1, 2, 3, 4].par_count(|x| *x % 2 == 0), 2);
    /// ```
    fn par_count<F>(self, f: F) -> usize
    where
        F: Fn(&Self::Item) -> bool + Sync + Send;
}

impl<I> ParIter for I
where
    I: IntoIterator,
    I::Item: Send,
{
    fn par_map<F, R>(self, f: F) -> Vec<R>
    where
        F: Fn(Self::Item) -> R + Sync + Send,
        R: Send,
    {
        let items: Vec<_> = self.into_iter().collect();
        items.into_par_iter().map(f).collect()
    }

    fn par_filter<F>(self, f: F) -> Vec<Self::Item>
    where
        F: Fn(&Self::Item) -> bool + Sync + Send,
        Self::Item: Send,
    {
        let items: Vec<_> = self.into_iter().collect();
        items.into_par_iter().filter(|x| f(x)).collect()
    }

    fn par_for_each<F>(self, f: F)
    where
        F: Fn(Self::Item) + Sync + Send,
    {
        let items: Vec<_> = self.into_iter().collect();
        items.into_par_iter().for_each(f);
    }

    fn par_map_results<F, T, E>(self, f: F) -> Result<Vec<T>, Vec<E>>
    where
        F: Fn(Self::Item) -> Result<T, E> + Sync + Send,
        T: Send,
        E: Send,
    {
        let items: Vec<_> = self.into_iter().collect();
        let results: Vec<Result<T, E>> = items.into_par_iter().map(f).collect();

        let mut oks = Vec::new();
        let mut errs = Vec::new();

        for result in results {
            match result {
                Ok(v) => oks.push(v),
                Err(e) => errs.push(e),
            }
        }

        if errs.is_empty() {
            Ok(oks)
        } else {
            Err(errs)
        }
    }

    fn par_flat_map<F, II, R>(self, f: F) -> Vec<R>
    where
        F: Fn(Self::Item) -> II + Sync + Send,
        II: IntoIterator<Item = R>,
        R: Send,
    {
        let items: Vec<_> = self.into_iter().collect();
        items.into_par_iter().flat_map_iter(f).collect()
    }

    fn par_any<F>(self, f: F) -> bool
    where
        F: Fn(&Self::Item) -> bool + Sync + Send,
    {
        let items: Vec<_> = self.into_iter().collect();
        items.into_par_iter().any(|x| f(&x))
    }

    fn par_all<F>(self, f: F) -> bool
    where
        F: Fn(&Self::Item) -> bool + Sync + Send,
    {
        let items: Vec<_> = self.into_iter().collect();
        items.into_par_iter().all(|x| f(&x))
    }

    fn par_count<F>(self, f: F) -> usize
    where
        F: Fn(&Self::Item) -> bool + Sync + Send,
    {
        let items: Vec<_> = self.into_iter().collect();
        items.into_par_iter().filter(|x| f(x)).count()
    }
}

/// Extension trait that adds parallel combinators with custom configuration.
///
/// Like `ParIter`, but each method takes a `&ParConfig` to control
/// thread pool settings.
pub trait ParIterWith: IntoIterator {
    /// Apply a function to each item in parallel with custom configuration.
    fn par_map_with<F, R>(self, config: &ParConfig, f: F) -> Vec<R>
    where
        F: Fn(Self::Item) -> R + Sync + Send,
        R: Send;

    /// Filter items in parallel with custom configuration.
    fn par_filter_with<F>(self, config: &ParConfig, f: F) -> Vec<Self::Item>
    where
        F: Fn(&Self::Item) -> bool + Sync + Send,
        Self::Item: Send;

    /// Execute a function on each item in parallel with custom configuration.
    fn par_for_each_with<F>(self, config: &ParConfig, f: F)
    where
        F: Fn(Self::Item) + Sync + Send;

    /// Apply a fallible function in parallel with custom configuration.
    fn par_map_results_with<F, T, E>(self, config: &ParConfig, f: F) -> Result<Vec<T>, Vec<E>>
    where
        F: Fn(Self::Item) -> Result<T, E> + Sync + Send,
        T: Send,
        E: Send;

    /// Apply a function returning an iterator, flattening results in parallel with custom configuration.
    fn par_flat_map_with<F, I, R>(self, config: &ParConfig, f: F) -> Vec<R>
    where
        F: Fn(Self::Item) -> I + Sync + Send,
        I: IntoIterator<Item = R>,
        R: Send;

    /// Return `true` if any item satisfies the predicate, with custom configuration.
    fn par_any_with<F>(self, config: &ParConfig, f: F) -> bool
    where
        F: Fn(&Self::Item) -> bool + Sync + Send;

    /// Return `true` if all items satisfy the predicate, with custom configuration.
    fn par_all_with<F>(self, config: &ParConfig, f: F) -> bool
    where
        F: Fn(&Self::Item) -> bool + Sync + Send;

    /// Count items satisfying the predicate in parallel, with custom configuration.
    fn par_count_with<F>(self, config: &ParConfig, f: F) -> usize
    where
        F: Fn(&Self::Item) -> bool + Sync + Send;
}

impl<I> ParIterWith for I
where
    I: IntoIterator,
    I::Item: Send,
{
    fn par_map_with<F, R>(self, config: &ParConfig, f: F) -> Vec<R>
    where
        F: Fn(Self::Item) -> R + Sync + Send,
        R: Send,
    {
        let items: Vec<_> = self.into_iter().collect();
        config.run(|| items.into_par_iter().map(f).collect())
    }

    fn par_filter_with<F>(self, config: &ParConfig, f: F) -> Vec<Self::Item>
    where
        F: Fn(&Self::Item) -> bool + Sync + Send,
        Self::Item: Send,
    {
        let items: Vec<_> = self.into_iter().collect();
        config.run(|| items.into_par_iter().filter(|x| f(x)).collect())
    }

    fn par_for_each_with<F>(self, config: &ParConfig, f: F)
    where
        F: Fn(Self::Item) + Sync + Send,
    {
        let items: Vec<_> = self.into_iter().collect();
        config.run(|| items.into_par_iter().for_each(f));
    }

    fn par_map_results_with<F, T, E>(self, config: &ParConfig, f: F) -> Result<Vec<T>, Vec<E>>
    where
        F: Fn(Self::Item) -> Result<T, E> + Sync + Send,
        T: Send,
        E: Send,
    {
        let items: Vec<_> = self.into_iter().collect();
        let results: Vec<Result<T, E>> =
            config.run(|| items.into_par_iter().map(f).collect());

        let mut oks = Vec::new();
        let mut errs = Vec::new();

        for result in results {
            match result {
                Ok(v) => oks.push(v),
                Err(e) => errs.push(e),
            }
        }

        if errs.is_empty() {
            Ok(oks)
        } else {
            Err(errs)
        }
    }

    fn par_flat_map_with<F, II, R>(self, config: &ParConfig, f: F) -> Vec<R>
    where
        F: Fn(Self::Item) -> II + Sync + Send,
        II: IntoIterator<Item = R>,
        R: Send,
    {
        let items: Vec<_> = self.into_iter().collect();
        config.run(|| items.into_par_iter().flat_map_iter(f).collect())
    }

    fn par_any_with<F>(self, config: &ParConfig, f: F) -> bool
    where
        F: Fn(&Self::Item) -> bool + Sync + Send,
    {
        let items: Vec<_> = self.into_iter().collect();
        config.run(|| items.into_par_iter().any(|x| f(&x)))
    }

    fn par_all_with<F>(self, config: &ParConfig, f: F) -> bool
    where
        F: Fn(&Self::Item) -> bool + Sync + Send,
    {
        let items: Vec<_> = self.into_iter().collect();
        config.run(|| items.into_par_iter().all(|x| f(&x)))
    }

    fn par_count_with<F>(self, config: &ParConfig, f: F) -> usize
    where
        F: Fn(&Self::Item) -> bool + Sync + Send,
    {
        let items: Vec<_> = self.into_iter().collect();
        config.run(|| items.into_par_iter().filter(|x| f(x)).count())
    }
}

// ---------------------------------------------------------------------------
// Standalone functions
// ---------------------------------------------------------------------------

/// Apply a function to each item in parallel, returning results in input order.
///
/// Standalone version of [`ParIter::par_map`] for when you don't want to import the trait.
///
/// # Example
///
/// ```rust
/// let squares = philiprehberger_parallel::par_map(0..5, |x| x * x);
/// assert_eq!(squares, vec![0, 1, 4, 9, 16]);
/// ```
pub fn par_map<T, F, R>(items: impl IntoIterator<Item = T>, f: F) -> Vec<R>
where
    T: Send,
    F: Fn(T) -> R + Sync + Send,
    R: Send,
{
    let items: Vec<T> = items.into_iter().collect();
    items.into_par_iter().map(f).collect()
}

/// Filter items in parallel, preserving input order.
///
/// Standalone version of [`ParIter::par_filter`].
///
/// # Example
///
/// ```rust
/// let evens = philiprehberger_parallel::par_filter(0..10, |x| x % 2 == 0);
/// assert_eq!(evens, vec![0, 2, 4, 6, 8]);
/// ```
pub fn par_filter<T, F>(items: impl IntoIterator<Item = T>, f: F) -> Vec<T>
where
    T: Send,
    F: Fn(&T) -> bool + Sync + Send,
{
    let items: Vec<T> = items.into_iter().collect();
    items.into_par_iter().filter(|x| f(x)).collect()
}

/// Execute a function on each item in parallel.
///
/// Standalone version of [`ParIter::par_for_each`].
///
/// # Example
///
/// ```rust
/// use std::sync::atomic::{AtomicUsize, Ordering};
///
/// let sum = AtomicUsize::new(0);
/// philiprehberger_parallel::par_for_each(vec![1, 2, 3], |x| {
///     sum.fetch_add(x, Ordering::Relaxed);
/// });
/// assert_eq!(sum.load(Ordering::Relaxed), 6);
/// ```
pub fn par_for_each<T, F>(items: impl IntoIterator<Item = T>, f: F)
where
    T: Send,
    F: Fn(T) + Sync + Send,
{
    let items: Vec<T> = items.into_iter().collect();
    items.into_par_iter().for_each(f);
}

/// Apply a fallible function in parallel, collecting all results or all errors.
///
/// Standalone version of [`ParIter::par_map_results`].
///
/// # Example
///
/// ```rust
/// let result: Result<Vec<i32>, Vec<&str>> =
///     philiprehberger_parallel::par_map_results(vec![1, 2, 3], |x| Ok(x * 2));
/// assert_eq!(result, Ok(vec![2, 4, 6]));
/// ```
pub fn par_map_results<T, F, R, E>(
    items: impl IntoIterator<Item = T>,
    f: F,
) -> Result<Vec<R>, Vec<E>>
where
    T: Send,
    F: Fn(T) -> Result<R, E> + Sync + Send,
    R: Send,
    E: Send,
{
    let items: Vec<T> = items.into_iter().collect();
    let results: Vec<Result<R, E>> = items.into_par_iter().map(f).collect();

    let mut oks = Vec::new();
    let mut errs = Vec::new();

    for result in results {
        match result {
            Ok(v) => oks.push(v),
            Err(e) => errs.push(e),
        }
    }

    if errs.is_empty() {
        Ok(oks)
    } else {
        Err(errs)
    }
}

/// Process items in parallel chunks.
///
/// Collects items into a `Vec`, splits it into chunks of `chunk_size`,
/// and processes each chunk in parallel with the given function.
///
/// # Example
///
/// ```rust
/// let sums = philiprehberger_parallel::par_chunks(vec![1, 2, 3, 4, 5, 6], 2, |chunk| {
///     chunk.into_iter().sum::<i32>()
/// });
/// assert_eq!(sums, vec![3, 7, 11]);
/// ```
pub fn par_chunks<T, F, R>(
    items: impl IntoIterator<Item = T>,
    chunk_size: usize,
    f: F,
) -> Vec<R>
where
    T: Send,
    F: Fn(Vec<T>) -> R + Sync + Send,
    R: Send,
{
    let mut items: Vec<T> = items.into_iter().collect();
    let mut chunks: Vec<Vec<T>> = Vec::new();
    while !items.is_empty() {
        let at = chunk_size.min(items.len());
        let rest = items.split_off(at);
        chunks.push(items);
        items = rest;
    }
    chunks.into_par_iter().map(f).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn test_par_map_squares() {
        let input: Vec<i32> = (0..100).collect();
        let result: Vec<i32> = input.par_map(|x| x * x);
        let expected: Vec<i32> = (0..100).map(|x| x * x).collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_par_filter() {
        let evens: Vec<i32> = (0..20).par_filter(|x| x % 2 == 0);
        let expected: Vec<i32> = (0..20).step_by(2).collect();
        assert_eq!(evens, expected);
    }

    #[test]
    fn test_par_for_each() {
        let counter = AtomicUsize::new(0);
        let items: Vec<usize> = (0..50).collect();
        items.par_for_each(|_| {
            counter.fetch_add(1, Ordering::Relaxed);
        });
        assert_eq!(counter.load(Ordering::Relaxed), 50);
    }

    #[test]
    fn test_par_map_results_all_ok() {
        let result: Result<Vec<i32>, Vec<String>> =
            vec![1, 2, 3].par_map_results(|x| Ok(x * 2));
        assert_eq!(result, Ok(vec![2, 4, 6]));
    }

    #[test]
    fn test_par_map_results_some_err() {
        let result: Result<Vec<i32>, Vec<String>> =
            vec![1, -1, 2, -2].par_map_results(|x| {
                if x > 0 {
                    Ok(x)
                } else {
                    Err(format!("negative: {}", x))
                }
            });
        assert!(result.is_err());
        let errs = result.unwrap_err();
        assert_eq!(errs.len(), 2);
    }

    #[test]
    fn test_par_flat_map() {
        let result: Vec<i32> = vec![1, 2, 3].par_flat_map(|x| vec![x, x * 10]);
        assert_eq!(result, vec![1, 10, 2, 20, 3, 30]);
    }

    #[test]
    fn test_par_any() {
        assert!(vec![1, 2, 3, 4, 5].par_any(|x| *x == 3));
        assert!(!vec![1, 2, 3, 4, 5].par_any(|x| *x == 10));
    }

    #[test]
    fn test_par_all() {
        assert!(vec![2, 4, 6, 8].par_all(|x| *x % 2 == 0));
        assert!(!vec![2, 4, 5, 8].par_all(|x| *x % 2 == 0));
    }

    #[test]
    fn test_par_count() {
        let count = vec![1, 2, 3, 4, 5, 6].par_count(|x| *x % 2 == 0);
        assert_eq!(count, 3);
    }

    #[test]
    fn test_par_chunks() {
        let sums = par_chunks(vec![1, 2, 3, 4, 5, 6], 2, |chunk| {
            chunk.into_iter().sum::<i32>()
        });
        assert_eq!(sums, vec![3, 7, 11]);
    }

    #[test]
    fn test_empty_collection() {
        let result: Vec<i32> = Vec::<i32>::new().par_map(|x| x * x);
        assert!(result.is_empty());

        let filtered: Vec<i32> = Vec::<i32>::new().par_filter(|_| true);
        assert!(filtered.is_empty());

        assert!(!Vec::<i32>::new().par_any(|_| true));
        assert!(Vec::<i32>::new().par_all(|_| false));
        assert_eq!(Vec::<i32>::new().par_count(|_| true), 0);
    }

    #[test]
    fn test_single_element() {
        let result: Vec<i32> = vec![42].par_map(|x| x * 2);
        assert_eq!(result, vec![84]);

        assert!(vec![42].par_any(|x| *x == 42));
        assert!(vec![42].par_all(|x| *x == 42));
        assert_eq!(vec![42].par_count(|x| *x == 42), 1);
    }

    #[test]
    fn test_ordering_preserved_par_map() {
        let input: Vec<i32> = (0..1000).collect();
        let result: Vec<i32> = input.par_map(|x| x * 2);
        let expected: Vec<i32> = (0..1000).map(|x| x * 2).collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_par_config_with_threads() {
        let config = ParConfig::new().threads(2);
        let result: Vec<i32> = (0..100).par_map_with(&config, |x| x * x);
        let expected: Vec<i32> = (0..100).map(|x| x * x).collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_par_config_filter_with() {
        let config = ParConfig::new().threads(2);
        let result: Vec<i32> = (0..20).par_filter_with(&config, |x| x % 2 == 0);
        let expected: Vec<i32> = (0..20).step_by(2).collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_standalone_par_map() {
        let result = par_map(0..5, |x| x * x);
        assert_eq!(result, vec![0, 1, 4, 9, 16]);
    }

    #[test]
    fn test_standalone_par_filter() {
        let result = par_filter(0..10, |x| x % 2 == 0);
        assert_eq!(result, vec![0, 2, 4, 6, 8]);
    }

    #[test]
    fn test_standalone_par_for_each() {
        let counter = AtomicUsize::new(0);
        par_for_each(0..10usize, |_| {
            counter.fetch_add(1, Ordering::Relaxed);
        });
        assert_eq!(counter.load(Ordering::Relaxed), 10);
    }

    #[test]
    fn test_standalone_par_map_results() {
        let result: Result<Vec<i32>, Vec<&str>> =
            par_map_results(vec![1, 2, 3], |x| Ok(x * 2));
        assert_eq!(result, Ok(vec![2, 4, 6]));
    }

    #[test]
    fn test_par_chunks_single_chunk() {
        let result = par_chunks(vec![1, 2, 3], 10, |chunk| chunk.len());
        assert_eq!(result, vec![3]);
    }

    #[test]
    fn test_par_chunks_empty() {
        let result = par_chunks(Vec::<i32>::new(), 5, |chunk| chunk.len());
        assert!(result.is_empty());
    }
}
