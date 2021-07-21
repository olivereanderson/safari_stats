//! # Date utilities
//!
//! This module yields utilities for obtaining dates represented as strings of the form YYYYMMDD.
//!

use chrono::{Datelike, NaiveDate, Utc};
use regex::Regex;

/// Produces today's date as a DateStamp
pub fn today_ymd() -> DateStamp {
    let today_utc = Utc::today();
    let today = NaiveDate::from_ymd(today_utc.year(), today_utc.month(), today_utc.day());
    DateStamp::from_ymd(today.format("%Y%m%d").to_string())
}

/// Produces a vector of the last seven dates formated as YYMMDD
pub fn last_seven_days_ymd() -> Vec<DateStamp> {
    let today_utc = Utc::today();
    let today = NaiveDate::from_ymd(today_utc.year(), today_utc.month(), today_utc.day());
    let mut dates = [today; 7];
    for i in (0..6).rev() {
        dates[i] = dates[i + 1].pred();
    }
    dates
        .iter()
        .map(|x| x.format("%Y%m%d").to_string())
        .map(DateStamp::from_ymd)
        .collect()
}

// Produces a vector of Strings of the form YYYYMMDD
// The first entry is the date from 7 days earlier and the last is yesterday.
pub fn previous_six_days() -> Vec<DateStamp> {
    last_seven_days_ymd().into_iter().take(6).collect()
}

/// A date of the form YYYYMMDD
#[derive(Eq, PartialEq, Clone)]
pub struct DateStamp {
    date: String,
}
impl DateStamp {
    /// transforms a string of the form YYYYMMDD to a DateStamp
    ///
    /// #Panics:
    /// If a string not of the form YYYYMMDD is passed then this function will panic.  
    pub fn from_ymd(date_ymd: String) -> Self {
        let regex = Regex::new(r"^(\d{4})(\d{2})(\d{2})$").unwrap();
        assert!(
            regex.is_match(date_ymd.as_str()),
            "The supplied string {} is not of the form YYYYMMDD",
            date_ymd
        );
        Self { date: date_ymd }
    }

    /// returns the corresponding date represented as a string of the form YYYYMMDD
    pub fn into_string(self) -> String {
        self.date
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn last_seven_days_distinct() {
        let last_seven_days_ymd = last_seven_days_ymd();
        for (idx, date) in last_seven_days_ymd.iter().enumerate() {
            assert!(
                !(last_seven_days_ymd[..idx].contains(date)
                    || last_seven_days_ymd[(idx + 1)..].contains(date))
            )
        }
    }
}
