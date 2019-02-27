use std::error::Error;
use std::fs::read_to_string;
use std::fs::File;
use std::io::prelude::*;
use std::num::ParseIntError;
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
fn main() {
    match kira() {
        Err(error) => {
            eprintln!("{}", match_error_to_message(&error));
            println!("{:?}", &error);
            let stderr = std::io::stderr();
            let mut err_handle = stderr.lock();
            print_usage(&mut err_handle).expect("Could not even write to stderr. Sad life.");
        }
        _ => {}
    }
}

fn print_usage(writer: &mut dyn Write) -> Result<(), Box<dyn Error>> {
    write!(
        writer,
        "
usage: kira [+-][percent]

percent must be a number between 0 and 100.
A prefix of either - oder + is allowed.
Without a prefix, the brightness gets set to the given percentage.
With the + prefix, the given percentage gets added to current brightness.
With the - prefix, the given percentage gets subtracted from current brightness.

You need permission to modify the backlight device in `/sys/class/backlight/`.
"
    )?;
    Ok(())
}

// For every error that is expected to occur in kira, this method maps a "friendly"
// explanation text to it.
fn match_error_to_message(error: &Box<dyn Error>) -> &'static str {
    if error.is::<ParseIntError>() {
        "Given percent value needs to be a number between 0 and 100."
    } else if error.is::<std::io::Error>() {
        "Could not access the backlight device.
Does ist exist?
Usually `/sys/class/backlight/intel_backlight/` or similar.
Also, do you have permission to edit it?
On most Linux distributions you need to be part of a special group (video?)."
    } else {
        "Oh oh, we've made a fucky wucky..."
    }
}

fn kira() -> Result<(), Box<dyn Error>> {
    let backlight = Path::new("/sys/class/backlight/intel_backlight/");
    let brightness = backlight.join("brightness");
    let max_brightness_value: u16 = read_to_string(backlight.join("max_brightness"))?
        .trim()
        .parse()?;
    let min_brightness_value: u16 = 0;
    let current_value: u16 = read_to_string(&brightness)?.trim().parse()?;
    let args: Vec<String> = std::env::args().collect();
    let target: u16 = if args.len() > 1 {
        let (signum, percent) = parse_input_as_percent(&args[1])?;
        calculate_target_value(
            signum,
            percent,
            current_value,
            max_brightness_value,
            min_brightness_value,
        )
    } else {
        max_brightness_value
    };
    let current_brightness_value: u16 = read_to_string(&brightness)?.trim().parse()?;
    let current_brightness_file = File::create(&brightness)?;
    if target > current_brightness_value {
        for b in current_brightness_value..=target {
            write_to_file_and_wait(&current_brightness_file, b, 100)?;
        }
    } else if target < current_brightness_value {
        for b in (target..=current_brightness_value).rev() {
            write_to_file_and_wait(&current_brightness_file, b, 100)?;
        }
    }
    Ok(())
}

fn write_to_file_and_wait(mut file: &File, value: u16, nanos: u64) -> Result<(), Box<dyn Error>> {
    file.write_all(&value.to_string().as_bytes())?;
    file.sync_data()?;
    std::thread::sleep(std::time::Duration::from_nanos(nanos));
    Ok(())
}

fn parse_input_as_percent(input: &str) -> Result<(Option<bool>, u8), Box<dyn Error>> {
    if input.starts_with('+') {
        Ok((Some(true), input[1..].parse()?))
    } else if input.starts_with('-') {
        Ok((Some(false), input[1..].parse()?))
    } else {
        Ok((None, input[..].parse()?))
    }
}

/// Calculates the actual brigthness value, given the min-max-range and percent value.
/// If a signum is given, the percentage value will be added//subtracted to the current
/// brightness value.
/// In any case, this method returns the absolute target brigthness value.
/// signum: Relativizes the `percent` value. `None` means the given `percent` value is
/// meant to be an absolute target value. `Some(true)` means the target is the current
/// value added to the percentage. `Some(false)` means the target is the subtraction of
/// the current value and the given percentage.
/// percent: Percentage of wanted target value. Expected to be between 0 - 100.
/// current: the current absolute brightness value (not a percentage).
/// max: the maximum absolute brightness value.
/// min: the minimum absolute brigthness value.
fn calculate_target_value(
    signum: Option<bool>,
    percent: u8,
    current: u16,
    max: u16,
    min: u16,
) -> u16 {
    let value: u16 = (max as f32 * percent as f32 / 100.0) as u16;
    match signum {
        Some(positive) => {
            let new_value = if positive {
                current.saturating_add(value)
            } else {
                current.saturating_sub(value)
            };
            if new_value >= max {
                max
            } else if new_value <= min {
                min
            } else {
                new_value
            }
        }
        None => {
            if value >= max {
                max
            } else if value <= min {
                min
            } else {
                value
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_that_usage_is_basically_there() {
        let mut buffer = vec![];
        print_usage(&mut buffer).expect("fail");
        let string = String::from_utf8(buffer).expect("fail");
        assert!(string.contains("usage"));
        assert!(string.contains("kira"));
        assert!(string.contains("percent"));
        assert!(string.contains("-"));
        assert!(string.contains("+"));
        assert!(string.contains("sys"));
        assert!(string.contains("device"));
        assert!(string.contains("permission"));
    }

    #[test]
    fn checkcalculated_target_values() {
        assert_eq!(calculate_target_value(None, 22, 0, 100, 0), 22u16);
        assert_eq!(calculate_target_value(None, 77, 0, 4438, 0), 3417u16);
        assert_eq!(calculate_target_value(None, 0, 0, 100, 0), 0u16);
        assert_eq!(calculate_target_value(None, 100, 0, 100, 0), 100u16);
        assert_eq!(calculate_target_value(None, 200, 0, 100, 0), 100u16);
        assert_eq!(calculate_target_value(None, 22, 0, 100, 50), 50u16);

        assert_eq!(calculate_target_value(Some(true), 22, 0, 100, 0), 22u16);
        assert_eq!(calculate_target_value(Some(true), 22, 10, 100, 0), 32u16);
        assert_eq!(calculate_target_value(Some(true), 22, 80, 100, 0), 100u16);
        assert_eq!(calculate_target_value(Some(true), 122, 80, 100, 0), 100u16);
        assert_eq!(calculate_target_value(Some(true), 200, 80, 100, 0), 100u16);
        assert_eq!(calculate_target_value(Some(true), 1, 100, 100, 0), 100u16);
        assert_eq!(calculate_target_value(Some(true), 0, 0, 100, 0), 0u16);

        assert_eq!(calculate_target_value(Some(false), 22, 0, 100, 0), 0u16);
        assert_eq!(calculate_target_value(Some(false), 22, 50, 100, 0), 28u16);
        assert_eq!(calculate_target_value(Some(false), 22, 55, 100, 50), 50u16);
        assert_eq!(calculate_target_value(Some(false), 22, 88, 100, 0), 66u16);

        assert_eq!(calculate_target_value(None, 22, 0, 1000, 0), 220u16);
        assert_eq!(calculate_target_value(None, 0, 0, 1000, 0), 0u16);
        assert_eq!(calculate_target_value(None, 100, 0, 1000, 0), 1000u16);
        assert_eq!(calculate_target_value(None, 110, 0, 1000, 0), 1000u16);
        assert_eq!(calculate_target_value(None, 1, 0, 10000, 0), 100u16);
        assert_eq!(calculate_target_value(None, 33, 0, 100, 0), 33u16);
        assert_eq!(calculate_target_value(None, 73, 0, 14687, 999), 10721u16);
    }

    #[test]
    fn check_expected_input_values() {
        let (signum, percent) = parse_input_as_percent("+10").unwrap();
        assert_eq!(signum, Some(true));
        assert_eq!(percent, 10u8);

        let (signum, percent) = parse_input_as_percent("+0").unwrap();
        assert_eq!(signum, Some(true));
        assert_eq!(percent, 0u8);

        let (signum, percent) = parse_input_as_percent("+100").unwrap();
        assert_eq!(signum, Some(true));
        assert_eq!(percent, 100u8);

        let (signum, percent) = parse_input_as_percent("+44").unwrap();
        assert_eq!(signum, Some(true));
        assert_eq!(percent, 44u8);

        let (signum, percent) = parse_input_as_percent("-10").unwrap();
        assert_eq!(signum, Some(false));
        assert_eq!(percent, 10u8);

        let (signum, percent) = parse_input_as_percent("-200").unwrap();
        assert_eq!(signum, Some(false));
        assert_eq!(percent, 200u8);

        let (signum, percent) = parse_input_as_percent("+250").unwrap();
        assert_eq!(signum, Some(true));
        assert_eq!(percent, 250u8);

        let (signum, percent) = parse_input_as_percent("244").unwrap();
        assert_eq!(signum, None);
        assert_eq!(percent, 244u8);

        let (signum, percent) = parse_input_as_percent("10").unwrap();
        assert_eq!(signum, None);
        assert_eq!(percent, 10u8);

        let (signum, percent) = parse_input_as_percent("-100").unwrap();
        assert_eq!(signum, Some(false));
        assert_eq!(percent, 100u8);

        let (signum, percent) = parse_input_as_percent("100").unwrap();
        assert_eq!(signum, None);
        assert_eq!(percent, 100u8);

        let (signum, percent) = parse_input_as_percent("35").unwrap();
        assert_eq!(signum, None);
        assert_eq!(percent, 35u8);

        let (signum, percent) = parse_input_as_percent("255").unwrap();
        assert_eq!(signum, None);
        assert_eq!(percent, 255u8);
    }

    #[test]
    #[should_panic]
    fn check_larger_than_u8_error() {
        parse_input_as_percent("300").unwrap();
    }
    #[test]
    #[should_panic]
    fn check_larger_than_u8_positive_error() {
        parse_input_as_percent("+300").unwrap();
    }
    #[test]
    #[should_panic]
    fn check_larger_than_u8_negative_error() {
        parse_input_as_percent("-300").unwrap();
    }
    #[test]
    #[should_panic]
    fn check_empty_error() {
        parse_input_as_percent("").unwrap();
    }
    #[test]
    #[should_panic]
    fn check_very_larger_than_u8_error() {
        parse_input_as_percent("42934632").unwrap();
    }
    #[test]
    #[should_panic]
    fn check_words_error() {
        parse_input_as_percent("not a number").unwrap();
    }
    #[test]
    #[should_panic]
    fn check_binary_error() {
        parse_input_as_percent("0x110010").unwrap();
    }
    #[test]
    #[should_panic]
    fn check_number_as_word_error() {
        parse_input_as_percent("five").unwrap();
    }
}
