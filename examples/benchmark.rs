//! Frame-rate / resolution benchmark for depth (and optional color) streams.
//!
//! Tries a set of common RealSense D400 configurations, streams each for a
//! few seconds, and reports the measured frame rate. Useful for picking the
//! right resolution for your application (e.g. 90 fps short-range scans vs
//! 30 fps for alignment/OCR).
//!
//! Usage:
//!   cargo run --example benchmark                 # depth only
//!   cargo run --example benchmark -- --color      # depth + color

use realsense2::{Config, Context, Pipeline, Rs2CameraInfo, Rs2Error, Rs2Format, Rs2StreamKind};
use std::time::{Duration, Instant};

#[derive(Clone, Copy)]
struct Preset {
    w: i32,
    h: i32,
    fps: i32,
}

const DEPTH_PRESETS: &[Preset] = &[
    Preset { w: 1280, h: 720, fps: 30 },
    Preset { w: 848, h: 480, fps: 90 },
    Preset { w: 848, h: 480, fps: 60 },
    Preset { w: 848, h: 480, fps: 30 },
    Preset { w: 640, h: 480, fps: 90 },
    Preset { w: 640, h: 480, fps: 60 },
    Preset { w: 640, h: 480, fps: 30 },
    Preset { w: 640, h: 360, fps: 60 },
    Preset { w: 640, h: 360, fps: 30 },
    Preset { w: 480, h: 270, fps: 60 },
    Preset { w: 424, h: 240, fps: 90 },
    Preset { w: 256, h: 144, fps: 90 },
];

fn pick_usb3_device(devices: &realsense2::DeviceList) -> Option<String> {
    let mut best: Option<(f32, String)> = None;
    for device in devices.iter() {
        let serial = device.info(Rs2CameraInfo::SerialNumber);
        let usb = device
            .info(Rs2CameraInfo::UsbTypeDescriptor)
            .and_then(|s| s.parse::<f32>().ok())
            .unwrap_or(0.0);
        if best.as_ref().map_or(true, |(b, _)| usb > *b) {
            best = Some((usb, serial.clone().unwrap_or_default()));
        }
    }
    best.map(|(_, s)| s)
}

/// Stream one config for `seconds`, return measured fps.
fn measure(
    context: &Context,
    serial: Option<&str>,
    preset: &Preset,
    with_color: bool,
    seconds: f32,
) -> Result<Option<f32>, Rs2Error> {
    let mut pipeline = Pipeline::new(context)?;
    let mut config = Config::new()?;
    if let Some(s) = serial {
        config.enable_device(s)?;
    }
    config.enable_stream(
        Rs2StreamKind::Depth,
        None,
        preset.w,
        preset.h,
        Rs2Format::Z16,
        preset.fps,
    )?;
    if with_color {
        config.enable_stream(
            Rs2StreamKind::Color,
            None,
            preset.w,
            preset.h,
            Rs2Format::Rgb8,
            preset.fps,
        )?;
    }
    if let Err(e) = pipeline.start_with_config(Some(&config)) {
        eprintln!("      (unsupported: {})", e.message);
        return Ok(None);
    }

    let mut frames = 0u32;
    let start = Instant::now();
    let duration = Duration::from_secs_f32(seconds);
    while start.elapsed() < duration {
        match pipeline.wait_for_frames(3000) {
            Ok(_) => frames += 1,
            Err(e) if e.function.contains("wait_for_frames") => {
                // timeout during measurement: count as no frames
                break;
            }
            Err(e) => return Err(e),
        }
    }
    let elapsed = start.elapsed().as_secs_f64();
    let _ = pipeline.stop();
    Ok(Some(frames as f32 / elapsed as f32))
}

fn main() -> Result<(), Rs2Error> {
    let with_color = std::env::args().any(|a| a == "--color");

    let context = Context::new()?;
    let devices = context.query_devices()?;
    if devices.is_empty() {
        eprintln!("No devices found");
        return Ok(());
    }
    let serial = pick_usb3_device(&devices);
    println!(
        "Benchmark depth{} on D400-class device (SN {})",
        if with_color { "+color" } else { "" },
        serial.as_deref().unwrap_or("?")
    );
    println!("{:<16} {:>10} {:>10}", "resolution", "requested", "measured");
    println!("{}", "-".repeat(40));

    for preset in DEPTH_PRESETS {
        let label = format!("{}x{}@{}", preset.w, preset.h, preset.fps);
        match measure(&context, serial.as_deref(), preset, with_color, 2.0)? {
            Some(fps) => println!(
                "{:<16} {:>8} fps {:>8.1} fps",
                label, preset.fps, fps
            ),
            None => println!("{:<16} {:>10} {}", label, "-", "N/A"),
        }
    }
    Ok(())
}
