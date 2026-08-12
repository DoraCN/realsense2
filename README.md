# realsense2

Safe Rust bindings for the [Intel RealSense SDK 2.0](https://github.com/realsenseai/librealsense)
(librealsense2), using **hand-written FFI** — no dependency on the third-party
`realsense-rust` crate.

## Features

- Hand-written `extern "C"` bindings to the librealsense2 C API (see `src/ffi.rs`)
- Safe, RAII-based wrappers: `Context`, `Device`, `DeviceList`, `Config`,
  `Pipeline`, `Frame`, `FrameSet`, `StreamProfile`
- Enum types mirroring the C headers (`Rs2StreamKind`, `Rs2Format`, `Rs2CameraInfo`,
  `Rs2Option`, …)
- Post-processing: depth filter chain, depth-to-color alignment, point cloud
- No bindgen / clang required — builds with just a C linker

## Requirements

- System-installed **librealsense2** (`librealsense2.so` + headers)
- `pkg-config` (optional, used to locate the library)

The build script (`build.rs`) locates the library via `pkg-config realsense2`
or by searching `/usr/local/lib` and common distro paths.

Install the SDK on a Jetson (L4T / Tegra):

```bash
# RSUSB backend (no kernel patching, recommended on Jetson)
cd librealsense
mkdir build && cd build
cmake .. -DCMAKE_BUILD_TYPE=Release -DFORCE_RSUSB_BACKEND=true -DBUILD_WITH_CUDA=true
make -j$(($(nproc)-1)) && sudo make install && sudo ldconfig
```

See `docs/install.md` for full installation instructions (apt / from source /
Jetson).

## Usage

```rust
use realsense2::{Config, Context, Pipeline, Rs2Format, Rs2StreamKind};

fn main() -> Result<(), realsense2::Rs2Error> {
    let context = Context::new()?;
    let devices = context.query_devices()?;
    assert!(!devices.is_empty(), "no device");

    let mut pipeline = Pipeline::new(&context)?;
    let mut config = Config::new()?;
    config.enable_stream(Rs2StreamKind::Depth, None, 640, 480, Rs2Format::Z16, 30)?;
    pipeline.start_with_config(Some(&config))?;

    loop {
        let frames = pipeline.wait_for_frames(5000)?;
        for depth in frames.frames_of_type(Rs2StreamKind::Depth) {
            let units = depth.depth_units().unwrap_or(0.001);
            println!("{}x{} units={}", depth.width(), depth.height(), units);
        }
    }
}
```

## Examples

```bash
cargo run --example enumerate_devices     # list connected cameras
cargo run --example depth_stream          # center-pixel distance, 640x480@30
cargo run --example depth_stream -- 848x480@60
cargo run --example filter_chain          # Decimation->Spatial->Temporal->HoleFill
cargo run --example align_stream          # depth aligned onto color viewpoint
cargo run --example pointcloud -- out.ply # point-cloud export (PLY or PCD)
cargo run --example benchmark             # fps across resolutions (--color for d+c)
```

## Post-processing

Depth filters, alignment, and point-cloud generation are available through the
`processing` module, built on librealsense processing blocks with an internal
frame queue (no Rust callbacks):

```rust
use realsense2::{decimation, spatial, temporal, hole_filling, align_to_color,
                 pointcloud, export_ply, Rs2Option};

let decim = decimation()?;
decim.set_option(Rs2Option::FilterMagnitude, 2.0)?; // 2x downsample

// push a depth frame through the chain, one stage at a time:
decim.process_frame(&depth_frame)?;
let filtered = decim.output(5000)?.unwrap();

// align depth onto color (needs full frameset):
let align = align_to_color()?;
align.process_frameset(&frameset)?;
let aligned = align.output(5000)?.unwrap();

// point cloud (also needs full frameset for texture):
let pc = pointcloud()?;
pc.process_frameset(&frameset)?;
let points = pc.output(5000)?.unwrap();
export_ply(&points, "cloud.ply", Some(&color_frame))?;
```

## License

Apache-2.0
