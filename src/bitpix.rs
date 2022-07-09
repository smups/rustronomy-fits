/*
    Copyright (C) 2022 Raúl Wolters

    This file is part of rustronomy-fits.

    rustronomy is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    rustronomy is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with rustronomy.  If not, see <http://www.gnu.org/licenses/>.
*/

use std::fmt::{self, Display, Formatter};

use crate::hdu_err::InvalidRecordValueError;

const VALID_BITPIX_VALUES: [&'static str; 6] = ["8", "16", "32", "64", "-32", "-64"];

#[derive(Debug, Clone, Copy)]
pub enum Bitpix {
  Byte,
  Short,
  Int,
  Long,
  Spf,
  Dpf,
}

impl Bitpix {
  pub(crate) fn from_code(code: &isize) -> Result<Bitpix, InvalidRecordValueError> {
    use Bitpix::*;
    match code {
      8 => Ok(Byte),
      16 => Ok(Short),
      32 => Ok(Int),
      64 => Ok(Long),
      -32 => Ok(Spf),
      -64 => Ok(Dpf),
      other => {
        Err(InvalidRecordValueError::new("BITPIX", &other.to_string(), &VALID_BITPIX_VALUES))
      }
    }
  }

  pub(crate) fn to_code(&self) -> isize {
    use Bitpix::*;
    match self {
      Byte => 8,
      Short => 16,
      Int => 32,
      Long => 64,
      Spf => -32,
      Dpf => -64,
    }
  }

  pub(crate) fn byte() -> Self {
    Self::Byte
  }
  pub(crate) fn short() -> Self {
    Self::Short
  }
  pub(crate) fn int() -> Self {
    Self::Int
  }
  pub(crate) fn long() -> Self {
    Self::Long
  }
  pub(crate) fn spf() -> Self {
    Self::Spf
  }
  pub(crate) fn dpf() -> Self {
    Self::Dpf
  }
}

impl Display for Bitpix {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    use Bitpix::*;
    match *self {
      Byte => write!(f, "u8"),
      Short => write!(f, "i16"),
      Int => write!(f, "i32"),
      Long => write!(f, "i62"),
      Spf => write!(f, "f32"),
      Dpf => write!(f, "f64"),
    }
  }
}
