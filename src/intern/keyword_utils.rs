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

use rayon::option;
use rustronomy_core::{meta::tags, prelude::MetaContainer};

use crate::{
  err::header_err::{InvalidHeaderErr, UTF8_KEYERR},
  hdu::Hdu,
};

use super::{fits_consts::*, HduOptions};

////////////////////////////////////////////////////////////////////////////////
//                          GENERAL HELPER FUNCTIONS                          //
////////////////////////////////////////////////////////////////////////////////

#[inline]
pub fn parse_fits_bool(string: &str) -> Result<bool, String> {
  match string {
    "T" => Ok(true),
    "F" => Ok(false),
    other => Err(format!("cannot parse {other} to bool, expected \"T\" or \"F\"")),
  }
}

#[inline]
/// This function removes the leading and trailing apostrophe's (') that FITS
/// string types always start and end with. It also removes any trailing whitespace.
pub fn strip_fits_string(string: &str) -> &str {
  let string = string.trim();
  if string.as_bytes()[0] == b'\'' && string.as_bytes()[string.len() - 1] == b'\'' {
    string[1..(string.len() - 1)].trim()
  } else {
    string
  }
}

pub fn parse_fits_datetime(
  key: &'static str,
  string: &str,
) -> Result<chrono::DateTime<chrono::Utc>, InvalidHeaderErr> {
  type Error = InvalidHeaderErr;

  //(1) Split datetime into date and time
  let datetime: Vec<&str> = string.split('T').collect();
  let (date, time) = if datetime.len() == 2 {
    (datetime[0].trim(), Some(datetime[1].trim()))
  } else {
    (datetime[0].trim(), None)
  };

  //(2) Split date into Year, month and day
  let ymd: Vec<&str> = date.split("-").collect();
  let year = if ymd[0].len() == 2 {
    //This is a year in the 20th century
    1900 + ymd[0].parse::<i32>().map_err(|err| Error::FmtErr { key, err: err.to_string() })?
  } else if ymd[0].len() == 4 {
    //This is a year in another century after the year 0
    ymd[0].parse::<i32>().map_err(|err| Error::FmtErr { key, err: err.to_string() })?
  } else if ymd[0].len() == 6 {
    //This is a year that might be BEFORE the year zero, or after 9999
    ymd[0].parse::<i32>().map_err(|err| Error::FmtErr { key, err: err.to_string() })?
  } else {
    return Err(Error::FmtErr { key, err: "".to_string() });
  };
  let month = ymd[1].parse::<u32>().map_err(|err| Error::FmtErr { key, err: err.to_string() })?;
  let day = ymd[2].parse::<u32>().map_err(|err| Error::FmtErr { key, err: err.to_string() })?;
  let date = chrono::NaiveDate::from_ymd(year, month, day);

  //(3) Split time into hour, minute and second
  let time = if let Some(hms) = time {
    let hms: Vec<&str> = hms.split(":").collect();
    let hour = hms[0].parse::<u32>().map_err(|err| Error::FmtErr { key, err: err.to_string() })?;
    let min = hms[1].parse::<u32>().map_err(|err| Error::FmtErr { key, err: err.to_string() })?;
    let sec = hms[2].parse::<u32>().map_err(|err| Error::FmtErr { key, err: err.to_string() })?;
    Some(chrono::NaiveTime::from_hms(hour, min, sec))
  } else {
    None
  };

  //(4) Return UTC time
  let datetime = if let Some(time) = time { date.and_time(time) } else { date.and_hms(0, 0, 0) };
  Ok(chrono::DateTime::from_utc(datetime, chrono::Utc))
}

////////////////////////////////////////////////////////////////////////////////
//                         TYPED TAG HELPER FUNCTIONS                         //
////////////////////////////////////////////////////////////////////////////////

#[inline]
pub fn set_creation_date(
  value: &str,
  meta: &mut impl MetaContainer,
) -> Result<(), InvalidHeaderErr> {
  meta.insert_tag(&tags::CreationDate(parse_fits_datetime(DATE_OBS, value)?));
  Ok(())
}

#[inline]
pub fn set_modified_date(
  value: &str,
  meta: &mut impl MetaContainer,
) -> Result<(), InvalidHeaderErr> {
  meta.insert_tag(&tags::LastModified(parse_fits_datetime(DATE, value)?));
  Ok(())
}

#[inline]
pub fn set_author(value: &str, meta: &mut impl MetaContainer) {
  meta.insert_tag(&tags::Author(value.to_string()));
  // Add authors to the reference publication if no authors have already been
  //specified
  if let Some(refpub) = meta.get_tag_mut::<tags::ReferencePublication>() {
    if refpub.authors() == "" {
      refpub.set_authors(value.to_string());
    }
  } else {
    meta.insert_tag(&tags::ReferencePublication::new("", value));
  }
}

#[inline]
pub fn set_refpub_title(value: &str, meta: &mut impl MetaContainer) {
  if let Some(refpub) = meta.get_tag_mut::<tags::ReferencePublication>() {
    refpub.set_title(value.to_string());
  } else {
    meta.insert_tag(&tags::ReferencePublication::new(value, ""));
  }
}

#[inline]
pub fn set_telescope(value: &str, meta: &mut impl MetaContainer) {
  meta.insert_tag(&tags::Telescope(value.to_string()));
}

#[inline]
pub fn set_instrument(value: &str, meta: &mut impl MetaContainer) {
  meta.insert_tag(&tags::Instrument(value.to_string()));
}

#[inline]
pub fn set_object(value: &str, meta: &mut impl MetaContainer) {
  meta.insert_tag(&tags::Object(value.to_string()));
}

////////////////////////////////////////////////////////////////////////////////
//                        FITS OPTIONS HELPER FUNCTIONS                       //
////////////////////////////////////////////////////////////////////////////////

macro_rules! create_parse_naxis_like_fn {
  ($(($base_key:ident, $fn_name:ident, $scalar_field:ident, $vec_field:ident)),*) => {$(
    #[inline]
    pub fn $fn_name(
      key: &str,
      value: Option<&str>,
      options: &mut HduOptions,
    ) -> Result<(), InvalidHeaderErr> {
      let idx = std::str::from_utf8(&key.as_bytes()[$base_key.len()..]).expect(UTF8_KEYERR);
      let raw_value = value.ok_or(InvalidHeaderErr::NoValue { key: $base_key })?;
      if idx == "" {
        *options.$scalar_field() = raw_value.parse().map_err(|err| InvalidHeaderErr::fmt_err($base_key, err))?;
        *options.$vec_field() = vec![0; *options.$scalar_field() as usize];
      } else {
        let idx: usize = idx.parse().map_err(|err| InvalidHeaderErr::fmt_err($base_key, err))?;
        let value = raw_value.parse().map_err(|err| InvalidHeaderErr::fmt_err($base_key, err))?;
        let naxes = *options.$scalar_field();
        //index in FITS starts with 1, rust starts with 0 so minus one to convert
        *options
          .$vec_field()
          .get_mut(idx - 1)
          .ok_or(InvalidHeaderErr::NaxisOob { idx, naxes })? = value;
      }
      Ok(())
    }
  )*};
}

create_parse_naxis_like_fn!((NAXIS, parse_naxis, n_axes_mut, shape_mut));

#[inline]
pub fn parse_simple(
  _key: &str,
  value: Option<&str>,
  options: &mut HduOptions,
) -> Result<(), InvalidHeaderErr> {
  let simple = value.ok_or(InvalidHeaderErr::NoValue { key: SIMPLE })?;
  //If simple is False, this is an error!
  if !parse_fits_bool(simple).map_err(|err| InvalidHeaderErr::FmtErr { key: SIMPLE, err })? {
    return Err(InvalidHeaderErr::SimpleErr);
  }
  //This is the primary header, conforming to the fits standard
  options.set_conforming(true);
  options.set_extension(super::Extension::PrimaryHdu);
  Ok(())
}

macro_rules! create_parse_bool_fn {
  ($(($base_key:ident, $fn_name:ident, $field:ident)),*) => {$(
    #[inline]
    pub fn $fn_name(
      _key: &str,
      value: Option<&str>,
      options: &mut HduOptions,
    ) -> Result<(), InvalidHeaderErr> {
      let bool_val = value.ok_or(InvalidHeaderErr::NoValue { key: $base_key })?;
      *options.$field() = super::keyword_utils::parse_fits_bool(bool_val)
        .map_err(|err| InvalidHeaderErr::FmtErr { key: $base_key, err })?;
      Ok(())
    }
  )*};
}

create_parse_bool_fn!(
  (EXTEND, parse_extend, extends_mut),
  (GROUPS, parse_groups, has_groups_mut),
  (INHERIT, parse_inherit, inherits_main_mut)
);

macro_rules! create_parse_number_fn {
  ($(($base_key:ident, $fn_name:ident, $field:ident, $type:ty)),*) => {$(
    #[inline]
    pub fn $fn_name(
      _key: &str,
      value: Option<&str>,
      options: &mut HduOptions,
    ) -> Result<(), InvalidHeaderErr> {
      let number = value.ok_or(InvalidHeaderErr::NoValue { key: $base_key })?;
      *options.$field() = str::parse::<$type>(number.trim())
        .map_err(|err| InvalidHeaderErr::FmtErr { key: $base_key, err: format!("{err}") })?;
      Ok(())
    }
  )*};
}

create_parse_number_fn!(
  (GCOUNT, parse_gcount, group_count_mut, u32),
  (PCOUNT, parse_pcount, parameter_count_mut, u32),
  (THEAP, parse_theap, heap_size_mut, u32),
  (TFIELDS, parse_tfields, row_size_mut, u32)
);

pub fn parse_bitpix(
  key: &str,
  value: Option<&str>,
  options: &mut HduOptions,
) -> Result<(), InvalidHeaderErr> {
  *options.bitpix_mut() = value
    .ok_or(InvalidHeaderErr::NoValue { key: BITPIX })?
    .parse()
    .map_err(|err| InvalidHeaderErr::fmt_err(BITPIX, err))?;
  Ok(())
}

#[test]
fn naxis_option_test() {
  //Setup dummy data
  const TEST_RECS: [(&str, Option<&str>, Option<&str>); 4] = [
    (NAXIS, Some("3"), None),
    ("NAXIS1", Some("1000"), None),
    ("NAXIS2", Some("2250"), None),
    ("NAXIS3", Some("272"), None),
  ];
  const TEST_ANSWER: [u32; 3] = [1000, 2250, 272];
  let mut input_options = HduOptions::new_invalid();
  for (key, value, _comment) in TEST_RECS {
    parse_naxis(key, value, &mut input_options).unwrap();
  }
  assert!(*input_options.n_axes() == input_options.shape().len() as u32);
  assert!(input_options.shape().len() == TEST_ANSWER.len());
  assert!(*input_options.shape() == TEST_ANSWER);
}

#[test]
fn naxis_oob_test() {
  const TEST_RECS: (&str, Option<&str>, Option<&str>) = ("NAXIS1", Some("1200"), None);
  let mut input_options = HduOptions::new_invalid();
  assert!(matches!(
    parse_naxis(TEST_RECS.0, TEST_RECS.1, &mut input_options),
    Err(InvalidHeaderErr::NaxisOob { idx: 1, naxes: 0 })
  ))
}

#[test]
fn invalid_novalue_simple_test() {
  const TEST_RECS: (&str, Option<&str>, Option<&str>) = (SIMPLE, None, None);
  let mut input_options = HduOptions::new_invalid();
  assert!(matches!(
    parse_simple(TEST_RECS.0, TEST_RECS.1, &mut input_options),
    Err(InvalidHeaderErr::NoValue { .. })
  ));
}

#[test]
fn simple_option_test() {
  //Setup dummy data
  const TEST_RECS: (&str, Option<&str>, Option<&str>) = (SIMPLE, Some("T"), None);
  const TEST_ANSWER: bool = true;
  let mut input_options = HduOptions::new_invalid();
  parse_simple(TEST_RECS.0, TEST_RECS.1, &mut input_options).unwrap();
  assert!(*input_options.conforming() == TEST_ANSWER);
}

#[test]
fn bitpix_option_test() {
  //Setup dummy data
  const TEST_RECS: (&str, Option<&str>, Option<&str>) = (BITPIX, Some("-32"), None);
  const TEST_ANSWER: i8 = -32;
  let mut input_options = HduOptions::new_invalid();
  parse_bitpix(TEST_RECS.0, TEST_RECS.1, &mut input_options).unwrap();
  assert!(*input_options.bitpix() == TEST_ANSWER);
}

#[test]
fn invalid_novalue_bitpix_test() {
  const TEST_RECS: (&str, Option<&str>, Option<&str>) = (BITPIX, None, None);
  let mut input_options = HduOptions::new_invalid();
  assert!(matches!(
    parse_bitpix(TEST_RECS.0, TEST_RECS.1, &mut input_options),
    Err(InvalidHeaderErr::NoValue { .. })
  ));
}
