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

use std::fmt::{Debug, Display};

use ndarray::{Array, IxDyn};
use num_traits::Num;
use rustronomy_core::data_type_traits::io_utils::{Decode, Encode};

use crate::raw::BlockSized;

#[derive(Debug, Clone)]
pub struct Image<T>
where
  T: Debug + Num + Sized + Decode + Encode + Display + Clone,
{
  /*  THIS STRUCT IS NOT PART OF THE USER-FACING API
      None of the implementations or fields of this struct are public.
      Users interface with Images through the TypedImage enum and its impleme-
      ntations.
  */
  shape: Vec<usize>,
  data: Array<T, IxDyn>,
  block_size: usize,
}

impl<T> BlockSized for Image<T>
where
  T: Debug + Num + Sized + Decode + Encode + Display + Clone,
{
  fn get_block_len(&self) -> usize {
    self.block_size
  }
}

impl<T> Display for Image<T>
where
  T: Debug + Num + Sized + Decode + Encode + Display + Clone,
{
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    for (i,&n) in self.shape.iter().enumerate() {
      if i > 0 {
	write!(f,"x")?;
      }
      write!(f,"{}",n)?;
    }
    write!(f,"/{}",self.block_size)
  }
}

impl<T> Image<T>
where
  T: Debug + Num + Sized + Decode + Encode + Display + Clone,
{
  /*
      PUBLIC API
  */
  pub fn new(array: Array<T, IxDyn>) -> Self {
    todo!()
  }

  /*
      INTERNAL CODE
  */
  pub(crate) fn new_sized(shape: Vec<usize>, array: Array<T, IxDyn>, size: usize) -> Self {
    Image { shape: shape, data: array, block_size: size }
  }

  //Getters
  pub(crate) fn get_data(&self) -> &Array<T, IxDyn> {
    &self.data
  }
  pub(crate) fn get_data_owned(self) -> Array<T, IxDyn> {
    self.data
  }
  pub(crate) fn get_shape(&self) -> &Vec<usize> {
    &self.shape
  }

  pub(crate) fn pretty_print_shape(&self) -> String {
    let mut rsp = String::from("(");
    for ax in &self.shape {
      rsp += format!("{ax},").as_str();
    }
    rsp.pop(); //remove last comma
    String::from(rsp + ")")
  }
}
