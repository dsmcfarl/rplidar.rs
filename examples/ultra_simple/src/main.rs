extern crate rplidar_drv;
extern crate rpos_drv;
extern crate serialport;
extern crate hex_slice;

use hex_slice::AsHex;

use rplidar_drv::{RplidarDevice, RplidarProtocol, ScanOptions};
use rpos_drv::{ ErrorKind, Channel };
use serialport::prelude::*;
use std::time::Duration;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("Usage: {} <serial_port>", args[0]);
        return;
    }

    let serial_port = &args[1];

    let s = SerialPortSettings {
        baud_rate: 115200,
        data_bits: DataBits::Eight,
        flow_control: FlowControl::None,
        parity: Parity::None,
        stop_bits: StopBits::One,
        timeout: Duration::from_millis(1),
    };

    let mut serial_port =
        serialport::open_with_settings(serial_port, &s).expect("failed to open serial port");

    serial_port.write_data_terminal_ready(false)
        .expect("failed to clear DTR");

    let channel = Channel::<RplidarProtocol, serialport::SerialPort>::new(
        RplidarProtocol::new(),
        serial_port,
    );

    let mut rplidar = RplidarDevice::new(channel);

    let device_info = rplidar
        .get_device_info()
        .expect("failed to get device info");

    println!("Connected to LIDAR: ");
    println!("    Model: {}", device_info.model);
    println!(
        "    Firmware Version: {}.{}",
        device_info.firmware_version >> 8,
        device_info.firmware_version & 0xff
    );
    println!("    Hardware Version: {}", device_info.hardware_version);
    println!("    Serial Number: {:02X}", device_info.serialnum.plain_hex(false));

    let all_supported_scan_modes = rplidar
        .get_all_supported_scan_modes()
        .expect("failed to get all supported scan modes");
    
    println!("All supported scan modes:");
    for scan_mode in all_supported_scan_modes {
        println!(
            "    {:2} {:16}: Max Distance: {:6.2}m, Ans Type: {:02X}, Us per sample: {:.2}us",
            scan_mode.id,
            scan_mode.name,
            scan_mode.max_distance,
            scan_mode.ans_type,
            scan_mode.us_per_sample
        );
    }

    let typical_scan_mode = rplidar
        .get_typical_scan_mode()
        .expect("failed to get typical scan mode");

    println!("Typical scan mode: {}", typical_scan_mode);

    rplidar.set_motor_pwm(600)
        .expect("failed to start motor");

    println!("Starting LIDAR in typical mode...");

    let actual_mode = rplidar.start_scan()
        .expect("failed to start scan in standard mode");
    
    println!("Started scan in mode `{}`", actual_mode.name);

    loop {
        match rplidar.grab_scan_point() {
            Ok(scan_point) => println!("Angle: {:5.2}, Distance: {:8.4}, Valid: {:5}, Sync: {:5}", scan_point.angle(), scan_point.distance(), scan_point.is_valid(), scan_point.is_sync()),
            Err(err) => {
                if err.kind() == ErrorKind::OperationTimeout {
                    continue;
                } else {
                    println!("Error: {:?}", err);
                    break;
                }
            }
        }
    }
}