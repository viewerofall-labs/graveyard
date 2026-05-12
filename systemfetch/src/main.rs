use sysinfo::{System, Disks, Networks, Components};
use std::fs;
use chrono::Local;

fn main() {
    let mut sys = System::new_all();
    sys.refresh_all();
    
    println!("\n");
    print_header();
    println!();
    
    print_os_info();
    print_hardware_info();
    print_cpu_info(&sys);
    print_memory_info(&sys);
    print_disk_info();
    print_network_info();
    print_temperature_info();
    print_uptime_info(&sys);
    
    println!();
}

fn print_header() {
    let username = whoami::username();
    let hostname = whoami::fallible::hostname().unwrap_or_else(|_| "unknown".to_string());
    
    println!("╔{}╗", "═".repeat(50));
    println!("║ {}@{:<43} ║", username, hostname);
    println!("╠{}╣", "═".repeat(50));
}

fn print_os_info() {
    println!("║ OS                                                ║");
    println!("║   Name: {:<41} ║", System::name().unwrap_or_else(|| "Unknown".to_string()));
    println!("║   Kernel: {:<39} ║", System::kernel_version().unwrap_or_else(|| "Unknown".to_string()));
    println!("║   OS Version: {:<35} ║", System::os_version().unwrap_or_else(|| "Unknown".to_string()));
    println!("║   Architecture: {:<33} ║", std::env::consts::ARCH);
    println!("╠{}╣", "═".repeat(50));
}

fn print_hardware_info() {
    println!("║ HARDWARE                                          ║");
    
    if let Some(host) = System::host_name() {
        println!("║   Host: {:<41} ║", host);
    }
    
    println!("╠{}╣", "─".repeat(50));
}

fn print_cpu_info(sys: &System) {
    println!("║ CPU                                               ║");
    
    let cpus = sys.cpus();
    if let Some(cpu) = cpus.first() {
        println!("║   Model: {:<40} ║", truncate(&cpu.brand(), 40));
        println!("║   Cores: {:<40} ║", sys.cpus().len());
        
        let mut frequencies = String::new();
        for (i, cpu) in cpus.iter().enumerate().take(8) {
            if i > 0 && i % 4 == 0 {
                println!("║   Freq: {:<41} ║", frequencies);
                frequencies.clear();
            }
            frequencies.push_str(&format!("C{}: {:.2}GHz ", i, cpu.frequency() as f64 / 1000.0));
        }
        if !frequencies.is_empty() {
            println!("║   Freq: {:<41} ║", frequencies);
        }
        
        println!("║   Usage:                                          ║");
        for (i, cpu) in cpus.iter().enumerate() {
            let usage_bar = create_bar(cpu.cpu_usage(), 20);
            println!("║     Core {:2}: [{:20}] {:5.1}%        ║", i, usage_bar, cpu.cpu_usage());
        }
    }
    
    println!("║   Global CPU Usage: {:<27.1}% ║", sys.global_cpu_usage());
    
    println!("╠{}╣", "─".repeat(50));
}

fn print_memory_info(sys: &System) {
    let total_mem = sys.total_memory();
    let used_mem = sys.used_memory();
    let total_swap = sys.total_swap();
    let used_swap = sys.used_swap();
    
    let mem_percent = (used_mem as f64 / total_mem as f64) * 100.0;
    let swap_percent = if total_swap > 0 {
        (used_swap as f64 / total_swap as f64) * 100.0
    } else {
        0.0
    };
    
    println!("║ MEMORY                                            ║");
    println!("║   RAM: {:.2} GB / {:.2} GB ({:.1}%)              ║", 
        used_mem as f64 / 1_048_576.0, 
        total_mem as f64 / 1_048_576.0,
        mem_percent
    );
    
    let mem_bar = create_bar(mem_percent as f32, 30);
    println!("║   [{:30}]            ║", mem_bar);
    
    if total_swap > 0 {
        println!("║   Swap: {:.2} GB / {:.2} GB ({:.1}%)            ║", 
            used_swap as f64 / 1_048_576.0, 
            total_swap as f64 / 1_048_576.0,
            swap_percent
        );
        let swap_bar = create_bar(swap_percent as f32, 30);
        println!("║   [{:30}]            ║", swap_bar);
    }
    
    println!("╠{}╣", "─".repeat(50));
}

fn print_disk_info() {
    let disks = Disks::new_with_refreshed_list();
    
    println!("║ DISKS                                             ║");
    
    for disk in &disks {
        let name = disk.name().to_string_lossy();
        let mount = disk.mount_point().to_string_lossy();
        let total = disk.total_space();
        let available = disk.available_space();
        let used = total - available;
        let used_percent = (used as f64 / total as f64) * 100.0;
        
        println!("║   {:<46} ║", truncate(&name, 46));
        println!("║     Mount: {:<38} ║", truncate(&mount, 38));
        println!("║     Used: {:.2} GB / {:.2} GB ({:.1}%)          ║",
            used as f64 / 1_073_741_824.0,
            total as f64 / 1_073_741_824.0,
            used_percent
        );
        
        let disk_bar = create_bar(used_percent as f32, 30);
        println!("║     [{:30}]            ║", disk_bar);
        
        if let Some(fs) = disk.file_system().to_str() {
            println!("║     Type: {:<39} ║", fs);
        }
    }
    
    println!("╠{}╣", "─".repeat(50));
}

fn print_network_info() {
    let networks = Networks::new_with_refreshed_list();
    
    println!("║ NETWORK                                           ║");
    
    for (name, network) in &networks {
        println!("║   Interface: {:<36} ║", truncate(name, 36));
        println!("║     RX: {:<12} TX: {:<12}       ║", 
            format_bytes(network.total_received()),
            format_bytes(network.total_transmitted())
        );
        println!("║     Packets RX: {:<10} TX: {:<10}   ║",
            network.total_packets_received(),
            network.total_packets_transmitted()
        );
    }
    
    println!("╠{}╣", "─".repeat(50));
}

fn print_temperature_info() {
    let components = Components::new_with_refreshed_list();
    
    if !components.is_empty() {
        println!("║ TEMPERATURES                                      ║");
        
        for component in &components {
            let temp = component.temperature();
            let label = component.label();
            
            let status = if temp > 80.0 {
                "🔥 HOT"
            } else if temp > 60.0 {
                "⚠ WARM"
            } else {
                "✓ OK"
            };
            
            println!("║   {:<30} {:5.1}°C {:6} ║", 
                truncate(label, 30),
                temp,
                status
            );
            
            let max_temp = component.max();
            if max_temp > 0.0 {
                println!("║     Max: {:.1}°C                                    ║", max_temp);
            }
        }
        
        println!("╠{}╣", "─".repeat(50));
    }
}

fn print_uptime_info(sys: &System) {
    let uptime_secs = System::uptime();
    let days = uptime_secs / 86400;
    let hours = (uptime_secs % 86400) / 3600;
    let minutes = (uptime_secs % 3600) / 60;
    
    println!("║ SYSTEM                                            ║");
    println!("║   Uptime: {} days, {} hours, {} minutes           ║", days, hours, minutes);
    println!("║   Processes: {:<36} ║", sys.processes().len());
    println!("║   Current Time: {:<32} ║", Local::now().format("%Y-%m-%d %H:%M:%S"));
    
    if let Ok(loadavg) = fs::read_to_string("/proc/loadavg") {
        let parts: Vec<&str> = loadavg.split_whitespace().collect();
        if parts.len() >= 3 {
            println!("║   Load Average: {}, {}, {}                        ║", 
                parts[0], parts[1], parts[2]);
        }
    }
    
    println!("╚{}╝", "═".repeat(50));
}

fn create_bar(percent: f32, width: usize) -> String {
    let filled = ((percent / 100.0) * width as f32) as usize;
    let empty = width - filled;
    format!("{}{}", "█".repeat(filled), "░".repeat(empty))
}

fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1_048_576 {
        format!("{:.2} KB", bytes as f64 / 1024.0)
    } else if bytes < 1_073_741_824 {
        format!("{:.2} MB", bytes as f64 / 1_048_576.0)
    } else {
        format!("{:.2} GB", bytes as f64 / 1_073_741_824.0)
    }
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len-3])
    }
}
