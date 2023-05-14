/*
  Copyright© 2022 Raúl Wolters(1)

  This file is part of rustronomy-fits.

  rustronomy is free software: you can redistribute it and/or modify it under
  the terms of the European Union Public License version 1.2 or later, as
  published by the European Commission.

  rustronomy is distributed in the hope that it will be useful, but WITHOUT ANY
  WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR
  A PARTICULAR PURPOSE. See the European Union Public License for more details.

  You should have received a copy of the EUPL in an/all official language(s) of
  the European Union along with rustronomy.  If not, see
  <https://ec.europa.eu/info/european-union-public-licence_en/>.

  (1) Resident of the Kingdom of the Netherlands; agreement between licensor and
  licensee subject to Dutch law as per article 15 of the EUPL.
*/

use rustronomy_core::{meta::tags, prelude::MetaContainer};

use crate::err::header_err::InvalidHeaderErr;

pub fn parse_fits_bool(string: &str) -> Result<bool, String> {
  match string {
    "T" => Ok(true),
    "F" => Ok(false),
    other => Err(format!("cannot parse {other} to bool, expected \"T\" or \"F\"")),
  }
}

// pub fn parse_fits_datetime(string: &str) -> Result<chrono::DateTime<chrono::Utc>, String> {
//   //(1) Split datetime into date and time
//   let datetime: Vec<&str> = string.split('T').collect();
//   let (date, time) = (datetime[0].trim(), datetime[1].trim());

//   //(2) Split date into Year, month and day
//   let ymd: Vec<&str> = date.split("-").collect();
//   let year: i32 = if ymd[0].len() == 2 {
//     //This is a year in the 20th century
//     1900i32 + str::parse(ymd[0]).map_err(|e| format!("{e}"))?
//   } else if ymd[0].len() == 4 {
//     //This is a year in another century after the year 0
//     str::parse(ymd[0]).map_err(|e| format!("{e}"))?
//   } else if ymd[0].len() == 6 {
//     //This is a year that might be BEFORE the year zero, or after 9999
//     str::parse(ymd[0]).map_err(|e| format!("{e}"))?
//   } else {
//     return Err("invalid date format".to_string());
//   };
//   let month: u8 = str::parse(ymd[1]).map_err(|e| format!("{e}"))?;

//   todo!()
// }

fn insert_date(value: &str, meta: &mut impl MetaContainer) -> Result<(), InvalidHeaderErr> {
  meta.insert_tag(&tags::CreationDate(
    value.parse().map_err(|err| InvalidHeaderErr::fmt_err(DATE_OBS, err))?,
  ));
  Ok(())
}
