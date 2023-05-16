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
mod file_io;
mod hdu_io;
mod header_io;
mod keyword_utils;
mod test_io;

//re-exports
pub use file_io::*;
pub use hdu_io::*;

#[derive(Debug, Clone, PartialEq)]
pub struct HduOptions {
  extension: String, //Type of extension described by this HDU
  conforming: bool,  //does the file conform to the FITS standard
  bitpix: i8,        //-64 to +64
  extends: bool,     //does the file contain extensions
  dim: u16,          //number of array axes. Max is 999
  shape: Vec<usize>, //each axis max size is undefined
  /* Random groups options */
  parameter_count: usize, //number of parameters preceding a group array
  group_count: usize,     //number of random groups
  //p_types (should have a custom type)
  param_scales: Vec<f64>, //rescaling of p_real = p_scale * p + p0
  param_zeros: Vec<f64>,  //see p_scales
  /* Table options */
  row_size: u16,            //number of entries (fields) in each row of table
  column_start: Vec<usize>, //specifies the column in which each field starts
  //column_format (should have a custom type)
  field_scales: Vec<f64>,     //rescaling of t_real = t_scale * t + t0
  field_zeros: Vec<f64>,      //see t_scales
  field_null: Vec<String>,    //null value format for each field
  field_dispfmt: Vec<String>, //display format for each field
}

impl HduOptions {
  pub fn new_invalid() -> Self {
    HduOptions {
      extension: String::default(),
      conforming: false,
      bitpix: -1,
      extends: false,
      dim: 0,
      shape: Vec::new(),
      parameter_count: 0,
      group_count: 0,
      param_scales: Vec::new(),
      param_zeros: Vec::new(),
      row_size: 0,
      column_start: Vec::new(),
      field_scales: Vec::new(),
      field_zeros: Vec::new(),
      field_null: Vec::new(),
      field_dispfmt: Vec::new(),
    }
  }
}

pub(crate) mod fits_consts {
  //Constants defined by the FITS standard
  pub const BLOCK_SIZE: usize = 2880;
  pub const RECORD_SIZE: usize = 80;

  //Comment separator
  pub const SEP: char = '/';

  //Keywords that correspond to FITS options
  pub const SIMPLE: &str = "SIMPLE";
  pub const BITPIX: &str = "BITPIX";
  pub const NAXIS: &str = "NAXIS";
  pub const END: &str = "END";
  pub const GCOUNT: &str = "GCOUNT";
  pub const PCOUNT: &str = "PCOUNT";
  pub const GROUPS: &str = "GROUPS";
  pub const PTYPE: &str = "PTYPE";
  pub const PSCAL: &str = "PSCAL";
  pub const PZERO: &str = "PZERO";
  pub const TFIELDS: &str = "TFIELDS";
  pub const TBCOL: &str = "TBCOL";
  pub const TFORM: &str = "TFORM";
  pub const TTYPE: &str = "TTYPE";
  pub const TUNIT: &str = "TUNIT";
  pub const TSCAL: &str = "TSCAL";
  pub const TZERO: &str = "TZERO";
  pub const TNULL: &str = "TNULL";
  pub const TDISP: &str = "TDISP";
  pub const THEAP: &str = "THEAP";
  pub const EXTEND: &str = "EXTEND";
  pub const INHERIT: &str = "INHERIT";
  pub const XTENSION: &str = "EXTENSION";
  pub const BSCALE: &str = "BSCALE";
  pub const BZERO: &str = "BZERO";
  pub const BUNIT: &str = "BUNIT";
  pub const DATASUM: &str = "DATASUM";
  pub const CHECKSUM: &str = "CHECKSUM";
  pub const CONTINUE: &str = "CONTINUE";
  pub const COMMENT: &str = "COMMENT";
  pub const HISTORY: &str = "HISTORY";
  pub const BLANK: &str = "BLANK";

  //FITS keywords that correspond to keywords that rustronomy understands
  pub const DATE: &str = "DATE";
  pub const DATE_OBS: &str = "DATE-OBS";
  pub const AUTHOR: &str = "AUTHOR";
  pub const REFERENC: &str = "REFERENC";
  pub const TELESCOP: &str = "TELESCOP";
  pub const INSTRUME: &str = "INSTRUME";
  pub const OBJECT: &str = "OBJECT";
  pub const ORIGIN: &str = "ORIGIN";
  pub const OBSERVER: &str = "OBSERVER";
}
