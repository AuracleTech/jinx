macro_rules! _bench {
    ($func:expr) => {{
        let mut total_duration = std::time::Duration::new(0, 0);
        let warmup_iterations = 1_000;
        let bench_iterations = 1_000_000;

        // TODO : Add a warmup loop before the bench loop to warm up the cache and cpu branch predictor
        // TODO : Before iterating a fixed number of times, check the time elapsed and stop iterating when a threshold is reached (e.g. 10 seconds)
        // TODO : Do not use a fixed number of iterations, but adjust the number of iterations based on the time elapsed in the previous step

        for _ in 0..warmup_iterations {
            $func;
        }

        for _ in 0..bench_iterations {
            let start_time = std::time::Instant::now();
            $func;
            let end_time = std::time::Instant::now();
            total_duration += end_time - start_time;
        }

        total_duration / bench_iterations as u32
    }};
}
