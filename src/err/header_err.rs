/*
  Copyright© 2023 Raúl Wolters(1)

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

//Error messages
pub(crate) const UTF8_KEYERR: &str = "Could not parse FITS keyword record using UTF-8 encoding";
pub(crate) const UTF8_RECERR: &str = "Could not parse FITS record value using UTF-8 encoding";

#[derive(Debug)]
pub enum HeaderReadErr {
  IoErr(super::io_err::FitsReadErr),
  InvalidHeader(InvalidHeaderErr),
}

impl From<InvalidHeaderErr> for HeaderReadErr {
  fn from(value: InvalidHeaderErr) -> Self {
    HeaderReadErr::InvalidHeader(value)
  }
}

impl From<super::io_err::FitsReadErr> for HeaderReadErr {
  fn from(value: super::io_err::FitsReadErr) -> Self {
    HeaderReadErr::IoErr(value)
  }
}

impl std::fmt::Display for HeaderReadErr {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    use HeaderReadErr::*;
    match self {
      IoErr(err) => write!(f, "IOError: \"{err}\""),
      InvalidHeader(err) => write!(f, "Malformed Header: \"{err}\"")
    }
  }
}
impl std::error::Error for HeaderReadErr {}

#[derive(Debug)]
pub enum InvalidHeaderErr {
  NoValue { key: &'static str },
  NaxisOob { idx: usize, naxes: u16 },
  FmtErr { key: &'static str, err: String },
}

impl InvalidHeaderErr {
  pub(crate) fn fmt_err<T>(key: &'static str, err: T) -> Self
  where
    T: std::fmt::Display + std::error::Error,
  {
    Self::FmtErr { key, err: format!("{err}") }
  }
}

impl std::fmt::Display for InvalidHeaderErr {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    use InvalidHeaderErr::*;
    match self {
      NoValue { key } => write!(f, "encountered a {key} keyword without a value."),
      NaxisOob { idx, naxes } => {
        write!(f, "encountered NAXIS{} keyword, but number of axes is only {}.", idx, naxes)
      }
      FmtErr { key, err } => {
        write!(f, "encountered malformed {} keyword. Fmt error:\"{}\"", key, err)
      }
    }
  }
}
impl std::error::Error for InvalidHeaderErr {}
