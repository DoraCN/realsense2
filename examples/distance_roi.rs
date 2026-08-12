//! Distance measurement in the center ROI of the depth frame, for a
//! camera selected by serial number.
//!
//! Usage:
//!   cargo run --example distance_roi -- --serial <SN>
//!   cargo run --example distance_roi -- --serial <SN> --roi-frac 0.5
//! `--roi-frac F` sets the ROI size as a fraction of the frame width/height
//! (default 0.5 -> central 50% x 50%). Prints per-frame center-ROI stats:
//! valid ratio, and min / mean / max distance in mm.

use realsense2::{Config, Context, Pipeline, Rs2Error, Rs2Format, Rs2StreamKind};
use std::time::{Duration, Instant};

fn main() -> Result<(), Rs2Error> {
    let mut serial: Option<String> = None;
    let mut roi_frac = 0.5f32;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--serial" => {
                serial = args.next();
                if serial.is_none() {
                    eprintln!("--serial requires a value");
                    std::process::exit(2);
                }
            }
            "--roi-frac" => {
                let v = args.next().and_then(|s| s.parse().ok());
                match v {
                    Some(f) if (0.0..=1.0).contains(&f) => roi_frac = f,
                    _ => {
                        eprintln!("--roi-frac requires a number in [0,1]");
                        std::process::exit(2);
                    }
                }
            }
            _ => {
                eprintln!("unknown argument: {}", arg);
                std::process::exit(2);
            }
        }
    }
    let Some(sn) = serial.as_deref() else {
        eprintln!("--serial is required");
        std::process::exit(2);
    };

    let context = Context::new()?;
    let devices = context.query_devices()?;
    if devices.is_empty() {
        eprintln!("No devices found");
        return Ok(());
    }

    let mut pipeline = Pipeline::new(&context)?;
    let mut config = Config::new()?;
    config.enable_device(sn)?;
    config.enable_stream(Rs2StreamKind::Depth, None, 640, 480, Rs2Format::Z16, 30)?;
    if let Err(e) = pipeline.start_with_config(Some(&config)) {
        eprintln!("Failed to start pipeline for SN {}: {}", sn, e.message);
        std::process::exit(1);
    }
    println!("Distance in center ROI ({}x{} of frame) | SN {}", roi_frac, roi_frac, sn);

    let mut last_print = Instant::now();
    let mut frames = 0u32;
    let start = Instant::now();

    loop {
        let frameset = pipeline.wait_for_frames(5000)?;
        let depth = frameset.frames_of_type(Rs2StreamKind::Depth);
        let Some(frame) = depth.first() else {
            continue;
        };

        frames += 1;
        if last_print.elapsed() >= Duration::from_millis(500) {
            let stats = roi_stats(frame, roi_frac);
            let elapsed = start.elapsed().as_secs_f64();
            match stats {
                Some(s) => println!(
                    "[{:.1}s] frame #{} | ROI {}x{} @ center | valid {:>5.1}% | min {:>7.1} mm | mean {:>7.1} mm | max {:>7.1} mm | ~{:.1} fps",
                    elapsed,
                    frame.frame_number(),
                    s.w, s.h,
                    s.valid_ratio * 100.0,
                    s.min_mm,
                    s.mean_mm,
                    s.max_mm,
                    frames as f64 / elapsed
                ),
                None => println!("[warn] empty ROI (all pixels invalid)"),
            }
            last_print = Instant::now();
        }
    }
}

struct RoiStats {
    w: usize,
    h: usize,
    valid_ratio: f32,
    min_mm: f32,
    mean_mm: f32,
    max_mm: f32,
}

/// Compute min/mean/max distance over the central `frac x frac` region of a
/// Z16 depth frame. Returns `None` if the whole ROI is invalid.
fn roi_stats(frame: &realsense2::Frame, frac: f32) -> Option<RoiStats> {
    let units = frame.depth_units().unwrap_or(0.001);
    let data = frame.data();
    let w = frame.width().max(0) as usize;
    let h = frame.height().max(0) as usize;
    if w == 0 || h == 0 || data.len() < w * h * 2 {
        return None;
    }

    // Central ROI.
    let roi_w = ((w as f32 * frac).round() as usize).max(1);
    let roi_h = ((h as f32 * frac).round() as usize).max(1);
    let x0 = (w - roi_w) / 2;
    let y0 = (h - roi_h) / 2;

    let mut sum_mm = 0.0f64;
    let mut min_mm = f32::MAX;
    let mut max_mm = f32::MIN;
    let mut valid = 0usize;

    for row in y0..(y0 + roi_h) {
        let base = row * w;
        for col in x0..(x0 + roi_w) {
            let idx = (base + col) * 2;
            let raw = u16::from_le_bytes([data[idx], data[idx + 1]]);
            if raw == 0 {
                continue;
            }
            let mm = raw as f32 * units * 1000.0;
            sum_mm += mm as f64;
            if mm < min_mm {
                min_mm = mm;
            }
            if mm > max_mm {
                max_mm = mm;
            }
            valid += 1;
        }
    }

    let total = roi_w * roi_h;
    if valid == 0 {
        return None;
    }
    Some(RoiStats {
        w: roi_w,
        h: roi_h,
        valid_ratio: valid as f32 / total as f32,
        min_mm,
        mean_mm: (sum_mm / valid as f64) as f32,
        max_mm,
    })
}
