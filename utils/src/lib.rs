use color_eyre::Result;
use chrono::{NaiveDate, Datelike, Duration};

pub mod err;

/// Setup the library and the address to use based on the environment variable
/// for each gRPC microservices
/// 
/// # Arguments
/// * `port` - i32
pub fn setup_services(name: &str) -> Result<()> {
    // set environment variable for log debugging
    std::env::set_var("RUST_LOG", format!("{}=info,util=info", name));
    std::env::set_var("RUST_BACKTRACE", "1");

    color_eyre::install()?;
    env_logger::init();

    Ok(())
}

/// Generate the server address
/// 
/// # Arguments
/// * `port` - i32
pub fn get_server_addr(port: i32) -> String {
    match std::env::var("rust_env") {
        Ok(res) => {
            if res == "prod" {
                format!("0.0.0.0:{}", port)
            } else {
                format!("127.0.0.1:{}", port)
            }
        },
        Err(_) => format!("127.0.0.1:{}", port)
    }
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
    fn build_date(&self) -> Result<String, err::MaskErr> {
        let (formatted_date, is_full_day) = match self.get_day() {
            Some(day) => (format!("{}-{}-{}", self.get_year(), self.get_month(), day), true),
            // add an empty 1 to make the date hopefully valid
            None => (format!("{}-{}-1", self.get_year(), self.get_month()), false)
        };

        match NaiveDate::parse_from_str(&formatted_date, "%Y-%m-%d") {
            Ok(d) => {
                if is_full_day {
                    return Ok(d.format("%Y-%m-%d").to_string())
                }

                Ok(d.format("%Y-%m").to_string())
            },
            Err(err) => Err(err::MaskErr::IO(err.to_string()))
        }
    }

    /// Build a date and append a '%' for LIKE queries
    ///
    /// # Arguments
    /// * `&self` - Self
    fn build_date_sql_like(&self) -> Result<String, err::MaskErr> {
        let date = self.build_date()?;

        Ok(format!("{}%", date))
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


