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

//Std imports
use std::error::Error;

//external imports
use rustronomy_core::universal_containers::*;

//internal imports
use crate::api::hdu::Hdu;
use header_io::*;

//module structure
mod fits_io;
mod hdu_io;
mod header_io;
mod keyword_utils;

//re-exports
pub use fits_io::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FitsOptions {
  conforming: bool,
  bitpix: i8, //-64 to +64
  extends: bool,
  dim: u16,        //number of axes. Max is 999
  shape: Vec<u16>, //each axis max size is 999
}

impl FitsOptions {
  pub fn new_invalid() -> Self {
    FitsOptions { conforming: false, bitpix: 0, extends: false, dim: 0, shape: Vec::new() }
  }
}

pub fn read_primary_hdu(
  reader: &mut FitsReader,
) -> Result<(meta_only::MetaOnly, Hdu), Box<dyn Error>> {
  //Max. number of records in a FITS block
  const MAX_RECS: usize = crate::BLOCK_SIZE / crate::RECORD_SIZE;

  //(1) Read all the raw bytes in the header
  let header_bytes = read_header(reader)?;

  todo!()
}
