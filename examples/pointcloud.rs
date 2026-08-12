//! Point-cloud generation: streams depth + color, generates a Points frame
//! (with color texture), and exports it as PLY or ASCII PCD.
//!
//! Usage:
//!   cargo run --example pointcloud -- <out.ply|out.pcd>   (default out.ply)
//! A frame is captured after the warm-up period and exported once; Ctrl-C to
//! stop streaming afterward.

use realsense2::{
    export_ply, pointcloud, Config, Context, Pipeline, Rs2CameraInfo, Rs2Error, Rs2Format,
    Rs2StreamKind,
};
use std::time::{Duration, Instant};

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

/// Write an ASCII PCD file (unorganized cloud, x y z + fixed gray color).
fn export_pcd(points: &realsense2::Frame, path: &str) -> Result<(), Rs2Error> {
    let vertices = points.vertices();
    let mut out = String::with_capacity(vertices.len() * 32);
    out.push_str(&format!("# .PCD v0.7 - Point Cloud Data file format\n"));
    out.push_str(&format!("VERSION 0.7\n"));
    out.push_str(&format!("FIELDS x y z rgb\n"));
    out.push_str(&format!("SIZE 4 4 4 4\n"));
    out.push_str(&format!("TYPE F F F F\n"));
    out.push_str(&format!("COUNT 1 1 1 1\n"));
    out.push_str(&format!("WIDTH {}\n", vertices.len()));
    out.push_str(&format!("HEIGHT 1\n"));
    out.push_str(&format!("VIEWPOINT 0 0 0 1 0 0 0\n"));
    out.push_str(&format!("POINTS {}\n", vertices.len()));
    out.push_str(&format!("DATA ascii\n"));

    let gray = 0x808080u32; // medium gray RGB
    for v in vertices.iter() {
        out.push_str(&format!(
            "{:.6} {:.6} {:.6} {}\n",
            v.xyz[0],
            v.xyz[1],
            v.xyz[2],
            gray
        ));
    }
    std::fs::write(path, out).map_err(|e| Rs2Error {
        message: format!("failed to write {}: {}", path, e),
        function: "export_pcd".into(),
        args: "".into(),
    })?;
    Ok(())
}

fn main() -> Result<(), Rs2Error> {
    let out_path = std::env::args()
        .nth(2)
        .unwrap_or_else(|| "out.ply".to_string());

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
    config.enable_stream(Rs2StreamKind::Depth, None, 640, 480, Rs2Format::Z16, 30)?;
    config.enable_stream(Rs2StreamKind::Color, None, 640, 480, Rs2Format::Rgb8, 30)?;
    pipeline.start_with_config(Some(&config))?;

    let pc = pointcloud()?;
    println!("Generating point cloud -> {} (streaming...)", out_path);

    let start = Instant::now();
    let mut exported = false;

    loop {
        let frameset = pipeline.wait_for_frames(5000)?;
        let depth = frameset.frames_of_type(Rs2StreamKind::Depth);
        let color = frameset.frames_of_type(Rs2StreamKind::Color);
        let (Some(depth_frame), Some(color_frame)) = (depth.first(), color.first()) else {
            continue;
        };

        // Feed the whole frameset so the point cloud gets color texture.
        pc.process_frameset(&frameset)?;
        let Some(points_frame) = pc.output_points(5000)? else {
            continue;
        };

        if start.elapsed() >= Duration::from_millis(2000) && !exported {
            let n = points_frame.points_count();
            let vertices = points_frame.vertices();
            let non_zero = vertices.iter().filter(|v| v.xyz.iter().all(|c| *c != 0.0)).count();
            println!(
                "Captured frame: {}x{} depth, {} points ({} valid)",
                depth_frame.width(),
                depth_frame.height(),
                n,
                non_zero
            );

            if out_path.ends_with(".pcd") {
                export_pcd(&points_frame, &out_path)?;
            } else {
                export_ply(&points_frame, &out_path, Some(color_frame))?;
            }
            println!("Saved to {}", out_path);
            exported = true;
        }
    }
}
