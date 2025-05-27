use first_date::{FirstDate, FirstDateOperations};

fn main() {
    let test = FirstDate::now();
    println!("{}", test.date_string(false));
    println!("{}", test.date_string(true));
}
