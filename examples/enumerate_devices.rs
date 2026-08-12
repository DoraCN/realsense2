use realsense2::{Context, Rs2CameraInfo, Rs2Error};

fn info_or(device: &realsense2::Device, info: Rs2CameraInfo, fallback: &str) -> String {
    device.info(info).unwrap_or_else(|| fallback.to_string())
}

fn main() -> Result<(), Rs2Error> {
    println!("----\nEnumerating all devices compatible with RealSense:\n----");
    let context = Context::new()?;
    let devices = context.query_devices()?;
    if devices.is_empty() {
        println!("No devices found");
        return Ok(());
    }
    for device in devices.iter() {
        let name = info_or(&device, Rs2CameraInfo::Name, "N/A");
        let sn = info_or(&device, Rs2CameraInfo::SerialNumber, "N/A");
        let fw = info_or(&device, Rs2CameraInfo::FirmwareVersion, "N/A");
        let usb = info_or(&device, Rs2CameraInfo::UsbTypeDescriptor, "N/A");
        println!(
            ">  {:25} | SN: {:15} | FW: {:15} | USB: {}",
            name, sn, fw, usb
        );
    }
    println!("---");
    Ok(())
}
