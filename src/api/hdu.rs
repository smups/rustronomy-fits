/*
  Copyright © 2022 Raúl Wolters

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

//! This module contains the `Hdu` enum representing the FITS Header Data Unit
//! (HDU) and accompanying types.
//!
//! ## The `Hdu` enum
//! The `Hdu` enum can contain either:
//! 1. No data, only metadata (`Hdu::NoData`)
//! 2. An array (`Hdu::Array`)
//! 3. A table (`Hdu::Table`)
//! The FITS image extension is mapped to the `Hdu::Array` variant and the table,
//! bintable and random group extensions are mapped to the `Hdu::Table` variant.
//! The `Array` variant can be converted into a rustronomy `DataArray<T>`, where
//! `T` corresponds to the type of the FITS image. The `Table` variant contains
//! a rustronomy `Table`.
//!
//! ## Accessing a FITS Image HDU
//! The `Hdu::Array` variant contains another enum, called `TypedArray`. The
//! `TypedArray` enum implements the `TryFrom<DataArray<T>>` and can thus be
//! turned into a generic rustronomy `DataArray<T>`. For example:
//! ```
//! let hdu = Fits::read("some_file.fits")?.get_hdu(0)?;
//! let typed_array = match hdu {
//!   Array(array) => array,
//!   _ => panic!("this HDU does not contain an array!")
//! };
//! let generic_array: DataArray<f64> = typed_array.try_into()?;
//! ```
//!
//! It is often more ergonomic to directly turn the `Hdu` instance into a
//! `DataArray<T>` if you do not care about the metadata. For example:
//! ```
//! let hdu = Fits::read("some_file.fits")?.get_hdu(0)?;
//! let array: DataArray<f64> = hdu.try_into()?;
//! ```

use std::fmt::{Display, Formatter};

use rustronomy_core::universal_containers::{MetaOnly, Table};
use ndarray as nd;

#[derive(Debug, Clone, PartialEq, Default)]
/// This struct represents the Header Data Unit (HDU) as described by the FITS
/// standard. See module-level documentation for details and examples.
pub struct Hdu {
  meta: Option<MetaOnly>,
  data: Option<HduData>
}

impl Hdu {
  /// Returns new Hdu with no metadata and `data` as data component.
  pub fn new(data: impl Into<HduData>) -> Self {
    Hdu { meta: None, data: Some(data.into()) }
  }

  pub fn get_data<T: TryFrom<Hdu>>(&self) -> Result<&T, <T as TryFrom<Hdu>>::Error> {
    &self.data.ok_or(0)?.try_into()
  }

  /// Constructs Hdu from HduData and MetaOnly components
  pub fn from_parts(data: HduData, meta: MetaOnly) -> Self {
    Hdu { meta: Some(meta), data: Some(data) }
  }

  /// Deconstructs Hdu into HduData and MetaOnly components;
  pub fn to_parts(self) -> (Option<HduData>, Option<MetaOnly>) {
    (self.data, self.meta)
  }
}

#[derive(Debug, Clone)]
pub enum FromHduErr {
  ArrayTypeErr { tried_into_type: &'static str, actual_type: &'static str },
  VaraintErr { wrong_variant: String, correct_variant: &'static str },
  NoDataErr
}

impl Display for FromHduErr {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    use FromHduErr::*;

    match self {
      ArrayTypeErr { tried_into_type, actual_type } => {
        write!(f, "Expected DataArray<{tried_into_type}>, found DataArray<{actual_type}>")
      }
      VaraintErr { wrong_variant, correct_variant } => {
        write!(f, "HDU contains invalid variant Hdu::{wrong_variant}, expected to find Hdu::{correct_variant}")
      },
      NoDataErr => write!(f, "HDU does not contain data")
    }
  }
}
impl std::error::Error for FromHduErr {}

#[derive(Debug, Clone)]
enum HduData {
  //Array types allowed by the FITS standard
  ArrayU8(nd::ArrayD<u8>),
  ArrayI16(nd::ArrayD<i16>),
  ArrayI32(nd::ArrayD<i32>),
  ArrayI64(nd::ArrayD<i64>),
  ArrayF32(nd::ArrayD<f32>),
  ArrayF64(nd::ArrayD<f64>),
  //(binary) tables
  Table(Table),
}

/*
  From implementations to create meta-only hdu's
*/

impl From<MetaOnly> for Hdu {
  fn from(value: MetaOnly) -> Self {
    Self { meta: Some(value), data: None }
  }
}

impl From<Table> for Hdu {
  fn from(data: Table) -> Self {
    Self { meta: None, data: Some(HduData::Table(data)) }
  }
}

//Implements From<array> for HduData for all the different kinds of arrays supported
//by the fits format
macro_rules! into_hdu_data {
  ($($variant:ident, $type:ty),*) => {
    $(impl<D: nd::Dimension> From<nd::Array<$type, D>> for HduData {
      fn from(data: nd::Array<$type, D>) -> Self {
        Self::$variant(data.into_dyn())
      }
    })*
  };
}
into_hdu_data!(ArrayU8, u8, ArrayI16, i16, ArrayI32, i32, ArrayI64, i64, ArrayF32, f32, ArrayF64, f64);

/*
  TryFrom implementations to turn Hdu's into other types
*/

impl TryFrom<Hdu> for Table {
  type Error = FromHduErr;

  fn try_from(value: Hdu) -> Result<Self, Self::Error> {
    match value {
      Hdu { meta, data: Some(HduData::Table(table)) } => Ok(table),
      Hdu { meta, data: Some(other) } => Err(FromHduErr::VaraintErr{
        wrong_variant: format!("{other:?}"),
        correct_variant: "Table"
      }),
      _ => Err(FromHduErr::NoDataErr)
    }
  }
}

//Implements TryFrom<array> for HduData for all the different kinds of arrays supported
//by the fits format
macro_rules! try_from_hdu {
  ($($variant:ident, $type:ty),*) => {
    $(impl<D: nd::Dimension> TryFrom<Hdu> for nd::Array<$type, D> {
      type Error = FromHduErr;

      fn try_from(hdu: Hdu) -> Result<Self, Self::Error> {
        match hdu.data {
          Some(HduData::$variant(array)) => Ok(array),
          Some(other) => Err(FromHduErr::ArrayTypeErr{
            tried_into_type: std::any::type_name::<$type>(),
            actual_type: (other as &dyn std::any::Any).type_name()
          }),
          _ => Err(FromHduErr::NoDataErr)
        }
      }
    })*
  };
}
try_from_hdu!(ArrayU8, u8, ArrayI16, i16, ArrayI32, i32, ArrayI64, i64, ArrayF32, f32, ArrayF64, f64);

////////////////////////////////////////////////////////////////////////////////
//                                 UNIT TESTS                                 //
////////////////////////////////////////////////////////////////////////////////
macro_rules! test_from_impl {
  ($($type:ty),*) => {
    $(
      #[test]
      fn test_from() {
        let test_array = nd::Array2::<$type>::zeros((10,10));
        let hdu_data: HduData = test_array.into();
      }
    )*
  };
}
test_from_impl!(u8);