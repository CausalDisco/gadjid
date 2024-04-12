// SPDX-License-Identifier: MPL-2.0

use std::env;
use std::str::FromStr;

/// Initialize rayon's global thread pool with the default number of threads being
/// the number of physical CPUs instead of logical CPUs (the current rayon default),
/// unless the environment variable `RAYON_NUM_THREADS` is set to a positive integer,
/// in which case that determines the number of threads in the thread pool.
pub fn build_global() {
    let num_threads = match env::var("RAYON_NUM_THREADS")
        .ok()
        .and_then(|s| usize::from_str(&s).ok())
    {
        Some(x @ 1..) => x,
        _ => num_cpus::get_physical(),
    };

    let _ = rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build_global();
}
