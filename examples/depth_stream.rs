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
    let (width, height, fps) = match std::env::args().nth(1) {
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

    // Prefer a USB3-connected device: USB2 cannot sustain 30 fps depth.
    let (serial, name) = pick_best_device(&devices);
    println!(
        "Streaming depth {}x{}@{} from {} (SN {})",
        width,
        height,
        fps,
        name,
        serial.as_deref().unwrap_or("<unknown>")
    );

    let mut pipeline = Pipeline::new(&context)?;
    let mut config = Config::new()?;
    if let Some(s) = serial.as_deref() {
        config.enable_device(s)?;
    }
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
    let mut consecutive_timeouts = 0u32;

    loop {
        let frameset = match pipeline.wait_for_frames(5000) {
            Ok(f) => f,
            Err(Rs2Error {
                function, message, ..
            }) if function.contains("wait_for_frames") => {
                consecutive_timeouts += 1;
                eprintln!("[warn] {} ({}/3)", message, consecutive_timeouts);
                if consecutive_timeouts >= 3 {
                    return Err(Rs2Error {
                        message: format!("streaming stalled: {}", message),
                        function,
                        args: "".into(),
                    });
                }
                continue;
            }
            Err(e) => return Err(e),
        };
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
                    if raw == 0 {
                        f32::NAN // invalid pixel (no depth)
                    } else {
                        raw as f32 * units * 1000.0 // mm
                    }
                } else {
                    f32::NAN
                };
                let elapsed = start.elapsed().as_secs_f64();
                let fps_measured = frame_count as f64 / elapsed;
                let dist_str = if distance.is_nan() {
                    "  invalid".to_string()
                } else {
                    format!("{:>7.1} mm", distance)
                };
                println!(
                    "frame #{} | {}x{} | center distance: {} | ~{:.1} fps",
                    frame.frame_number(),
                    frame.width(),
                    frame.height(),
                    dist_str,
                    fps_measured
                );
                last_print = Instant::now();
            }
        }
    }
}

/// Pick the first device; prefer USB3 over USB2.
fn pick_best_device(devices: &realsense2::DeviceList) -> (Option<String>, String) {
    let mut best: Option<(f32, String, String)> = None;
    for device in devices.iter() {
        let serial = device.info(Rs2CameraInfo::SerialNumber);
        let name = device
            .info(Rs2CameraInfo::Name)
            .unwrap_or_else(|| "<unknown>".into());
        let usb = device
            .info(Rs2CameraInfo::UsbTypeDescriptor)
            .and_then(|s| s.parse::<f32>().ok())
            .unwrap_or(0.0);
        let better = match &best {
            None => true,
            Some((b_usb, ..)) => usb > *b_usb,
        };
        if better {
            best = Some((usb, serial.clone().unwrap_or_default(), name));
        }
    }
    let (_, serial, name) = best.unwrap();
    (Some(serial), name)
}
