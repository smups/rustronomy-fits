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
    fmt::{Display, Formatter, Write, self},
    error::Error
};

use ndarray::{Array, IxDyn};
use simple_error::SimpleError;

use crate::{raw::BlockSized, extensions::ExtensionPrint};

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
    DpfImg(Image<f64>)
}

impl BlockSized for TypedImage {
    fn get_block_len(&self) -> usize {
        match self {
            Self::ByteImg(var) => var.get_block_len(),
            Self::I16Img(var) => var.get_block_len(),
            Self::I32Img(var) => var.get_block_len(),
            Self::I64Img(var) => var.get_block_len(),
            Self::SpfImg(var) => var.get_block_len(),
            Self::DpfImg(var) => var.get_block_len()
        }
    }
}

impl ExtensionPrint for TypedImage {
    fn xprint(&self) -> String{
        let mut f = String::from("(IMAGE) - ");
        match self {
            Self::ByteImg(img) => {
               write!(f, "datatype: u8, shape: {}, size: {}",
                    img.pretty_print_shape(), img.get_block_len()
                )
            } Self::I16Img(img) => {
                write!(f, "datatype: i16, shape: {}, size: {}",
                    img.pretty_print_shape(), img.get_block_len()
                )
            } Self::I32Img(img) => {
                write!(f, "datatype: i32, shape: {}, size: {}",
                    img.pretty_print_shape(), img.get_block_len()
                )
            } Self::I64Img(img) => {
                write!(f, "datatype: i64, shape: {}, size: {}",
                    img.pretty_print_shape(), img.get_block_len()
                )
            } Self::SpfImg(img) => {
                write!(f, "datatype: f32, shape: {}, size: {}",
                    img.pretty_print_shape(), img.get_block_len()
                )
            } Self::DpfImg(img) => {
                write!(f, "datatype: f64, shape: {}, size: {}",
                    img.pretty_print_shape(), img.get_block_len()
                )
            }
       }.unwrap();
       return f;
    }
}

impl Display for TypedImage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
       //TODO: make pretty display for image!
       todo!()
    }
}

impl TypedImage {

    pub fn as_u8_array(&self) -> Result<&Array<u8, IxDyn>, Box<dyn Error>> {
        match &self {
            Self::ByteImg(img) => Ok(img.get_data()),
            &var => Err(Box::new(SimpleError::new(
                format!("Tried to borrow {:?} as u8 array", var)
            )))
        }
    }

    pub fn as_i16_array(&self) -> Result<&Array<i16, IxDyn>, Box<dyn Error>> {
        match &self {
            Self::I16Img(img) => Ok(img.get_data()),
            &var => Err(Box::new(SimpleError::new(
                format!("Tried to borrow {:?} as i16 array", var)
            )))
        }
    }

    pub fn as_i32_array(&self) -> Result<&Array<i32, IxDyn>, Box<dyn Error>> {
        match &self {
            Self::I32Img(img) => Ok(img.get_data()),
            &var => Err(Box::new(SimpleError::new(
                format!("Tried to borrow {:?} as i32 array", var)
            )))
        }
    }

    pub fn as_i64_array(&self) -> Result<&Array<i64, IxDyn>, Box<dyn Error>> {
        match &self {
            Self::I64Img(img) => Ok(img.get_data()),
            &var => Err(Box::new(SimpleError::new(
                format!("Tried to borrow {:?} as i64 array", var)
            )))
        }
    }

    pub fn as_f32_array(&self) -> Result<&Array<f32, IxDyn>, Box<dyn Error>> {
        match &self {
            Self::SpfImg(img) => Ok(img.get_data()),
            &var => Err(Box::new(SimpleError::new(
                format!("Tried to borrow {:?} as f32 array", var)
            )))
        }
    }

    pub fn as_f64_array(&self) -> Result<&Array<f64, IxDyn>, Box<dyn Error>> {
        match &self {
            Self::DpfImg(img) => Ok(img.get_data()),
            &var => Err(Box::new(SimpleError::new(
                format!("Tried to borrow {:?} as f64 array", var)
            )))
        }
    }

    pub fn as_owned_u8_array(self) -> Result<Array<u8, IxDyn>, Box<dyn Error>> {
        match self {
            Self::ByteImg(img) => Ok(img.get_data_owned()),
            var => Err(Box::new(SimpleError::new(
                format!("Tried to convert {:?} to an u8 array", var)
            )))
        }
    }

    pub fn as_owned_i16_array(self) -> Result<Array<i16, IxDyn>, Box<dyn Error>> {
        match self {
            Self::I16Img(img) => Ok(img.get_data_owned()),
            var => Err(Box::new(SimpleError::new(
                format!("Tried to convert {:?} to an i16 array", var)
            )))
        }
    }

    pub fn as_owned_i32_array(self) -> Result<Array<i32, IxDyn>, Box<dyn Error>> {
        match self {
            Self::I32Img(img) => Ok(img.get_data_owned()),
            var => Err(Box::new(SimpleError::new(
                format!("Tried to convert {:?} to an i32 array", var)
            )))
        }
    }

    pub fn as_owned_i64_array(self) -> Result<Array<i64, IxDyn>, Box<dyn Error>> {
        match self {
            Self::I64Img(img) => Ok(img.get_data_owned()),
            var => Err(Box::new(SimpleError::new(
                format!("Tried to convert {:?} to an i64 array", var)
            )))
        }
    }

    pub fn as_owned_f32_array(self) -> Result<Array<f32, IxDyn>, Box<dyn Error>> {
        match self {
            Self::SpfImg(img) => Ok(img.get_data_owned()),
            var => Err(Box::new(SimpleError::new(
                format!("Tried to convert {:?} to an f32 array", var)
            )))
        }
    }

    pub fn as_owned_f64_array(self) -> Result<Array<f64, IxDyn>, Box<dyn Error>> {
        match self {
            Self::DpfImg(img) => Ok(img.get_data_owned()),
            var => Err(Box::new(SimpleError::new(
                format!("Tried to convert {:?} to an f64 array", var)
            )))
        }
    }

}