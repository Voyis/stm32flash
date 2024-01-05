//! Firmware Flash Utility
//!
//! This tool is used to flash Voyis microcontrollers through serial ports, 232 or 485. It contains an embedded modified version of the stm32 flash utility.
//!


use clap::Parser;
use std::fs::OpenOptions;
use std::io::{Write, Seek};

//const STM32FLASH_BINARY: &[u8; 570365] = std::include_bytes!("../stm32flash");
const FORCE_BOOTLOADER: [u8; 17] = [
0x32, 0x47, 0x20, 0x0c, 0x00, 0x06, 0xdf, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x0c, 0x26,
0x44,
];
const BAUDRATE: u32 = 115_200;


/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the serial port
    port_name: String,
    
    /// hex file to load
    hex_file: String,
    
    /// path to the GPIO we need to toggle for 485 comms.
    gpio_path: Option<String>
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    let args = Args::parse();
    
    // If we are using a 485, put the system into bootloader mode.
    if let Some(gpio_path) = args.gpio_path {
        let mut port = match serialport::new(
            args.port_name,
            BAUDRATE,
        )
        .open()
        {
            Ok(val) => val,
            Err(e) => {
                return Err(format!("Failed to open serial port: {}", e).into());
            }
        };   
        
        println!("Setting the system into bootloader mode.");
        let mut gpio_file = OpenOptions::new().write(true).open(gpio_path)?;
        gpio_file.write(&['1' as u8])?;
        gpio_file.rewind()?;
        port.write(&FORCE_BOOTLOADER)
            .expect("Failed to write set bootloader mode to port.");
        std::thread::sleep(std::time::Duration::from_micros((FORCE_BOOTLOADER.len() as f32 * 10.0/BAUDRATE as f32 * 1000000.0) as u64));
        gpio_file.write(&['0' as u8])?;
        //allow time for the board to reset
        std::thread::sleep(std::time::Duration::new(2, 0));
    }

    //Extract the stm32flash utility

    //Run the stm32flash tool with the appropriate commands.
    
    return Ok(());
}
