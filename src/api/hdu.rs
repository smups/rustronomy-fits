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

use ndarray as nd;
use rustronomy_core::universal_containers::{MetaOnly, Table};

#[derive(Debug, Clone, Default, PartialEq)]
/// This struct represents the Header Data Unit (HDU) as described by the FITS
/// standard. See module-level documentation for details and examples.
pub struct Hdu {
  meta: Option<MetaOnly>,
  data: Option<HduData>,
}

impl Hdu {
  /// Returns new Hdu with no metadata and `data` as data component.
  pub fn new(data: impl Into<HduData>) -> Self {
    Hdu { meta: None, data: Some(data.into()) }
  }

  /// Returns reference to data held by this Hdu, if such data is present. If no
  /// data is present, or the data in the Hdu cannot be converted to the specified
  /// type, an error will be returned instead. This method does not panic.
  pub fn get_data<T>(&self) -> Result<&T, FromHduErr>
  where
    for<'a> &'a T: TryFrom<&'a HduData, Error = FromHduErr>,
  {
    let &data = &self.data.as_ref().ok_or(FromHduErr::NoDataErr)?;
    data.try_into()
  }

  /// Returns reference to data held by this Hdu, if such data is present. If no
  /// data is present, or the data in the Hdu cannot be converted to the specified
  /// type, an error will be returned instead. This method does not panic.
  pub fn get_data_mut<T>(&mut self) -> Result<&mut T, FromHduErr>
  where
    for<'a> &'a mut T: TryFrom<&'a mut HduData, Error = FromHduErr>,
  {
    let data = (&mut self.data).as_mut().ok_or(FromHduErr::NoDataErr)?;
    data.try_into()
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
  ArrayTypeErr { tried_into_type: &'static str, actual_type: String },
  VaraintErr { wrong_variant: String, correct_variant: &'static str },
  NoDataErr,
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
      }
      NoDataErr => write!(f, "HDU does not contain data"),
    }
  }
}
impl std::error::Error for FromHduErr {}

#[non_exhaustive]
#[derive(Debug, Clone)]
/// Enum representing the different kinds of data that can be held by a FITS file
/// (at least those that are currently implemented). `From<Type>` is implemented
/// for `HduData` for all the data types that can be held in a FITS file.
/// Similarly, `TryFrom<HduData>` is implemented for all `Type`s that can be held
/// by `HduData`.
pub enum HduData {
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

///Temporary impl of PartialEq for HduData that always returns false for tables
impl PartialEq for HduData {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (Self::ArrayU8(l0), Self::ArrayU8(r0)) => l0 == r0,
      (Self::ArrayI16(l0), Self::ArrayI16(r0)) => l0 == r0,
      (Self::ArrayI32(l0), Self::ArrayI32(r0)) => l0 == r0,
      (Self::ArrayI64(l0), Self::ArrayI64(r0)) => l0 == r0,
      (Self::ArrayF32(l0), Self::ArrayF32(r0)) => l0 == r0,
      (Self::ArrayF64(l0), Self::ArrayF64(r0)) => l0 == r0,
      (Self::Table(_), _) => false,
      _ => false,
    }
  }
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
  ($($variant:ident, $type:ty),*) => {$(
    impl<D: nd::Dimension> From<nd::Array<$type, D>> for HduData {
      fn from(data: nd::Array<$type, D>) -> Self {
        Self::$variant(data.into_dyn())
      }
    }

    impl<D: nd::Dimension> From<nd::Array<$type, D>> for Hdu {
      fn from(data: nd::Array<$type, D>) -> Self {
        Hdu { data: Some(data.into()), meta: None }
      }
    }
  )*};
}
into_hdu_data!(
  ArrayU8, u8, ArrayI16, i16, ArrayI32, i32, ArrayI64, i64, ArrayF32, f32, ArrayF64, f64
);

/*
  TryFrom implementations to turn Hdu's into other types
*/

impl TryFrom<HduData> for Table {
  type Error = FromHduErr;

  fn try_from(hdudata: HduData) -> Result<Self, Self::Error> {
    match hdudata {
      HduData::Table(table) => Ok(table),
      other => Err(FromHduErr::VaraintErr {
        wrong_variant: format!("{other:?}"),
        correct_variant: "Table",
      }),
    }
  }
}

//Implements TryFrom<array> for HduData for all the different kinds of arrays supported
//by the fits format
macro_rules! try_from_hdu {
  ($($variant:ident, $type:ty),*) => {$(
    impl TryFrom<HduData> for nd::ArrayD<$type> {
      type Error = FromHduErr;

      fn try_from(hdudata: HduData) -> Result<Self, Self::Error> {
        match hdudata {
          HduData::$variant(array) => Ok(array.into_dyn()),
          other => Err(FromHduErr::ArrayTypeErr{
            tried_into_type: std::any::type_name::<$type>(),
            actual_type: format!("{other:?}")
          }),
          _ => Err(FromHduErr::NoDataErr)
        }
      }
    }

    impl<'a> TryFrom<&'a HduData> for nd::ArrayViewD<'a, $type> {
      type Error = FromHduErr;

      fn try_from(hdudata: &'a HduData) -> Result<Self, Self::Error> {
        match hdudata {
          HduData::$variant(ref array) => Ok(array.view()),
          ref other => Err(FromHduErr::ArrayTypeErr{
            tried_into_type: std::any::type_name::<$type>(),
            actual_type: format!("{other:?}")
          })
        }
      }
    }

  )*};
}
try_from_hdu!(
  ArrayU8, u8, ArrayI16, i16, ArrayI32, i32, ArrayI64, i64, ArrayF32, f32, ArrayF64, f64
);

////////////////////////////////////////////////////////////////////////////////
//                                 UNIT TESTS                                 //
////////////////////////////////////////////////////////////////////////////////
macro_rules! test_from_hdudata_impl {
  ($(($type:ty, $test_name:ident, $answer:ident)),*) => {
    $(
      #[test]
      fn $test_name() {
        let test_array = nd::Array2::<$type>::zeros((10,10));
        let converted: HduData = test_array.clone().into();
        let correct = HduData::$answer(test_array.into_dyn());
        assert_eq!(converted, correct);
      }
    )*
  };
}
test_from_hdudata_impl!(
  (u8, from_hdudata_u8, ArrayU8),
  (i16, from_hdudata_i16, ArrayI16),
  (i32, from_hdudata_i32, ArrayI32),
  (i64, from_hdudata_i63, ArrayI64),
  (f32, from_hdudata_f32, ArrayF32),
  (f64, from_hdudata_f64, ArrayF64)
);

macro_rules! test_from_hdu_impl {
  ($(($type:ty, $test_name:ident)),*) => {
    $(
      #[test]
      fn $test_name() {
        let test_array = nd::Array2::<$type>::zeros((10,10));
        let converted: Hdu = test_array.clone().into();
        let correct = Hdu { data: Some(test_array.into()), meta: None };
        assert_eq!(converted, correct);
      }
    )*
  };
}
test_from_hdu_impl!(
  (u8, test_from_u8),
  (i16, test_from_i16),
  (i32, test_from_i32),
  (i64, test_from_i64),
  (f32, test_from_f32),
  (f64, test_from_f64)
);
