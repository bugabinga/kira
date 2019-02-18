use std::fs::read_to_string;
use std::fs::File;
use std::io::prelude::*;
use std::io::Result;
use std::path::Path;

/// `kira` allows you to set the display brightness on linux machines with intel graphics cards.
fn main() -> Result<()> {
    let backlight = Path::new("/sys/class/backlight/intel_backlight/");
    let max_brightness = backlight.join("max_brightness");
    let brightness = backlight.join("brightness");
    let max_brightness = read_to_string(max_brightness)?;
    let max_brightness_value = max_brightness.trim().parse().unwrap();

    let args: Vec<String> = std::env::args().collect();
    let target: u16 = if args.len() > 1 {
        let percent: u8 = args[1].parse().unwrap();
        (max_brightness_value as f32 * percent as f32 / 100.0) as u16
    } else {
        max_brightness_value
    };

    let current_brightness_value: u16 = read_to_string(&brightness)?.trim().parse().unwrap();

    if target > current_brightness_value {
        for b in current_brightness_value..=target {
            let mut current_brightness_file = File::create(&brightness)?;
            current_brightness_file.write_all(&b.to_string().as_bytes())?;
            current_brightness_file.sync_data()?;
            std::thread::sleep(std::time::Duration::from_nanos(100));
        }
    } else if target < current_brightness_value {
        for b in (target..=current_brightness_value).rev() {
            let mut current_brightness_file = File::create(&brightness)?;
            current_brightness_file.write_all(&b.to_string().as_bytes())?;
            current_brightness_file.sync_data()?;
            std::thread::sleep(std::time::Duration::from_nanos(100));
        }
    }

    Ok(())
}
