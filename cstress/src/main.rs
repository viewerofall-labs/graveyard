use std::time::Instant;

fn main() {
    println!("GPU Stress Test - Press Ctrl+C to stop");
    println!("This will perform heavy floating-point operations to stress the GPU/CPU");
    println!("Starting in 3 seconds...\n");

    std::thread::sleep(std::time::Duration::from_secs(3));

    let start = Instant::now();
    let mut iterations = 0u64;

    // Spawn multiple threads to max out all cores
    let num_threads = num_cpus::get();
    println!("Using {} threads\n", num_threads);

    let handles: Vec<_> = (0..num_threads)
    .map(|thread_id| {
        std::thread::spawn(move || {
            stress_compute(thread_id)
        })
    })
    .collect();

    // Monitor thread that prints stats every second
    let monitor = std::thread::spawn(move || {
        loop {
            std::thread::sleep(std::time::Duration::from_secs(1));
            let elapsed = start.elapsed().as_secs();
            println!("Running for {} seconds...", elapsed);
        }
    });

    // Wait for threads (will run until Ctrl+C)
    for handle in handles {
        let _ = handle.join();
    }

    let _ = monitor.join();
}

fn stress_compute(thread_id: usize) {
    let mut data: Vec<f64> = (0..1000000).map(|i| i as f64).collect();

    loop {
        // Heavy mathematical operations
        for i in 0..data.len() {
            data[i] = (data[i].sin().powi(3) + data[i].cos().powi(3)).sqrt();
            data[i] = data[i].tan().abs() + data[i].ln_1p();
            data[i] = (data[i] * 1.5).powf(1.3);
        }

        // Matrix-like operations
        for chunk in data.chunks_mut(1000) {
            for i in 0..chunk.len() {
                for j in 0..chunk.len() {
                    chunk[i] += chunk[j].sin() * 0.0001;
                }
            }
        }

        // Prevent optimization away
        if data[0] > 1e308 {
            println!("Thread {}: Overflow detected", thread_id);
            data[0] = 1.0;
        }
    }
}

// Add to Cargo.toml dependencies:
// [dependencies]
// num_cpus = "1.16"
