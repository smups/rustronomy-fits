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

use rustronomy_core::universal_containers::*;

#[derive(Debug, Clone)]
/// This struct represents the Header Data Unit (HDU) as described by the FITS
/// standard. See module-level documentation for details and examples.
pub enum Hdu {
  NoData(meta_only::MetaOnly),
  Array(TypedArray),
  Table(Table),
}

impl Hdu {
  /// Create a Header Data Unit (HDU) containing no data and only metadata.
  pub fn empty(metadata: meta_only::MetaOnly) -> Self {
    Self::NoData(metadata)
  }

  /// Clones the metadata of the underlying Header Data Unit (HDU) and returns it
  /// as the type `meta_only::MetaOnly`.
  pub fn clone_meta(&self) -> meta_only::MetaOnly {
    use Hdu::*;
    match self {
      NoData(meta) => meta.clone(),
      Table(tab) => tab.clone_metadata(),
      Array(arr) => arr.clone_meta(),
    }
  }
}

#[derive(Debug, Clone)]
pub enum FromHduErr {
  ArrayTypeErr { tried_into_type: &'static str, actual_type: &'static str },
  VaraintErr { correct_variant: &'static str },
}

impl Display for FromHduErr {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    use FromHduErr::*;

    match self {
      ArrayTypeErr { tried_into_type, actual_type } => {
        write!(f, "Expected DataArray<{tried_into_type}>, found DataArray<{actual_type}>")
      }
      VaraintErr { correct_variant } => {
        write!(f, "Hdu contains invalid variant, expected to find Hdu::{correct_variant}")
      }
    }
  }
}
impl std::error::Error for FromHduErr {}

impl From<Table> for Hdu {
  fn from(data: Table) -> Self {
    Self::Table(data)
  }
}

impl TryFrom<Hdu> for Table {
  type Error = FromHduErr;

  fn try_from(value: Hdu) -> Result<Self, Self::Error> {
    if let Hdu::Table(table) = value {
      return Ok(table);
    } else {
      return Err(FromHduErr::VaraintErr { correct_variant: "Table" });
    }
  }
}

impl<T> From<DataArray<T>> for Hdu
where
  DataArray<T>: Into<TypedArray>,
  T: num_traits::Num,
{
  fn from(data: DataArray<T>) -> Self {
    Self::Array(data.into())
  }
}

impl<T> TryFrom<Hdu> for DataArray<T>
where
  DataArray<T>: TryFrom<TypedArray>,
  <DataArray<T> as TryFrom<TypedArray>>::Error: Into<FromHduErr>,
  T: num_traits::Num,
{
  type Error = FromHduErr;

  fn try_from(value: Hdu) -> Result<Self, FromHduErr> {
    if let Hdu::Array(data) = value {
      return match data.try_into() {
        Ok(good) => Ok(good),
        Err(err) => Err(err.into()),
      };
    } else {
      return Err(FromHduErr::VaraintErr { correct_variant: "Array" });
    }
  }
}

#[derive(Debug, Clone)]
/// This enum represents all `DataArray<T>`'s that a valid FITS file can contain.
/// As of the FITS standard v4.0, the valid types for `T` are: `u8`, `i16`, `i32`,
/// `i64`, `f32` and `f64`. For those types, `TypedArray` implements
/// `From<DataArray<T>>` and `DataArray<T>` implements `TryFrom<TypedArray>`.
pub enum TypedArray {
  U8(DataArray<u8>),
  I16(DataArray<i16>),
  I32(DataArray<i32>),
  I64(DataArray<i64>),
  F32(DataArray<f32>),
  F64(DataArray<f64>),
}

impl TypedArray {
  pub(crate) fn clone_meta(&self) -> meta_only::MetaOnly {
    use TypedArray::*;
    match self {
      U8(arr) => arr.clone_metadata(),
      I16(arr) => arr.clone_metadata(),
      I32(arr) => arr.clone_metadata(),
      I64(arr) => arr.clone_metadata(),
      F32(arr) => arr.clone_metadata(),
      F64(arr) => arr.clone_metadata(),
    }
  }
}

macro_rules! try_into_generic_array {
  ($($variant:ident, $type:ty),*) => {
    $(impl TryFrom<TypedArray> for DataArray<$type> {
      type Error = FromHduErr;

      fn try_from(data: TypedArray) -> Result<DataArray<$type>, Self::Error> {

        fn helper<T>(_:&T) -> &'static str {std::any::type_name::<T>()}

        if let TypedArray::$variant(data) = data {
          return Ok(data)
        } else {
          return Err(FromHduErr::ArrayTypeErr {
            tried_into_type: std::any::type_name::<$type>(),
            actual_type: helper(&data)
          })
        }
      }
    })*
  };
}
try_into_generic_array!(U8, u8, I16, i16, I32, i32, I64, i64, F32, f32, F64, f64);

macro_rules! into_typed_array {
  ($($variant:ident, $type:ty),*) => {
    $(impl From<DataArray<$type>> for TypedArray {
      fn from(data: DataArray<$type>) -> Self {
        Self::$variant(data)
      }
    })*
  };
}
into_typed_array!(U8, u8, I16, i16, I32, i32, I64, i64, F32, f32, F64, f64);

////////////////////////////////////////////////////////////////////////////////
//                                 UNIT TESTS                                 //
////////////////////////////////////////////////////////////////////////////////

#[test]
fn test_impl_hdu_clone_meta() {
  use rustronomy_core::universal_containers::*;
  let exp = 1234;
  let mut mock_meta = meta_only::MetaOnly::new();
  mock_meta.insert_exposure_time(exp).expect("could not insert mock tag");

  let hdu = Hdu::NoData(mock_meta);
  assert_eq!(exp, hdu.clone_meta().remove_exposure_time().unwrap());
}
