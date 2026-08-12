//! Depth-to-color alignment: streams depth + color and aligns depth onto the
//! color sensor's viewpoint. After alignment, the depth frame has the same
//! resolution as the color stream, so depth pixels correspond 1:1 to color
//! pixels (useful for OCR / object detection overlays).
//!
//! Usage: `cargo run --example align_stream [-- WIDTHxHEIGHT@FPS]`

use realsense2::{
    align_to_color, Config, Context, Pipeline, Rs2CameraInfo, Rs2Error, Rs2Format, Rs2StreamKind,
};
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

/// Center-pixel depth in mm (NaN = invalid).
fn center_distance_mm(frame: &realsense2::Frame) -> f32 {
    let units = frame.depth_units().unwrap_or(0.001);
    let data = frame.data();
    let w = frame.width().max(0) as usize;
    let h = frame.height().max(0) as usize;
    if w == 0 || h == 0 || data.len() < w * h * 2 {
        return f32::NAN;
    }
    let cx = w / 2;
    let cy = h / 2;
    let idx = (cy * w + cx) * 2;
    let raw = u16::from_le_bytes([data[idx], data[idx + 1]]);
    if raw == 0 {
        f32::NAN
    } else {
        raw as f32 * units * 1000.0
    }
}

fn fmt_mm(v: f32) -> String {
    if v.is_nan() {
        "invalid".to_string()
    } else {
        format!("{:>6.1} mm", v)
    }
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
    let serial = pick_usb3_device(&devices);

    let mut pipeline = Pipeline::new(&context)?;
    let mut config = Config::new()?;
    if let Some(s) = serial.as_deref() {
        config.enable_device(s)?;
    }
    config.enable_stream(Rs2StreamKind::Depth, None, width, height, Rs2Format::Z16, fps)?;
    config.enable_stream(Rs2StreamKind::Color, None, width, height, Rs2Format::Rgb8, fps)?;
    pipeline.start_with_config(Some(&config))?;

    let align = align_to_color()?;
    println!("Depth+Color {}x{}@{} | align depth -> color", width, height, fps);

    let mut last_print = Instant::now();
    let mut frames = 0u32;
    let start = Instant::now();
    let mut consecutive_timeouts = 0u32;

    loop {
        let frameset = match pipeline.wait_for_frames(5000) {
            Ok(f) => f,
            Err(e) if e.function.contains("wait_for_frames") => {
                consecutive_timeouts += 1;
                eprintln!("[warn] frame timeout ({}/3)", consecutive_timeouts);
                if consecutive_timeouts >= 3 {
                    return Err(e);
                }
                continue;
            }
            Err(e) => return Err(e),
        };

        let depth = frameset.frames_of_type(Rs2StreamKind::Depth);
        let color = frameset.frames_of_type(Rs2StreamKind::Color);
        let (Some(depth_frame), Some(color_frame)) = (depth.first(), color.first()) else {
            continue;
        };

        align.process_frameset(&frameset)?;
        let Some(aligned) = align.output(5000)? else {
            continue;
        };

        frames += 1;
        if last_print.elapsed() >= Duration::from_millis(1000) {
            let raw_mm = center_distance_mm(depth_frame);
            let aligned_mm = center_distance_mm(&aligned);
            let elapsed = start.elapsed().as_secs_f64();
            println!(
                "[{:.0}s] color {}x{} | raw depth {}x{} center {} | aligned depth {}x{} center {} ({} fps)",
                elapsed,
                color_frame.width(),
                color_frame.height(),
                depth_frame.width(),
                depth_frame.height(),
                fmt_mm(raw_mm),
                aligned.width(),
                aligned.height(),
                fmt_mm(aligned_mm),
                frames as f64 / elapsed
            );
            last_print = Instant::now();
        }
    }
}
