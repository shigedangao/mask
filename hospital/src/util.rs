use chrono::NaiveDate;

use crate::err::MaskErr;

pub trait Date {
    fn build_date(&self) -> (String, bool);
}

/// Check whenever the date is valid
/// 
/// # Arguments
/// * `date` - &str
/// * `is_day` - bool
pub fn is_date_valid(input: &impl Date) -> Result<String, MaskErr> {
    let (date, is_day) = Date::build_date(input);
    let updated_date = match is_day {
        true => date.clone(),
        false => format!("{}-1", date)
    };

    if let Err(_) = NaiveDate::parse_from_str(&updated_date, "%Y-%m-%d") {
        return Err(MaskErr::MalformattedDate);
    }

    Ok(date)
} 
