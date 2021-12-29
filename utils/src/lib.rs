use std::fs;
use color_eyre::Result;
use chrono::{NaiveDate, Datelike, Duration};

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
                ("warn", format!("0.0.0.0:{}", port))
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

/// Retrieve the certificate either form a filepath set on the environment variable
/// or by looking at the keys folder (local dev)
pub fn get_certificates() -> Result<(Vec<u8>, Vec<u8>), Box<dyn std::error::Error>> {
    let env = match std::env::var("rust_env") {
        Ok(res) => res,
        Err(_) => "dev".to_owned()
    };

    if env == "prod" {
        let filepath_cert = std::env::var("server_cert")?;
        let filepath_key = std::env::var("server_key")?;

        let server_cert = fs::read(filepath_cert)?;
        let server_key = fs::read(filepath_key)?;

        return Ok((server_cert, server_key));
    }

    let server_cert = fs::read("../keys/server-cert.pem")?;
    let server_key = fs::read("../keys/server-key.key")?;

    Ok((server_cert, server_key))
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
    fn get_month(&self) -> i32;
    /// Return the day
    /// Optional as we might only want the month for a year...
    /// 
    /// # Arguments
    /// * `&self` - &Date
    fn get_day(&self) -> Option<i32>;
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

    /// Return a list of the last 7 day based on a given day
    /// For example if the given date is 2021-12-23. The method will returns a list of
    /// dates between 2021-12-20 -> 2021-12-26
    /// 
    /// # Arguments
    /// * `&self`
    fn get_previous_seven_date_from_day(&self) -> Option<Vec<String>> {
        let day = self.get_day()?;

        // building a date with chrono
        let stringify_date = format!("{}-{}-{}", self.get_year(), self.get_month(), day);
        let date = match NaiveDate::parse_from_str(&stringify_date, "%Y-%m-%d") {
            Ok(res) => res,
            Err(_) => {
                return None;
            }
        };

        let mut days = Vec::new();
        let based_date = NaiveDate::from_ymd_opt(date.year(), date.month(), date.day())?;
        for i in 0..7 {
            let naivedate = based_date.pred_opt()? - Duration::days(i);
            days.push(naivedate);
        };

        let days: Vec<String> = days
            .into_iter()
            .map(|day| format!("{}-{}-{}", day.year(), day.month(), day.day()))
            .collect();
            
        Some(days)
    }
}

