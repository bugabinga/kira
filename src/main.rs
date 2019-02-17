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

    let args: Vec<String> = std::env::args().collect();
    let target: u16 = if args.len() > 1 {
        args[1].parse().unwrap()
    } else {
        max_brightness.trim().parse().unwrap()
    };

    let current_brightness: u16 = read_to_string(&brightness)?.trim().parse().unwrap();

    dbg!(&current_brightness);
    for b in current_brightness..target {
        let mut current_brightness_file = File::create(&brightness)?;
        current_brightness_file.write_all(&b.to_string().as_bytes())?;
        current_brightness_file.sync_data()?;
    }

    // TODO delete later
    let current = read_to_string(&brightness)?;
    dbg!(&current);

    Ok(())
}
