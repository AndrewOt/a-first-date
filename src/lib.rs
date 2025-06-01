use crate::date_utilities::{
    add_day_helper, add_helper, epoch_milliseconds, generate_first_date_from_millis, is_dst,
    is_leap_year, local_timezone, local_timezone_offset, AddEnum, TimeUnit,
};
use std::env::consts::OS;

mod date_utilities;

#[derive(Debug, Clone)]
pub struct FirstDate {
    pub day: u8,
    pub hour: u8,
    pub month: u8,
    pub year: u16,
    pub minute: u8,
    pub second: u8,
    pub millisecond: u16,
    pub is_leap_year: bool,
    pub month_name: String,
    pub timezone_name: String,
    pub timezone_offset: Option<i8>,
    pub is_daylight_savings: Option<bool>,
}

pub trait FirstDateOperations {
    /// Creates a new instance with values from the current instant. Note, if you are on a Linux machine, this will account for timezone (via the `/etc/timezone` file). If the library fails to get the timezone from the system or fails in retriving the matching value from the timezone table file, the timezone will not be applied. Otherwise it the timezone will be UTC-0. Also attempts to query daylight savings transition rules (also linux specific) and will fail silently if the operation fails.
    fn now() -> Self;
    /// Creates a new instance with values determined by the number of milliseconds provided.
    /// Timezone will be UTC-0.
    fn from_millis(millis: isize) -> Self;
    /// Adds a given amount of time based on the unit. Note: `time_to_add` can be negative time to subtract.
    fn add(&mut self, time_to_add: i16, unit: TimeUnit);
    /// Can be used if timezone not set because `now()` was called but not on a compatible linux system or another date method was used (for example `from_millis`) to initiate the first_date.
    fn set_timezone(&mut self, timezone_name: String);
    /// Can be used if daylight savings not set because `now()` was called but not on a compatible linux system or another date method was used (for example `from_millis`) to initiate the first_date. If the timezone is not found, it silently fails.
    fn set_daylight_savings(&mut self, is_dst: bool);
    /// This method returns a singular date string and it takes a flag to toggle 24-hour time and 12-hour time (with AM/PM). Follows the pattern MM/dd/YYYY HH:mm:ss (AM/PM if applicable).
    fn date_string(&self, display_24_hour: bool) -> String;
}

impl Default for FirstDate {
    fn default() -> FirstDate {
        FirstDate {
            day: 0,
            hour: 0,
            year: 0,
            month: 1,
            minute: 0,
            second: 0,
            millisecond: 0,
            is_leap_year: false,
            timezone_offset: Some(0),
            is_daylight_savings: Some(false),
            month_name: String::from("Januray"),
            timezone_name: String::from("unset"),
        }
    }
}

impl FirstDateOperations for FirstDate {
    fn now() -> Self {
        let epoch_duration = epoch_milliseconds();
        let mut first_date = generate_first_date_from_millis(epoch_duration);
        if OS == "linux" {
            let local_machine_timezone = local_timezone();
            first_date.timezone_name = local_machine_timezone.clone();
            first_date.set_timezone(local_machine_timezone);
            if is_dst(&first_date).unwrap() {
                first_date.set_daylight_savings(true);
            }
        }

        first_date
    }

    fn from_millis(millis: isize) -> Self {
        return generate_first_date_from_millis(millis);
    }

    fn add(&mut self, time_to_add: i16, unit: TimeUnit) {
        match unit {
            TimeUnit::Day => {
                let new_day_result = add_day_helper(
                    self.day as i16,
                    time_to_add,
                    self.month as i8,
                    self.is_leap_year,
                );
                match new_day_result {
                    AddEnum::SameUnit(day_value, _) => self.day = day_value as u8,
                    AddEnum::NextUnit(day_value, month_index_value, _) => {
                        self.day = day_value as u8;
                        self.add(month_index_value as i16, TimeUnit::Month);
                    }
                }
            }
            TimeUnit::Month => {
                let months_in_year = 12;
                let new_month_value = add_helper(self.month as i16, time_to_add, months_in_year);
                match new_month_value {
                    AddEnum::SameUnit(month_value, month_name) => {
                        self.month = month_value as u8;
                        self.month_name = month_name.unwrap();
                    }
                    AddEnum::NextUnit(month_value, year_value, month_name) => {
                        self.month_name = month_name.unwrap();
                        self.month = month_value as u8;
                        self.add(year_value as i16, TimeUnit::Year);
                    }
                }
            }
            TimeUnit::Year => {
                let current_year = self.year as i16;
                self.year = (current_year + time_to_add) as u16; // Would be cool to handle BC/AD at some point
                self.is_leap_year = is_leap_year(self.year);
                // Known issue: add day from any new leap years
            }
            TimeUnit::Hour => {
                let hours_in_day = 24;
                let new_hour_value = add_helper(self.hour as i16, time_to_add, hours_in_day);
                match new_hour_value {
                    AddEnum::SameUnit(hour_value, _) => self.hour = hour_value as u8,
                    AddEnum::NextUnit(hour_value, day_value, _) => {
                        self.hour = hour_value as u8;
                        self.add(day_value as i16, TimeUnit::Day);
                    }
                }
            }
            TimeUnit::Minute => {
                let minutes_in_hour = 60;
                let new_minute_value = add_helper(self.minute as i16, time_to_add, minutes_in_hour);
                match new_minute_value {
                    AddEnum::SameUnit(minute_value, _) => self.minute = minute_value as u8,
                    AddEnum::NextUnit(minute_value, hour_value, _) => {
                        self.minute = minute_value as u8;
                        self.add(hour_value as i16, TimeUnit::Hour);
                    }
                }
            }
            TimeUnit::Second => {
                let seconds_in_hour = 60;
                let new_second_value = add_helper(self.second as i16, time_to_add, seconds_in_hour);
                match new_second_value {
                    AddEnum::SameUnit(second_value, _) => self.second = second_value as u8,
                    AddEnum::NextUnit(second_value, minute_value, _) => {
                        self.second = second_value as u8;
                        self.add(minute_value as i16, TimeUnit::Minute);
                    }
                }
            }
            TimeUnit::Millisecond => {
                let milliseconds_in_hour = 1000;
                let new_millisecond_value =
                    add_helper(self.millisecond as i16, time_to_add, milliseconds_in_hour);
                match new_millisecond_value {
                    AddEnum::SameUnit(millisecond_value, _) => {
                        self.millisecond = millisecond_value as u16
                    }
                    AddEnum::NextUnit(millisecond_value, second_value, _) => {
                        self.millisecond = millisecond_value as u16;
                        self.add(second_value as i16, TimeUnit::Second);
                    }
                }
            }
        }
    }

    fn set_timezone(&mut self, timezone_name: String) {
        let timezone_offset = local_timezone_offset(timezone_name).unwrap();
        self.add(timezone_offset, TimeUnit::Hour);
        self.timezone_offset = Some(timezone_offset as i8);
    }

    fn set_daylight_savings(&mut self, is_dst: bool) {
        match is_dst {
            true => {
                if self.is_daylight_savings == None || !self.is_daylight_savings.unwrap() {
                    self.is_daylight_savings = Some(true);
                    self.add(1, TimeUnit::Hour);
                }
            }
            false => {
                if self.is_daylight_savings.unwrap() {
                    self.is_daylight_savings = Some(false);
                    self.add(-1, TimeUnit::Hour);
                }
            }
        }
    }

    fn date_string(&self, display_24_hour: bool) -> String {
        let mut date_string: String = String::new();
        if self.month < 10 {
            date_string.push_str(format!("0{}", self.month).as_str());
        } else {
            date_string.push_str(format!("{}", self.month).as_str());
        }

        date_string.push('/');
        if self.day < 10 {
            date_string.push_str(format!("0{}", self.day).as_str());
        } else {
            date_string.push_str(format!("{}", self.day).as_str());
        }

        date_string.push('/');
        date_string.push_str(self.year.to_string().as_str());
        date_string.push(' ');

        if !display_24_hour {
            date_string.push_str(self.hour.to_string().as_str());
        } else {
            date_string.push_str((self.hour % 12).to_string().as_str());
        }
        date_string.push(':');

        date_string.push_str(self.minute.to_string().as_str());
        date_string.push(':');
        date_string.push_str(self.second.to_string().as_str());

        if display_24_hour {
            date_string.push(' ');
            date_string.push_str(if self.hour < 12 { "AM" } else { "PM" });
        }

        date_string
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_year_positive_leap_false() {
        let mock_millis: isize = 1747972008447;
        let mut sut = FirstDate::from_millis(mock_millis);
        sut.add(1, TimeUnit::Year);
        assert_eq!(sut.year, 2026);
        assert_eq!(sut.is_leap_year, false);
    }

    #[test]
    fn add_year_positive_leap_true() {
        let mock_millis: isize = 1747972008447;
        let mut sut = FirstDate::from_millis(mock_millis);
        sut.add(3, TimeUnit::Year);
        assert_eq!(sut.year, 2028);
        assert_eq!(sut.is_leap_year, true);
    }

    #[test]
    fn subtract_year_positive_leap_false() {
        let mock_millis: isize = 1747972008447;
        let mut sut = FirstDate::from_millis(mock_millis);
        sut.add(-1, TimeUnit::Year);
        assert_eq!(sut.year, 2024, "Year incorrect");
        assert_eq!(sut.is_leap_year, true, "Leap incorrect");
    }

    #[test]
    fn add_hour_positive() {
        let mock_millis: isize = 1747972008447;
        let mut sut = FirstDate::from_millis(mock_millis);
        sut.add(3, TimeUnit::Hour);
        assert_eq!(sut.hour, 6);
    }

    #[test]
    fn subtract_day_positive() {
        let mock_millis: isize = 1747972008447;
        let mut sut = FirstDate::from_millis(mock_millis);
        sut.add(-1, TimeUnit::Hour);
        assert_eq!(sut.hour, 2);
    }

    #[test]
    fn add_minute_positive() {
        let mock_millis: isize = 1747972008447;
        let mut sut = FirstDate::from_millis(mock_millis);
        sut.add(3, TimeUnit::Minute);
        assert_eq!(sut.minute, 49);
    }

    #[test]
    fn subtract_minute_positive() {
        let mock_millis: isize = 1747972008447;
        let mut sut = FirstDate::from_millis(mock_millis);
        sut.add(-3, TimeUnit::Minute);
        assert_eq!(sut.minute, 43);
    }

    #[test]
    fn add_minute_rollover_to_hour_positive() {
        let mock_millis: isize = 1747972008447;
        let mut sut = FirstDate::from_millis(mock_millis);
        sut.add(15, TimeUnit::Minute);
        assert_eq!(sut.minute, 1);
        assert_eq!(sut.hour, 4);
    }

    #[test]
    fn add_second_rollover_to_minute_positive() {
        let mock_millis: isize = 1747972008447;
        let mut sut = FirstDate::from_millis(mock_millis);
        sut.add(15, TimeUnit::Second);
        assert_eq!(sut.second, 3);
        assert_eq!(sut.minute, 47);
    }

    #[test]
    fn subtract_second_positive() {
        let mock_millis: isize = 1747972008447;
        let mut sut = FirstDate::from_millis(mock_millis);
        sut.add(-15, TimeUnit::Second);
        assert_eq!(sut.second, 33);
    }

    #[test]
    fn subtract_millisecond_positive() {
        let mock_millis: isize = 1747972008447;
        let mut sut = FirstDate::from_millis(mock_millis);
        sut.add(-1, TimeUnit::Millisecond);
        assert_eq!(sut.millisecond, 446);
    }

    #[test]
    fn add_millisecond_rollover_to_second_positive() {
        let mock_millis: isize = 1747972008447;
        let mut sut = FirstDate::from_millis(mock_millis);
        sut.add(600, TimeUnit::Millisecond);
        assert_eq!(sut.millisecond, 47);
        assert_eq!(sut.second, 49);
    }

    #[test]
    fn subtract_month_positive() {
        let mock_millis: isize = 1747972008447;
        let mut sut = FirstDate::from_millis(mock_millis);
        sut.add(-1, TimeUnit::Month);
        assert_eq!(sut.month, 4);
        assert_eq!(sut.month_name, "April");
    }

    #[test]
    fn add_month_rollover_to_year_positive() {
        let mock_millis: isize = 1747972008447;
        let mut sut = FirstDate::from_millis(mock_millis);
        sut.add(9, TimeUnit::Month);
        assert_eq!(sut.month, 2);
        assert_eq!(sut.month_name, "February");
    }

    #[test]
    fn add_day_positive() {
        let mock_millis: isize = 1747972008447;
        let mut sut = FirstDate::from_millis(mock_millis);
        sut.add(1, TimeUnit::Day);
        assert_eq!(sut.day, 24);
    }

    #[test]
    fn add_day_rollover_to_month_positive() {
        let mock_millis: isize = 1747972008447;
        let mut sut = FirstDate::from_millis(mock_millis);
        sut.add(9, TimeUnit::Day);
        assert_eq!(sut.day, 1);
        assert_eq!(sut.month, 6);
        assert_eq!(sut.month_name, "June");
    }

    #[test]
    fn set_daylight_savings_true_positive() {
        let mock_millis: isize = 1747972008447;
        let mut sut = FirstDate::from_millis(mock_millis);
        dbg!(&sut);

        // Set daylight savings
        sut.set_daylight_savings(true);
        assert_eq!(sut.hour, 4);

        // Revert daylight savings
        sut.set_daylight_savings(false);
        assert_eq!(sut.hour, 3);
    }
}
