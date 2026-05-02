use std::time::Instant;
use sysinfo::System;

#[tokio::main]
async fn main() {
    println!("GPU Stress Test - Press Ctrl+C to stop");
    println!("Initializing GPU...\n");

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            ..Default::default()
        })
        .await
        .expect("Failed to find an appropriate adapter");

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
            },
            None,
        )
        .await
        .expect("Failed to create device");

    println!("GPU: {}", adapter.get_info().name);
    println!("Backend: {:?}\n", adapter.get_info().backend);

    // Initialize system monitoring
    let mut sys = System::new_all();

    println!("Starting stress test...\n");
    print_system_info(&mut sys);
    println!("\n{}", "=".repeat(70));

    // Compute shader that does heavy mathematical operations
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Stress Test Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
    });

    let buffer_size = 10_000_000 * std::mem::size_of::<f32>();
    let input_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Input Buffer"),
        size: buffer_size as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Output Buffer"),
        size: buffer_size as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Bind Group Layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Bind Group"),
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: input_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: output_buffer.as_entire_binding(),
            },
        ],
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Pipeline Layout"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });

    let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("Compute Pipeline"),
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: "main",
    });

    let start = Instant::now();
    let mut iterations = 0;
    let mut last_update = Instant::now();

    loop {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Command Encoder"),
        });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Compute Pass"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&compute_pipeline);
            compute_pass.set_bind_group(0, &bind_group, &[]);
            compute_pass.dispatch_workgroups(35000, 1, 1);
        }

        queue.submit(Some(encoder.finish()));
        device.poll(wgpu::Maintain::Wait);

        iterations += 1;
        
        // Update stats every second
        if last_update.elapsed().as_secs() >= 1 {
            sys.refresh_all();
            
            print!("\x1B[2J\x1B[1;1H"); // Clear screen and move cursor to top
            println!("GPU Stress Test - Press Ctrl+C to stop");
            println!("GPU: {}", adapter.get_info().name);
            println!("{}\n", "=".repeat(70));
            
            let elapsed = start.elapsed().as_secs_f64();
            let fps = iterations as f64 / elapsed;
            
            println!("📊 Performance:");
            println!("  Iterations: {} | Time: {:.1}s | Rate: {:.1} iter/s", 
                     iterations, elapsed, fps);
            println!();
            
            print_system_info(&mut sys);
            
            last_update = Instant::now();
        }
    }
}

fn print_system_info(sys: &mut System) {
    // CPU info
    sys.refresh_cpu();
    let cpus = sys.cpus();
    let cpu_usage: f32 = cpus.iter().map(|cpu| cpu.cpu_usage()).sum::<f32>() / cpus.len() as f32;
    println!("🖥️  CPU Usage: {:.1}%", cpu_usage);
    
    // Memory info
    sys.refresh_memory();
    let used_mem = sys.used_memory() as f64 / 1_073_741_824.0; // Convert to GB
    let total_mem = sys.total_memory() as f64 / 1_073_741_824.0;
    println!("💾 Memory: {:.2} GB / {:.2} GB ({:.1}%)", 
             used_mem, total_mem, (used_mem / total_mem) * 100.0);
    
    // Temperature info - sysinfo 0.30 uses Components separately
    println!("\n🌡️  System Info:");
    println!("  Run 'nvidia-smi' or 'sensors' in another terminal for GPU temps");
}

// Create a file named shader.wgsl in src/ with this content:
/*
@group(0) @binding(0) var<storage, read> input: array<f32>;
@group(0) @binding(1) var<storage, read_write> output: array<f32>;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = global_id.x;
    if (idx >= arrayLength(&input)) {
        return;
    }
    
    var result = input[idx];
    
    // Heavy mathematical operations to stress the GPU
    for (var i = 0u; i < 100u; i++) {
        result = sin(result) * cos(result) + tan(result * 0.01);
        result = sqrt(abs(result)) + pow(abs(result), 1.5);
        result = result * 1.01 - 0.001;
        
        // Matrix-like operations
        for (var j = 0u; j < 10u; j++) {
            let val = f32(j) * 0.1;
            result = result + sin(val * result) * cos(val);
        }
    }
    
    output[idx] = result;
}
*/

// Add to Cargo.toml:
// [dependencies]
// wgpu = "0.19"
// tokio = { version = "1", features = ["full"] }
// sysinfo = "0.30"
