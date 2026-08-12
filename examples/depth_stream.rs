//! Stream depth frames and print the distance at the center pixel.
//!
//! Usage: `cargo run --example depth_stream [-- WIDTHxHEIGHT@FPS]`
//! Optional argument as WIDTHxHEIGHT@FPS, default 640x480@30.

use realsense2::{Config, Context, Pipeline, Rs2CameraInfo, Rs2Error, Rs2Format, Rs2StreamKind};
use std::time::{Duration, Instant};

fn parse_resolution(arg: &str) -> Result<(i32, i32, i32), String> {
    let (res, fps) = arg.split_once('@').ok_or("expected WIDTHxHEIGHT@FPS")?;
    let (w, h) = res.split_once('x').ok_or("expected WIDTHxHEIGHT")?;
    Ok((
        w.parse().map_err(|_| "bad width")?,
        h.parse().map_err(|_| "bad height")?,
        fps.parse().map_err(|_| "bad fps")?,
    ))
}

fn main() -> Result<(), Rs2Error> {
    let (width, height, fps) = match std::env::args().nth(2) {
        Some(arg) => {
            let (w, h, f) = parse_resolution(&arg).map_err(|e| Rs2Error {
                message: e,
                function: "args".into(),
                args: "".into(),
            })?;
            (w, h, f)
        }
        None => (640, 480, 30),
    };

    let context = Context::new()?;
    let devices = context.query_devices()?;
    if devices.is_empty() {
        eprintln!("No devices found");
        return Ok(());
    }
    let name = devices
        .get(0)
        .and_then(|d| d.info(Rs2CameraInfo::Name))
        .unwrap_or_else(|| "<unknown>".into());
    println!("Streaming depth {}x{}@{} from {}", width, height, fps, name);

    let mut pipeline = Pipeline::new(&context)?;
    let mut config = Config::new()?;
    config.enable_stream(
        Rs2StreamKind::Depth,
        None,
        width,
        height,
        Rs2Format::Z16,
        fps,
    )?;
    pipeline.start_with_config(Some(&config))?;

    let mut last_print = Instant::now();
    let mut frame_count = 0u32;
    let start = Instant::now();

    loop {
        let frameset = pipeline.wait_for_frames(5000)?;
        let depth = frameset.frames_of_type(Rs2StreamKind::Depth);
        if let Some(frame) = depth.first() {
            frame_count += 1;
            if last_print.elapsed() >= Duration::from_millis(500) {
                let units = frame.depth_units().unwrap_or(0.001);
                let data = frame.data();
                let w = frame.width().max(0) as usize;
                let h = frame.height().max(0) as usize;
                let cx = w / 2;
                let cy = h / 2;
                let idx = (cy * w + cx) * 2;
                let distance = if data.len() >= idx + 2 && w > 0 && h > 0 {
                    let raw = u16::from_le_bytes([data[idx], data[idx + 1]]);
                    raw as f32 * units * 1000.0 // mm
                } else {
                    f32::NAN
                };
                let elapsed = start.elapsed().as_secs_f64();
                let fps_measured = frame_count as f64 / elapsed;
                println!(
                    "frame #{} | {}x{} | center distance: {:>7.1} mm | ~{:.1} fps",
                    frame.frame_number(),
                    frame.width(),
                    frame.height(),
                    distance,
                    fps_measured
                );
                last_print = Instant::now();
            }
        }
    }
}
