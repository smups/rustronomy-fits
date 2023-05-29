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

use getset::{Getters, MutGetters, Setters};

use crate::err::header_err::InvalidHeaderErr;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
#[non_exhaustive]
pub enum Extension {
  PrimaryHdu,
  Groups,
  Image,
  Table,
  BinTable,
  Foreign,
  #[default]
  Dump, //NRAO AIPS binary tables are currently unsupported!
}

#[derive(Debug, Clone, PartialEq, Getters, Setters, MutGetters)]
#[getset(get = "pub", set = "pub", get_mut = "pub")]
pub struct HduOptions {
  extension: Extension, //Type of extension described by this HDU
  conforming: bool,     //does the file conform to the FITS standard
  extends: bool,        //does the file contain extensions
  has_groups: bool,     //does the file contain groups
  inherits_main: bool,  //does the file inherit the metadata from the primary HDU
  bitpix: i8,           //data type of array
  n_axes: u32,          //number of array axes. Max is 999
  shape: Vec<u32>,      //each axis max size is undefined
  /* Random groups options */
  parameter_count: u32, //number of parameters preceding a group array
  group_count: u32,     //number of random groups
  //p_types (should have a custom type)
  param_scales: Vec<f64>, //rescaling of p_real = p_scale * p + p0
  param_zeros: Vec<f64>,  //see p_scales
  /* Table options */
  row_size: u32,          //number of entries (fields) in each row of table
  column_start: Vec<u32>, //specifies the column in which each field starts
  heap_size: u32,         //specifies the size of the heap
  //column_format (should have a custom type)
  field_scales: Vec<f64>,     //rescaling of t_real = t_scale * t + t0
  field_zeros: Vec<f64>,      //see t_scales
  field_null: Vec<String>,    //null value format for each field
  field_dispfmt: Vec<String>, //display format for each field
}

impl HduOptions {
  pub fn new_invalid() -> Self {
    HduOptions {
      extension: Extension::Dump,
      conforming: false,
      extends: false,
      has_groups: false,
      inherits_main: false,
      bitpix: -1,
      n_axes: 0,
      shape: Vec::new(),
      parameter_count: 0,
      group_count: 0,
      param_scales: Vec::new(),
      param_zeros: Vec::new(),
      row_size: 0,
      column_start: Vec::new(),
      heap_size: 0,
      field_scales: Vec::new(),
      field_zeros: Vec::new(),
      field_null: Vec::new(),
      field_dispfmt: Vec::new(),
    }
  }

  pub fn determine_data_type(&self) -> Result<Extension, InvalidHeaderErr> {
    use Extension::*;
    use InvalidHeaderErr::*;

    let extension = if self.extension != PrimaryHdu {
      self.extension
    } else {
      if self.has_groups {
        Groups
      } else {
        Image
      }
    };

    match extension {
      Image => {
        //Check if the shape of the image is correct
        if self.n_axes as usize != self.shape.len() {
          return Err(NaxisOob { idx: self.shape.len(), naxes: self.n_axes });
        }
        //Check if the bitpix value is allowed for an image extension
        if self.bitpix == -1 {
          return Err(NoValue { key: crate::intern::fits_consts::BITPIX });
        } else if !matches!(self.bitpix, 8 | 16 | 32 | 64 | -32 | -64) {
          return Err(InvalidBitPix {
            bpx: self.bitpix as i64,
            allowed: &[8, 16, 32, 64, -32, -64],
          });
        }
        //Check if the pcount value was set to the correct value
        if self.parameter_count != 0 {
          return Err(InvalidPCount { xt: "Image", pc: self.parameter_count, allowed: "0" });
        }
        //Check if the gcount value was set to the correct value
        if self.group_count != 1 {
          return Err(InvalidGCount { xt: "Image", gc: self.parameter_count, allowed: "1" });
        }
        Ok(Image)
      }
      other => Err(UnsupportedExtension { xt: format!("{other:?}") }),
    }
  }
}
