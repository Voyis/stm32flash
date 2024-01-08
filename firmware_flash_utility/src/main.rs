//! Firmware Flash Utility
//!
//! This tool is used to flash Voyis microcontrollers through serial ports, 232 or 485. It contains an embedded modified version of the stm32 flash utility.
//!


use clap::Parser;
use std::fs::OpenOptions;
use std::io::{Write, Seek};
use tempfile::tempdir;
use std::env;
use std::os::unix::fs::PermissionsExt;

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
    
    /// Is this doing RS485 programming?
    #[arg(short, long)]
    rs485: bool
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[allow(non_snake_case)]
    let STM32FLASH_BINARY =  std::include_bytes!("../../stm32flash");

    let led_de = format!("{}/value", env::var("LED_DE_RE_n_GPIO")?);
    let boot_0 = format!("{}/value", env::var("BOOT0_GPIO")?);
    let boot_1 = format!("{}/value", env::var("BOOT1_GPIO")?);
    let reset = format!("{}/value", env::var("RESET_GPIO")?);
    
    let args = Args::parse();
    
    // If we are using a 485, put the system into bootloader mode.
    if args.rs485 {
        let mut port = match serialport::new(
            args.port_name.clone(),
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
        let mut gpio_file = OpenOptions::new().write(true).open(led_de.clone())?;
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
    let temp_dir = tempdir()?;
    let file_path = temp_dir.path().join("stm32flash");
    let mut file = std::fs::File::create(file_path.clone())?;
    let metadata = file.metadata()?;
    let mut permissions = metadata.permissions();
    permissions.set_mode(0o755);
    file.write_all(STM32FLASH_BINARY)?;
    drop(file);
    let _ = std::fs::set_permissions(file_path.clone(), permissions);

    println!("Temporary flash utility at {}", file_path.display());

    //Run the stm32flash tool with the appropriate commands.
    if args.rs485 {
        //program the board.
        println!("Running programming over RS485...");
        match std::process::Command::new(
            file_path,
        )
        .arg("-b")
        .arg(BAUDRATE.to_string())
        .args(["-R", "-s", "5", "-e", "7", "-w"])
        .arg(args.hex_file)
        .arg("-v")
        .arg(args.port_name)
        .arg("-d")
        .arg(led_de)
        .output()
        {
            Ok(val) => {
                println!("Command ran, success: {}", val.status.success());
                std::io::stdout().write_all(&val.stdout).unwrap();
                std::io::stderr().write_all(&val.stderr).unwrap();
            }
            Err(e) => {
                eprintln!("Failed to program: {}", e);
            }
        }
    } else {
        //program the board.
        println!("Running programming over serial...");

        //handle the gpio toggline
        let mut reset_file = OpenOptions::new().write(true).open(reset)?;
        let mut boot0_file = OpenOptions::new().write(true).open(boot_0)?;
        let mut boot1_file = OpenOptions::new().write(true).open(boot_1)?;
        reset_file.write(&['0' as u8])?;
        reset_file.rewind()?;
        boot0_file.write(&['1' as u8])?;
        boot0_file.rewind()?;
        boot1_file.write(&['0' as u8])?;
        boot1_file.rewind()?;
        std::thread::sleep(std::time::Duration::from_millis(500));

        reset_file.write(&['1' as u8])?;
        reset_file.rewind()?;

        std::thread::sleep(std::time::Duration::from_millis(500));

        match std::process::Command::new(
            file_path,
        )
        .arg("-b")
        .arg(BAUDRATE.to_string())
        .arg("-w")
        .arg(args.hex_file)
        .arg("-v")
        .arg(args.port_name)
        .output()
        {
            Ok(val) => {
                println!("Command ran, success: {}", val.status.success());
                std::io::stdout().write_all(&val.stdout).unwrap();
                std::io::stderr().write_all(&val.stderr).unwrap();
            }
            Err(e) => {
                eprintln!("Failed to program: {}", e);
            }
        }

        reset_file.write(&['0' as u8])?;
        reset_file.rewind()?;
        boot0_file.write(&['0' as u8])?;
        boot1_file.write(&['0' as u8])?;

        std::thread::sleep(std::time::Duration::from_millis(500));

        reset_file.write(&['1' as u8])?;
    }

    
    return Ok(());
}
