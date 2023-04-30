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

use crate::api::hdu::Hdu;

#[derive(Debug)]
/// User-facing struct representing a FITS file
pub struct Fits {
  data: Vec<Option<Hdu>>,
}

impl Fits {
  /// Returns the HDU at the specified slot number. If the slot number is
  /// unoccupied, None is returned.
  pub fn remove_hdu(&mut self, slotnr: usize) -> Option<Hdu> {
    Some(std::mem::take(self.data.get_mut(idx)?))
  }
}

//TODO: impl display for Fits