use crate::FirstDate;
pub use std::fs::{self, File};
use std::io::{BufRead, BufReader, Error, ErrorKind};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const MONTHS_IN_YEAR: i16 = 12;
const EPOCH_YEAR: f32 = 1970.0;
const EPOCH_DAY: isize = 86400;
const DAYS_IN_YEAR_WITH_LEAP: f32 = 365.2425;
const MILLISECONDS_IN_SECOND: isize = 1000;

const MONTHS: [(&str, i16); 12] = [
    ("January", 31),
    ("February", 28),
    ("March", 31),
    ("April", 30),
    ("May", 31),
    ("June", 30),
    ("July", 31),
    ("August", 31),
    ("September", 30),
    ("October", 31),
    ("November", 30),
    ("December", 31),
];

pub enum AddEnum {
    SameUnit(u16, Option<String>),
    NextUnit(u16, u16, Option<String>),
}

struct DstDateRange {
    day: i8,
    month: String,
    year: i16,
    hour: i16,
    minute: i16,
    second: i16,
}

struct DstRange {
    end: DstDateRange,
    start: DstDateRange,
}

pub enum TimeUnit {
    Day,
    Year,
    Hour,
    Month,
    Minute,
    Second,
    Millisecond,
}

pub fn is_dst(date: &FirstDate) -> Result<bool, Error> {
    let args: Vec<String> = Vec::from([
        String::from("-V"),
        String::from("-c"),
        String::from(format!("{},{}", date.year, date.year + 1)),
        date.timezone_name.clone(),
    ]);

    let command_result = Command::new("zdump").args(args).output()?;
    let string_output_result = String::from_utf8(command_result.stdout);
    let raw_string_output = match string_output_result {
        Ok(string_data) => string_data,
        Err(e) => return Err(Error::new(ErrorKind::Other, e)),
    };

    let sliced_output: Vec<&str> = raw_string_output.split("\n").collect();
    let dst_range = parse_zdump(sliced_output);

    let january_index = 1;
    let february_index = 2;
    let march_index = 3;
    let novemember_index = 11;
    let december_index = 12;
    if date.month == january_index || date.month == february_index || date.month == december_index {
        return Ok(false);
    }
    if date.month == march_index {
        if date.day <= dst_range.start.day as u8 {
            return Ok(false);
        }
        if date.day == dst_range.start.day as u8
            && (date.hour <= dst_range.start.hour as u8
                || date.minute <= dst_range.start.minute as u8
                || date.second <= dst_range.start.second as u8)
        {
            return Ok(false);
        }
    }

    if date.month == novemember_index {
        if date.day <= dst_range.end.day as u8 {
            return Ok(false);
        }
        if date.day == dst_range.end.day as u8
            && (date.hour >= dst_range.end.hour as u8
                || date.minute >= dst_range.end.minute as u8
                || date.second >= dst_range.end.second as u8)
        {
            return Ok(false);
        }
    }
    Ok(true)
}

fn parse_zdump(zdump_data: Vec<&str>) -> DstRange {
    let filtered_formatted_data: Vec<&str> = zdump_data
        .into_iter()
        .filter(|line| line.contains("isdst=1"))
        .map(|line| {
            let formatted_line: Vec<&str> = line.split("=").collect();
            return formatted_line[1].trim();
        })
        .collect();

    let mut range = DstRange {
        start: DstDateRange {
            day: -1,
            hour: -1,
            minute: -1,
            second: -1,
            year: -1,
            month: String::from(""),
        },
        end: DstDateRange {
            day: -1,
            hour: -1,
            minute: -1,
            second: -1,
            year: -1,
            month: String::from(""),
        },
    };

    let start = filtered_formatted_data[0];
    let end = filtered_formatted_data[1];
    if start.contains("Mar") {
        range.start.month = String::from("March");
        let start_time_data = extract_day_and_time(&start);
        range.start.day = start_time_data.0;
        range.start.hour = start_time_data.1;
        range.start.minute = start_time_data.2;
        range.start.second = start_time_data.3;
        range.start.year = start_time_data.4;
    }
    if end.contains("Nov") {
        range.end.month = String::from("November");
        let end_time_data = extract_day_and_time(&end);
        range.end.day = end_time_data.0;
        range.end.hour = end_time_data.1;
        range.end.minute = end_time_data.2;
        range.end.second = end_time_data.3;
        range.end.year = end_time_data.4;
    }

    range
}

// (day, hour, minute, second, year)
fn extract_day_and_time(data: &str) -> (i8, i16, i16, i16, i16) {
    let mut answer = (-1, -1, -1, -1, -1);
    let mut char_iterator = data.chars();
    let mut current_char: char = ' ';
    let mut current_char_option: Option<char> = None;
    loop {
        current_char_option = char_iterator.next();
        if current_char_option == None {
            break;
        }

        current_char = current_char_option.unwrap();
        if current_char != ' ' {
            if current_char.is_digit(10) {
                if answer.0 < 0 {
                    let mut initial_number = String::from(current_char);
                    if initial_number == "1" {
                        initial_number.push(char_iterator.next().unwrap());
                    }
                    answer.0 = initial_number.parse::<i8>().unwrap();
                } else if answer.1 < 0 {
                    let hour = char_iterator
                        .next()
                        .unwrap()
                        .to_string()
                        .parse::<i16>()
                        .unwrap();
                    answer.1 = hour;
                } else if answer.4 < 0 {
                    let mut year_index = 0;
                    let mut year_string = String::from(current_char);
                    loop {
                        if year_index == 3 {
                            break;
                        } else {
                            year_string.push(char_iterator.next().unwrap());
                        }
                        year_index += 1;
                    }
                    answer.4 = year_string.parse::<i16>().unwrap();
                }
            } else if current_char == ':' {
                loop {
                    let next_sub_char = char_iterator.next().unwrap();
                    if next_sub_char == ' ' {
                        break;
                    }

                    if answer.2 < 0 {
                        let minute_1 = next_sub_char;
                        let minute_2 = char_iterator.next().unwrap();
                        answer.2 = format!("{}{}", minute_1, minute_2).parse::<i16>().unwrap();
                    } else if answer.3 < 0 {
                        let second_1 = char_iterator.next().unwrap();
                        let second_2 = char_iterator.next().unwrap();
                        answer.3 = format!("{}{}", second_1, second_2).parse::<i16>().unwrap();
                    }
                }
            }
        } else if answer.0 > 0
            && answer.0 > 0
            && answer.1 > 0
            && answer.2 > 0
            && answer.3 > 0
            && answer.4 > 0
        {
            break;
        }
    }

    answer
}

pub fn add_helper(current_value: i16, time_to_add: i16, unit_modulus: i16) -> AddEnum {
    let new_value = current_value + time_to_add;
    if (new_value / unit_modulus as i16) < 1 {
        if unit_modulus == MONTHS_IN_YEAR {
            return AddEnum::SameUnit(
                new_value as u16,
                Some(MONTHS[(new_value - 1) as usize].0.to_string()),
            );
        }
        return AddEnum::SameUnit(new_value as u16, None);
    }

    let current_unit_new_value = (new_value % unit_modulus) as u16;
    let next_unit_new_value = (new_value / unit_modulus) as u16;
    if unit_modulus == MONTHS_IN_YEAR {
        return AddEnum::NextUnit(
            current_unit_new_value,
            next_unit_new_value,
            Some(MONTHS[(current_unit_new_value - 1) as usize].0.to_string()),
        );
    }
    AddEnum::NextUnit(current_unit_new_value, next_unit_new_value, None)
}

/// current_month_index is expected to be 1-indexed.
pub fn add_day_helper(
    current_day: i16,
    time_to_add: i16,
    current_month_index: i8,
    is_leap_year: bool,
) -> AddEnum {
    let new_value = current_day + time_to_add;
    let current_month_modulus = if is_leap_year && current_month_index == 2 {
        MONTHS[(current_month_index - 1) as usize].1 + 1
    } else {
        MONTHS[(current_month_index - 1) as usize].1
    };
    if (new_value / current_month_modulus as i16) < 1 {
        return AddEnum::SameUnit(new_value as u16, None);
    }

    let current_unit_new_value = (new_value % current_month_modulus) as u16;
    let next_unit_new_value = (new_value / current_month_modulus) as u16;
    return AddEnum::NextUnit(
        current_unit_new_value,
        next_unit_new_value,
        Some(MONTHS[(current_unit_new_value - 1) as usize].0.to_string()),
    );
}

pub fn generate_first_date_from_millis<'b>(millis: isize) -> FirstDate {
    // convert the milliseconds to seconds
    let epoch_seconds = millis / MILLISECONDS_IN_SECOND;

    // convert the seconds to days
    let epoch_days = epoch_seconds / EPOCH_DAY;

    let day_milliseconds = epoch_days * EPOCH_DAY * MILLISECONDS_IN_SECOND;

    // convert the days to years. We need to account for leap years
    let epoch_years = (epoch_days as f32) / DAYS_IN_YEAR_WITH_LEAP;

    // use the years to calculate the current year
    let year = (epoch_years + EPOCH_YEAR) as u16;

    // at this point, the number of years is a fraction (something like 55.382385) and we want to get rid of the whole number and then multiply by the number of days in a year to get the number of days again.
    let mut remaining_days = (epoch_years - epoch_years.floor()) * DAYS_IN_YEAR_WITH_LEAP;

    // is the current year a leap year?
    let is_current_year_leap_year = is_leap_year(year);

    // calculate the current month from the remaining days
    let mut month_index: i8 = 0;
    let mut month: &str = "";
    for (index, (current_month, days_in_month)) in MONTHS.iter().enumerate() {
        if remaining_days <= *(days_in_month) as f32 {
            month_index = index as i8;
            month = current_month;
            break;
        } else if is_current_year_leap_year && current_month.to_owned() == "February" {
            remaining_days -= *(days_in_month) as f32;
            remaining_days -= 1.0;
        } else {
            remaining_days -= *(days_in_month) as f32;
        }
    }

    // calculate the current day from the remaining days
    let day = (remaining_days.floor() as u8) + 1; // add one because days are 1-based

    // calculate the current hour from the remaining days
    let min_in_ms = 60 * MILLISECONDS_IN_SECOND;
    let hour_in_ms = 60 * 60 * MILLISECONDS_IN_SECOND;
    let remaining_milliseconds = millis - day_milliseconds;
    let hour = (remaining_milliseconds / hour_in_ms) as u8;

    // calculate the current minute from the remaining days
    let remaining_after_hours = remaining_milliseconds % hour_in_ms;
    let minute = (remaining_after_hours / min_in_ms) as u8;

    // calculate the current second from the remaining days
    let remaining_after_minutes = remaining_after_hours % min_in_ms;
    let second = (remaining_after_minutes / MILLISECONDS_IN_SECOND) as u8;

    // calculate the current millisecond from the remaining after minutes
    let millisecond = (remaining_after_minutes % MILLISECONDS_IN_SECOND) as u16;

    return FirstDate {
        day,
        hour,
        year,
        second,
        minute,
        millisecond,
        timezone_offset: None,
        is_daylight_savings: None,
        month: (month_index + 1) as u8, // for loops start at 0 so we need to add 1.
        month_name: String::from(month),
        timezone_name: String::from("unset"),
        is_leap_year: is_current_year_leap_year,
    };
}

pub fn epoch_milliseconds() -> isize {
    let start = SystemTime::now();
    let duration = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards?");
    return duration.as_millis() as isize;
}

pub fn is_leap_year(year_to_test: u16) -> bool {
    year_to_test % 4 == 0 && (year_to_test % 100 != 0 || year_to_test % 400 == 0)
}

pub fn local_timezone() -> String {
    let raw_system_timezone_result = fs::read_to_string("/etc/timezone");
    match raw_system_timezone_result {
        Ok(system_timezone) => String::from(system_timezone.trim()),
        Err(_) => String::from("Unable to get system timezone."),
    }
}

pub fn local_timezone_offset(system_timezone: String) -> Result<i16, Error> {
    let timezone_table = File::open("resources/timezone_list.txt")?;
    let reader = BufReader::new(timezone_table);

    for line in reader.lines() {
        let current_line = match line {
            Ok(data) => data,
            Err(_) => String::from(""),
        };

        if current_line.contains(&system_timezone) {
            let matched_line: Vec<&str> = current_line.split(" ").collect();
            let result = matched_line[1][..matched_line.len()].parse::<i16>();
            return match result {
                Ok(timezone) => Ok(timezone),
                Err(_e) => Err(Error::new(
                    ErrorKind::Other,
                    format!("Could not parse the timezone value: {}", &current_line),
                )),
            };
        }
    }

    Err(Error::new(
        ErrorKind::NotFound,
        "Could not find the provided timezone",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_leap_year_test_true_century() {
        assert_eq!(is_leap_year(2000), true);
    }

    #[test]
    fn is_leap_year_test_true_mod4() {
        assert_eq!(is_leap_year(2020), true);
    }

    #[test]
    fn is_leap_year_test_false() {
        assert_eq!(is_leap_year(2021), false);
    }

    #[test]
    fn generate_first_date_from_millis_test_positive() {
        let mock_millis: isize = 1747972008447;
        let result = generate_first_date_from_millis(mock_millis);

        assert_eq!(result.year, 2025);
        assert_eq!(result.month, 5);
        assert_eq!(result.day, 23);
        assert_eq!(result.hour, 3);
        assert_eq!(result.minute, 46);
        assert_eq!(result.second, 48);
        assert_eq!(result.millisecond, 447);
        assert_eq!(result.is_leap_year, false);
    }
}
