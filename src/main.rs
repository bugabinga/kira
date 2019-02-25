use std::error::Error;
use std::fs::read_to_string;
use std::fs::File;
use std::io::prelude::*;
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
fn main() -> Result<(), Box<dyn Error>> {
    let backlight = Path::new("/sys/class/backlight/intel_backlight/");
    let max_brightness = backlight.join("max_brightness");
    let brightness = backlight.join("brightness");
    let max_brightness = read_to_string(max_brightness)?;
    let max_brightness_value = max_brightness.trim().parse()?;
    let min_brightness_value = 0;
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
    file.write_all(&value.to_ne_bytes())?;
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
    fn checkcalculated_target_values() {
        assert_eq!(calculate_target_value(None, 22, 0, 100, 0), 22u16);
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
