//! Depth post-processing filter chain:
//! Decimation -> Spatial -> Temporal -> Hole Filling.
//!
//! Prints the center-pixel distance before/after filtering and the
//! validity ratio (fraction of non-zero depth pixels) for comparison.
//!
//! Usage: `cargo run --example filter_chain [-- WIDTHxHEIGHT@FPS]`

use realsense2::{
    decimation, hole_filling, spatial, temporal, Config, Context, Pipeline, ProcessingBlock,
    Rs2CameraInfo, Rs2Error, Rs2Format, Rs2Option, Rs2StreamKind,
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

/// Pick the first USB3-connected device.
fn pick_usb3_device(devices: &realsense2::DeviceList) -> (Option<String>, String) {
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
        if best.as_ref().map_or(true, |(b, ..)| usb > *b) {
            best = Some((usb, serial.clone().unwrap_or_default(), name));
        }
    }
    let (_, serial, name) = best.unwrap();
    (Some(serial), name)
}

/// Center-pixel distance in mm and validity ratio of a Z16 depth frame.
fn analyze(frame: &realsense2::Frame) -> (f32, f32) {
    let units = frame.depth_units().unwrap_or(0.001);
    let data = frame.data();
    let w = frame.width().max(0) as usize;
    let h = frame.height().max(0) as usize;
    if w == 0 || h == 0 || data.len() < w * h * 2 {
        return (f32::NAN, 0.0);
    }
    let cx = w / 2;
    let cy = h / 2;
    let idx = (cy * w + cx) * 2;
    let center = u16::from_le_bytes([data[idx], data[idx + 1]]);
    let center_mm = if center == 0 {
        f32::NAN
    } else {
        center as f32 * units * 1000.0
    };

    // validity ratio: pixels with depth > 0
    let mut valid = 0usize;
    for chunk in data.chunks_exact(2) {
        if u16::from_le_bytes([chunk[0], chunk[1]]) > 0 {
            valid += 1;
        }
    }
    (center_mm, valid as f32 / (w * h) as f32)
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
    let (serial, name) = pick_usb3_device(&devices);
    println!("Device: {} (SN {})", name, serial.as_deref().unwrap_or("?"));

    let mut pipeline = Pipeline::new(&context)?;
    let mut config = Config::new()?;
    if let Some(s) = serial.as_deref() {
        config.enable_device(s)?;
    }
    config.enable_stream(Rs2StreamKind::Depth, None, width, height, Rs2Format::Z16, fps)?;
    pipeline.start_with_config(Some(&config))?;

    // Build the filter chain.
    let decim = decimation()?;
    let spatial = spatial()?;
    let temporal = temporal()?;
    let holes = hole_filling()?;

    // Tune typical filter parameters.
    decim.set_option(Rs2Option::FilterMagnitude, 2.0)?;
    spatial.set_option(Rs2Option::FilterMagnitude, 2.0)?;
    spatial.set_option(Rs2Option::FilterSmoothAlpha, 0.5)?;
    spatial.set_option(Rs2Option::FilterSmoothDelta, 20.0)?;
    temporal.set_option(Rs2Option::FilterSmoothAlpha, 0.4)?;
    temporal.set_option(Rs2Option::FilterSmoothDelta, 20.0)?;
    holes.set_option(Rs2Option::HolesFill, 1.0)?;

    println!(
        "Filter chain: Decimation(2x) -> Spatial -> Temporal -> HoleFill, depth {}x{}@{}",
        width, height, fps
    );

    let mut last_print = Instant::now();
    let mut frames = 0u32;
    let start = Instant::now();

    loop {
        let frameset = pipeline.wait_for_frames(5000)?;
        let depth = frameset.frames_of_type(Rs2StreamKind::Depth);
        let Some(frame) = depth.first() else {
            continue;
        };

        let raw_analysis = analyze(frame);
        let Some(final_frame) = run_chain(frame, &decim, &spatial, &temporal, &holes)? else {
            continue;
        };

        frames += 1;
        if last_print.elapsed() >= Duration::from_millis(1000) {
            let (filtered_center, filtered_valid) = analyze(&final_frame);
            let (raw_center, raw_valid) = raw_analysis;
            let center_str = |v: f32| {
                if v.is_nan() {
                    "invalid".to_string()
                } else {
                    format!("{:>6.1} mm", v)
                }
            };
            let elapsed = start.elapsed().as_secs_f64();
            println!(
                "[{:.0}s] raw: center {}  valid {:>4.1}%  |  filtered: center {}  valid {:>4.1}%  ({} fps)",
                elapsed,
                center_str(raw_center),
                raw_valid * 100.0,
                center_str(filtered_center),
                filtered_valid * 100.0,
                frames as f64 / elapsed
            );
            last_print = Instant::now();
        }
    }
}

/// Push a depth frame through the whole filter chain, returning the final
/// filtered frame. Returns `None` if any stage fails to produce output.
fn run_chain(
    frame: &realsense2::Frame,
    decim: &ProcessingBlock,
    spatial: &ProcessingBlock,
    temporal: &ProcessingBlock,
    holes: &ProcessingBlock,
) -> Result<Option<realsense2::Frame>, Rs2Error> {
    decim.process_frame(frame)?;
    let Some(decim_frame) = decim.output(5000)? else {
        return Ok(None);
    };

    spatial.process_frame(&decim_frame)?;
    let Some(spatial_frame) = spatial.output(5000)? else {
        return Ok(None);
    };

    temporal.process_frame(&spatial_frame)?;
    let Some(temporal_frame) = temporal.output(5000)? else {
        return Ok(None);
    };

    holes.process_frame(&temporal_frame)?;
    holes.output(5000)
}
