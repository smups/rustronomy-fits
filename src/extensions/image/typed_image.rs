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
  fmt::{Display, Write},
};

use anyhow::Result;
use ndarray::{Array, IxDyn};

use crate::{
  bitpix::Bitpix, extensions::ExtensionPrint, img_err::WrongImgTypeErr as WITErr, raw::BlockSized,
};

use super::generic_image::Image;

#[derive(Debug, Clone)]
pub enum TypedImage {
  /*  THIS ENUM IS PART OF THE USER-FACING API
      Users obtain a TypedImage variant when parsing a FITS Image.
  */
  ByteImg(Image<u8>),
  I16Img(Image<i16>),
  I32Img(Image<i32>),
  I64Img(Image<i64>),
  SpfImg(Image<f32>),
  DpfImg(Image<f64>),
}

impl BlockSized for TypedImage {
  fn get_block_len(&self) -> usize {
    use TypedImage::*;
    match self {
      ByteImg(var) => var.get_block_len(),
      I16Img(var) => var.get_block_len(),
      I32Img(var) => var.get_block_len(),
      I64Img(var) => var.get_block_len(),
      SpfImg(var) => var.get_block_len(),
      DpfImg(var) => var.get_block_len(),
    }
  }
}

impl ExtensionPrint for TypedImage {
  fn xprint(&self) -> String {
    use TypedImage::*;
    let mut f = String::from("(IMAGE) - ");

    match self {
      ByteImg(img) => write!(
        f,
        "datatype: u8, shape: {}, size: {}",
        img.pretty_print_shape(),
        img.get_block_len()
      ),
      I16Img(img) => write!(
        f,
        "datatype: i16, shape: {}, size: {}",
        img.pretty_print_shape(),
        img.get_block_len()
      ),
      I32Img(img) => write!(
        f,
        "datatype: i32, shape: {}, size: {}",
        img.pretty_print_shape(),
        img.get_block_len()
      ),
      I64Img(img) => write!(
        f,
        "datatype: i64, shape: {}, size: {}",
        img.pretty_print_shape(),
        img.get_block_len()
      ),
      SpfImg(img) => write!(
        f,
        "datatype: f32, shape: {}, size: {}",
        img.pretty_print_shape(),
        img.get_block_len()
      ),
      DpfImg(img) => write!(
        f,
        "datatype: f64, shape: {}, size: {}",
        img.pretty_print_shape(),
        img.get_block_len()
      ),
    }
    .unwrap();

    return f;
  }
}

impl Display for TypedImage {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f,"TypedImage(")?;
    match self {
      Self::ByteImg(img) => write!(f,"u16,{}",img)?,
      Self::I16Img(img)=> write!(f,"i16,{}",img)?,
      Self::I32Img(img)=> write!(f,"i32,{}",img)?,
      Self::I64Img(img)=> write!(f,"i64,{}",img)?,
      Self::SpfImg(img)=> write!(f,"f32,{}",img)?,
      Self::DpfImg(img)=> write!(f,"f64,{}",img)?,
    }
    write!(f,")")
  }
}

impl TypedImage {
  pub(crate) fn bpx(&self) -> Bitpix {
    use Bitpix::*;
    use TypedImage::*;

    match self {
      ByteImg(_) => Byte,
      I16Img(_) => Short,
      I32Img(_) => Int,
      I64Img(_) => Long,
      SpfImg(_) => Spf,
      DpfImg(_) => Dpf,
    }
  }

  pub fn as_u8_array(&self) -> Result<&Array<u8, IxDyn>> {
    match &self {
      Self::ByteImg(img) => Ok(img.get_data()),
      &var => Err(WITErr::new(var, Bitpix::byte()).into()),
    }
  }

  pub fn as_i16_array(&self) -> Result<&Array<i16, IxDyn>> {
    match &self {
      Self::I16Img(img) => Ok(img.get_data()),
      &var => Err(WITErr::new(var, Bitpix::short()).into()),
    }
  }

  pub fn as_i32_array(&self) -> Result<&Array<i32, IxDyn>> {
    match &self {
      Self::I32Img(img) => Ok(img.get_data()),
      &var => Err(WITErr::new(var, Bitpix::int()).into()),
    }
  }

  pub fn as_i64_array(&self) -> Result<&Array<i64, IxDyn>> {
    match &self {
      Self::I64Img(img) => Ok(img.get_data()),
      &var => Err(WITErr::new(var, Bitpix::long()).into()),
    }
  }

  pub fn as_f32_array(&self) -> Result<&Array<f32, IxDyn>> {
    match &self {
      Self::SpfImg(img) => Ok(img.get_data()),
      &var => Err(WITErr::new(var, Bitpix::spf()).into()),
    }
  }

  pub fn as_f64_array(&self) -> Result<&Array<f64, IxDyn>> {
    match &self {
      Self::DpfImg(img) => Ok(img.get_data()),
      &var => Err(WITErr::new(var, Bitpix::dpf()).into()),
    }
  }

  pub fn as_owned_u8_array(self) -> Result<Array<u8, IxDyn>> {
    match self {
      Self::ByteImg(img) => Ok(img.get_data_owned()),
      var => Err(WITErr::new(&var, Bitpix::byte()).into()),
    }
  }

  pub fn as_owned_i16_array(self) -> Result<Array<i16, IxDyn>> {
    match self {
      Self::I16Img(img) => Ok(img.get_data_owned()),
      var => Err(WITErr::new(&var, Bitpix::short()).into()),
    }
  }

  pub fn as_owned_i32_array(self) -> Result<Array<i32, IxDyn>> {
    match self {
      Self::I32Img(img) => Ok(img.get_data_owned()),
      var => Err(WITErr::new(&var, Bitpix::int()).into()),
    }
  }

  pub fn as_owned_i64_array(self) -> Result<Array<i64, IxDyn>> {
    match self {
      Self::I64Img(img) => Ok(img.get_data_owned()),
      var => Err(WITErr::new(&var, Bitpix::long()).into()),
    }
  }

  pub fn as_owned_f32_array(self) -> Result<Array<f32, IxDyn>> {
    match self {
      Self::SpfImg(img) => Ok(img.get_data_owned()),
      var => Err(WITErr::new(&var, Bitpix::spf()).into()),
    }
  }

  pub fn as_owned_f64_array(self) -> Result<Array<f64, IxDyn>> {
    match self {
      Self::DpfImg(img) => Ok(img.get_data_owned()),
      var => Err(WITErr::new(&var, Bitpix::dpf()).into()),
    }
  }
}
