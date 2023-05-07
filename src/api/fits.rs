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

/*
  This file is the "entry point" for the user-facing API
*/

//! The `Fits` struct represents a FITS file containing (multiple) Header Data
//! Units (HDU's). Each HDU is decoded as a Rustronomy universal data container.
//! A `Fits` struct can be turned into a `Vec<HDU>` and vice-versa.
//!  
//! ## **H**eader **D**ata **U**nits
//! FITS Header Data Units or HDU's are mapped to Rustronomy universal data
//! containers when opening a Fits file. Header tags are automatically transformed
//! into Rustronomy's `MetaDataTag`-like tags.
//!
//! FITS's Random groups, Tables and Binary Tables are all mapped to a Rustronomy
//! Table. FITS's Image data type is mapped to Rustronomy's `DataArray<T>`. To
//! obtain a Rustronomy `Image<T>`, one can
//!
//! ## Decoding a FITS file
//! rustronomy-fits decodes FITS files HDU by HDU. FITS metadata tags are
//! automatically transformed to rustronomy metadata tags in this step. Not all
//! FITS metadata tags that were present in the FITS file are mapped to
//! rustronomy metadata tags. In particular:
//! - strings spanning multiple FITS tags are automatically combined into a
//! single tag
//! - FITS tags used only in decoding the file are not present in the rustronomy
//! HDU (examples are BITPIX and the NAXIS tags)
//! - FITS tags that correspond to restricted rustronomy tags are mapped to those
//! tags, rather than general metadata ones.
//!
//! All FITS arrays are mapped to NDArrays of the appropriate type, conserving
//! FITS's column-major layout.

use crate::{
  api::{
    hdu::Hdu,
    io::{FitsReader, FitsWriter},
  },
  err::io_err::{FitsReadErr, FitsWriteErr},
};

#[derive(Debug, Default, Clone, PartialEq)]
/// User-facing struct representing a FITS file
pub struct Fits {
  data: Vec<Option<Hdu>>,
}

impl Fits {
  /// Attempts to create a `Fits` instance from the file at the supplied path.
  pub fn read_from_file(path: &std::path::Path) -> Result<Self, FitsReadErr> {
    todo!()
  }

  /// Attempts to write the current `Fits` instance to the file at the supplied
  /// path.
  pub fn write_to_file(path: &std::path::Path) -> Result<Self, FitsWriteErr> {
    todo!()
  }

  /// Attempts to read a FITS file from the supplied FitsReader.
  pub fn read_from(reader: &mut impl FitsReader) -> Result<Self, FitsReadErr> {
    todo!()
  }

  /// Attempts to write this FITS object using the supplied FitsWriter.
  pub fn write_to(&self, writer: &mut impl FitsWriter) -> Result<Self, FitsWriteErr> {
    todo!()
  }

  /// Returns the HDU at the specified slot number, leaving it unoccupied. The
  /// previous HDU stored at the specified slot number will be returned, if one
  /// was present. Does not panic.
  pub fn remove_hdu(&mut self, slotnr: usize) -> Option<Hdu> {
    Some(std::mem::take(self.data.get_mut(slotnr)?.as_mut()?))
  }

  /// Swaps HDU entry at slot number with a new hdu, returning the old HDU (if
  /// one was present). If slotnr was not assigned yet, the HDU is appended
  /// instead. Does not panic.
  pub fn swap_hdu(&mut self, hdu: &Hdu, slotnr: usize) -> Option<Hdu> {
    //First append the new hdu, then swap it with the to-be-removed hdu
    self.data.push(Some(hdu.clone()));
    if slotnr >= self.data.len() {
      None
    } else {
      // this panics if slotnr is oob, which we checked so no panic should be
      // possible
      self.data.swap_remove(slotnr)
    }
  }

  /// Adds a HDU at a new slot number one higher than the current highest.
  pub fn append_hdu(&mut self, hdu: &Hdu) {
    self.data.push(Some(hdu.clone()))
  }

  /// Cleans up unused HDU slots in the Fits struct. This operation does not
  /// preserve the order of the HDU's in the Fits file!
  pub fn clean(mut self) {
    self.data = self.data.into_iter().filter(|x| x.is_some()).collect();
  }
}

//TODO: impl display for Fits
