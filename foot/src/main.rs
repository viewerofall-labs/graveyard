use std::{
    ffi::CStr,
    fs,
    os::unix::io::{AsRawFd, FromRawFd},
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use chrono::Local;
use image::{ImageBuffer, Rgba};
use nix::sys::memfd::MemFdCreateFlag;
use wayland_client::{
    protocol::{
        wl_buffer::WlBuffer, wl_output::WlOutput, wl_shm::{Format, WlShm},
        wl_shm_pool::WlShmPool,
    },
    Display, GlobalManager, Main,
};
use wayland_protocols::wlr::unstable::screencopy::v1::client::{
    zwlr_screencopy_frame_v1::{Event as FrameEvent, ZwlrScreencopyFrameV1},
    zwlr_screencopy_manager_v1::ZwlrScreencopyManagerV1,
};

/// Shared state between the Wayland event handler and the main loop.
#[derive(Default)]
struct FrameState {
    format: Option<Format>,
    width: u32,
    height: u32,
    stride: u32,
    ready: bool,
    failed: bool,
}

/// Capture one screenshot and save it to ~/Pictures/Screenshots.
fn take_screenshot(screenshots_dir: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Connect to the Wayland display.
    let display = Display::connect_to_env()?;
    let mut event_queue = display.create_event_queue();
    let attached_display = display.attach(event_queue.token());

    // 2. Discover globals (registry round-trip).
    let globals = GlobalManager::new(&attached_display);
    event_queue.sync_roundtrip(&mut (), |_, _, _| {})?;

    // 3. Bind the protocols we need.
    let screencopy_manager: Main<ZwlrScreencopyManagerV1> = globals
        .instantiate_exact(1)
        .map_err(|_| "zwlr_screencopy_manager_v1 not available. \
                Your compositor must support the wlr-screencopy protocol \
                (Sway, Hyprland, River, dwl, etc.)")?;

    let output: Main<WlOutput> = globals
        .instantiate_exact(1)
        .map_err(|_| "No wl_output found")?;

    let shm: Main<WlShm> = globals
        .instantiate_exact(1)
        .map_err(|_| "No wl_shm found")?;

    // 4. Create a screencopy frame for the first output (overlay_cursor = 1).
    let frame: Main<ZwlrScreencopyFrameV1> = screencopy_manager.capture_output(1, &output);

    let state = Arc::new(Mutex::new(FrameState::default()));
    let state_cb = state.clone();

    // 5. Assign the event handler for the frame.
    frame.quick_assign(move |_frame, event, _| {
        let mut s = state_cb.lock().unwrap();
        match event {
            FrameEvent::Buffer {
                format,
                width,
                height,
                stride,
            } => {
                s.format = Some(format);
                s.width = width;
                s.height = height;
                s.stride = stride;
            }
            FrameEvent::Ready { .. } => {
                s.ready = true;
            }
            FrameEvent::Failed => {
                s.failed = true;
            }
            _ => {}
        }
    });

    // 6. Dispatch until we receive the buffer parameters.
    loop {
        let s = state.lock().unwrap();
        if s.format.is_some() || s.failed {
            break;
        }
        drop(s);
        event_queue.dispatch(&mut (), |_, _, _| {})?;
    }

    if state.lock().unwrap().failed {
        return Err("Compositor rejected the screencopy frame.".into());
    }

    let (width, height, stride, format) = {
        let s = state.lock().unwrap();
        (s.width, s.height, s.stride, s.format.unwrap())
    };

    let buf_size = (stride * height) as usize;

    // 7. Create an anonymous in-memory file (memfd) for the shared buffer.
    let name = CStr::from_bytes_with_nul(b"screencopy\0").unwrap();
    let fd = nix::sys::memfd::memfd_create(name, MemFdCreateFlag::MFD_CLOEXEC)?;
    nix::unistd::ftruncate(fd, buf_size as i64)?;

    // Wrap the raw fd so it closes automatically when dropped.
    let mem_file = unsafe { std::fs::File::from_raw_fd(fd) };

    // 8. Create the shm pool and buffer.
    let pool: Main<WlShmPool> = shm.create_pool(mem_file.as_raw_fd(), buf_size as i32);
    let buffer: Main<WlBuffer> = pool.create_buffer(
        0,
        width as i32,
        height as i32,
        stride as i32,
        format,
    );

    // 9. Ask the compositor to copy the screen into our buffer.
    frame.copy(&buffer);

    // 10. Dispatch until the copy is done.
    loop {
        let s = state.lock().unwrap();
        if s.ready || s.failed {
            break;
        }
        drop(s);
        event_queue.dispatch(&mut (), |_, _, _| {})?;
    }

    if state.lock().unwrap().failed {
        return Err("Compositor failed to copy screen data.".into());
    }

    // 11. Memory-map the buffer and convert to an image.
    let mmap = unsafe { memmap2::MmapOptions::new().len(buf_size).map_mut(&mem_file)? };

    // Wayland SHM ARGB8888 / XRGB8888 is little-endian: [B, G, R, A] in memory.
    // The `image` crate expects [R, G, B, A].
    let img: ImageBuffer<Rgba<u8>, Vec<u8>> = if format == Format::Argb8888
        || format == Format::Xrgb8888
    {
        let mut rgba = Vec::with_capacity((width * height * 4) as usize);
        for y in 0..height {
            let row_base = (y * stride) as usize;
            for x in 0..width {
                let i = row_base + (x * 4) as usize;
                let b = mmap[i];
                let g = mmap[i + 1];
                let r = mmap[i + 2];
                let a = mmap[i + 3];
                rgba.extend_from_slice(&[r, g, b, a]);
            }
        }
        ImageBuffer::from_raw(width, height, rgba)
            .ok_or("Failed to create image buffer")?
    } else {
        return Err(format!("Unsupported pixel format: {:?}", format).into());
    };

    // 12. Save to disk.
    let filename = format!("screenshot_{}.png", Local::now().format("%Y-%m-%d_%H-%M-%S"));
    let path = screenshots_dir.join(&filename);
    img.save(&path)?;
    println!("Saved: {}", path.display());

    // Objects are dropped here, cleaning up Wayland resources and closing the memfd.
    Ok(())
}

fn main() {
    // Resolve ~/Pictures/Screenshots.
    let screenshots_dir = dirs::picture_dir()
        .map(|p| p.join("Screenshots"))
        .unwrap_or_else(|| {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
            PathBuf::from(home).join("Pictures/Screenshots")
        });

    fs::create_dir_all(&screenshots_dir).expect("Failed to create screenshots directory");

    println!("Wayland screencopy daemon started.");
    println!("Saving screenshots to: {}", screenshots_dir.display());
    println!("Press Ctrl+C to stop.\n");

    loop {
        if let Err(e) = take_screenshot(&screenshots_dir) {
            eprintln!("Screenshot error: {}", e);
        }
        thread::sleep(Duration::from_secs(60));
    }
}
