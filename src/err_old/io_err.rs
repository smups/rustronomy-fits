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

use std::{
  error::Error,
  fmt::{self, Display, Formatter},
};

#[derive(Debug)]
pub struct InvalidFitsFileErr {
  /*
    This error may be thrown when opening a FITS file. If the FITS file has
    invalid encoding (for whatever reason), this error will be thrown.
  */
  msg: &'static str,
}

//Possible messages
pub(crate) const FILE_BLOCK_DIV: &'static str =
  "tried to open file with a length not equal to an integer multiple of FITS blocks";
pub(crate) const BUF_BLOCK_DIV: &'static str =
  "supplied buffer not an integer multiple of FITS blocks";
pub(crate) const FILE_END: &'static str = "tried to read more FITS blocks than the file contains";
pub(crate) const CORRUPTED: &'static str = "tried to access corrupted data";

impl Error for InvalidFitsFileErr {}
impl Display for InvalidFitsFileErr {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    write!(f, "Error while accessing FITS file: {}", self.msg)
  }
}

impl InvalidFitsFileErr {
  pub(crate) fn new(msg: &'static str) -> Self {
    InvalidFitsFileErr { msg: msg }
  }
}
