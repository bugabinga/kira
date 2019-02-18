use std::fs::read_to_string;
use std::fs::File;
use std::io::prelude::*;
use std::io::Result;
use std::path::Path;

/// `kira` allows you to set the display brightness on linux machines with intel graphics cards.
/// There are three input modes:
/// ```sh
/// $ kira
/// ```
/// Invoking `kira` without arguments will set brightness to 100%.
///
/// ```sh
/// $ kira 55
/// ```
/// Invoking `kira` with an integer between 0 and 100 will set the brightness to the percent amount of
/// that number.
///
/// ```sh
/// $ kira +10
/// $ kira -22
/// ```
/// Invoking `kira` with an integer prefixed with either `-` or `+` will decrease or increase by
/// given amount in percent.
///
/// Any change in brightness will occur stepwise with a small delay inbetween.
/// This results in a linear smooth change of brightness over time.
fn main() -> Result<()> {
    let backlight = Path::new("/sys/class/backlight/intel_backlight/");
    let max_brightness = backlight.join("max_brightness");
    let brightness = backlight.join("brightness");
    let max_brightness = read_to_string(max_brightness)?;
    let max_brightness_value = max_brightness.trim().parse().unwrap();
    let args: Vec<String> = std::env::args().collect();
    let target: u16 = if args.len() > 1 {
        let input = &args[1];
        let (signum, input) = if input.starts_with('+') {
            (Some(true), &input[1..])
        } else if input.starts_with('-') {
            (Some(false), &input[1..])
        } else {
            (None, &input[..])
        };
        let percent: u8 = input.parse().unwrap();
        let value: u16 = (max_brightness_value as f32 * percent as f32 / 100.0) as u16;
        let current_value: u16 = read_to_string(&brightness)?.trim().parse().unwrap();
        match signum {
            Some(positive) => {
                if positive {
                    current_value + value
                } else {
                    current_value - value
                }
            }
            None => value,
        }
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
