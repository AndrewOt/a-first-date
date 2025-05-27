# A First Date
**NOTE: THIS IS NOT A PRODUCTION READY LIBRARY!**

**This is a library for learning and exploring ideas, please do not use this in production (see below for known issues).**

This is a date library written in Rust so could learn the Rust programming language and how date information is managed in common date libraries. It supports some of basic features that allowed me to learn more about dates.

## Usage
```Rust
use first_date::{FirstDate, FirstDateOperations};

fn main() {
    let my_date = FirstDate::now();
    // your code here
}
```

Start by importing the library by using `FirstDate` (the struct) and `FirstDateOperations` (the trait implementation). See the API section below for the methods available on `FirstDateOperations`.

## API

### Fields
day - u8 - The current day of the first-date.

hour - u8 - The current hour of the first-date.

month - u8 - The current month of the first-date.

year - u16 - The current year of the first-date.

minute - u8 - The current minute of the first-date.

second - u8 - The current second of the first-date.

millisecond - u16 -The current millisecond of the first-date.

is_leap_year - bool - Indicates if the current year is a leap year or not.

month_name - String - Then english name of the current month.

timezone_name - String - The name of the current timezone (based on system settings), for example "America/Chicago". This ONLY works on linux (or maybe Unix systems) and will fail silently if not present.

timezone_offset - Option<i8> - The numerical offset of the current timezone (based on system settings), for example "America/Chicago" is -5 (not in daylight savings time). This ONLY works on linux (or maybe Unix systems) and will fail silently and set the field to `None` if not present.

is_daylight_savings - Option<bool> - Inidcates if the date accounts for daylight savings time (by adding 1 hour to the time). While the dates will vary this period is between early March and early November, following the standard rules for DST. The library queries the Unix daylight savings time database to determing the rules based on the year. This ONLY works on Unix, it will fail silently and set the field to `None` otherwise.

### Methods
#### `FirstDate::now()`
This method returns a `FirstDate` object that represents the current instant in time. To do this, the method follows the following steps:
1. Query the millisecond duration from the system.
2. Determine and factor in the timezone. This is done by querying the `/etc/timezone` file in the Unix system (same throughout).
3. Determine and factor in the daylight savings time. This is done by querying the daylight savings database via the `zdump` command (same throughout).

#### `FirstDate::from_millis(millis)`
This method returns a `FirstDate` object that represents the instant in time based on the milliseconds provided (in reference to Jan 1 1970). Note, this does not account for timezone or daylight savings time (this can be done with methods below).

#### `add(time_to_add: i16, unit: TimeUnit)`
This method adds time to a given first date.

The value added to the date object is the first parameter. This value can be positive or negative and that amount of time is added to the object.

The second parameter, the `TimeUnit` enum, indicates what part of the duration will be affected. For example, the following code `my_date.add(15, TimeUnit::Hour);` will add 15 hours to `my_date`. Subtracting time would look like this `my_date.add(15, TimeUnit::Hour);`.

#### `set_timezone(timezone_name: String)`
This method sets the timezone based on a timezone name (for example "America/Chicago") and adds the offset to the current date.

#### `set_daylight_savings(is_dst: bool)`
This method sets the fields appropiately based on the boolean parameter.

#### `date_string(display_24_hour: bool)`
This method spits out a formatted string for human consumption. The only formatting parameter indicates if the time should be in 12-hour or 24-hour time. Thus, all the strings returned by this method are like either "05/27/2025 17:32" or "05/27/2025 5:32 PM."

## Known Issues/Limitations
This is not a production ready library. It was built so I could learn the Rust programming language and how dates work. Thus, there are some issues and limitations (known and unknown):

1. Adding years does not account for leap years.
2. BC/AD distinctions are not present (but could be easily added). Thus BC is going to be displayed as negative years.
3. Timezone/daylight savings time is Linux dependent (and even some distros may not work but I think everything should work on Debain).
4. The `add` function accounts for overflow from one unit to another (for example, 65 minutes => 1 hr. 5 mins.) but has not been tested and thus may not properly handle daylight savings time.
5. There are unit tests present (so I could learn unit tests in Rust) but they don't cover all the corner cases, so there is a possiblity that cases exist where the code could panic or produce an erroneous result.
6. There are several places where the code will silently fail (primarily various points in the timezone and DST logic). Optimally, these places would support these functionalities in a platform independant way and if there are errors, notifying the consumer.
