use color_eyre::Result;
use chrono::{NaiveDate, Datelike};

/// Setup the library and the address to use based on the environment variable
/// for each gRPC microservices
/// 
/// # Arguments
/// * `port` - i32
pub fn setup_services(port: i32) -> Result<String> {
    // set RUST_LOG based on the environment variable
    let (log_level, addr) = match std::env::var("rust_env") {
        Ok(res) => {
            if res == "prod" {
                ("warn", format!("0.0.0.1:{}", port))
            } else {
                ("info", format!("127.0.0.1:{}", port))
            }
        },
        Err(_) => ("info", format!("127.0.0.1:{}", port))
    };

    // set environment variable for log debugging
    std::env::set_var("RUST_LOG", log_level);
    std::env::set_var("RUST_BACKTRACE", "1");

    color_eyre::install()?;
    env_logger::init();

    Ok(addr.to_owned())
}

pub trait Date {
    /// Return the year
    /// 
    /// # Arguments
    /// * `&self` - &Date
    fn get_year(&self) -> i32;
    /// Return the month
    /// 
    /// # Arguments
    /// * `&self` - &Date
    fn get_month(&self) -> String;
    /// Return the day
    /// Optional as we might only want the month for a year...
    /// 
    /// # Arguments
    /// * `&self` - &Date
    fn get_day(&self) -> Option<String>;
    /// Build Date based day, month, year from the struct. 
    /// Return an option if the date is valid.
    /// The date can be in either this format:
    ///     - YYYY-MM-DD
    ///     - YYYY-MM
    /// 
    /// /!\ Chrono won't validate a date which does not have a day. Hence
    ///     In the case if the day is returning None. We're adding 1 to the formatted date
    ///     to possibly mark it as a valid date
    /// 
    /// # Arguments
    /// * `&self` - &Date
    fn build_date(&self) -> Option<String> {
        let (formatted_date, is_full_day) = match self.get_day() {
            Some(day) => (format!("{}-{}-{}", self.get_year(), self.get_month(), day), true),
            // add an empty 1 to make the date hopefully valid
            None => (format!("{}-{}-1", self.get_year(), self.get_month()), false)
        };

        match NaiveDate::parse_from_str(&formatted_date, "%Y-%m-%d") {
            Ok(d) => {
                if is_full_day {
                    return Some(d.format("%Y-%m-%d").to_string())
                }

                Some(format!("{}-{}", d.year(), d.month()))
            },
            Err(_) => None
        }
    }
}

