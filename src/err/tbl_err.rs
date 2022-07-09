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
  fmt::{self, Display, Formatter},
};

use crate::extensions::table::{AsciiTable, TableEntry};

#[derive(Debug)]
pub struct IndexOutOfRangeErr {
  index: (Option<usize>, usize),
  tbl_shape: (Option<usize>, usize),
}

impl Error for IndexOutOfRangeErr {}
impl Display for IndexOutOfRangeErr {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    match &self.index.0 {
      Some(col) => write!(
        f,
        "index (cols, rows) ({},{}) is out of range for table with shape ({},{})",
        col,
        &self.index.1,
        &self.tbl_shape.0.unwrap(),
        &self.tbl_shape.1
      ),
      None => write!(
        f,
        "index {} is out of range for column with length {}",
        &self.index.1, &self.tbl_shape.1
      ),
    }
  }
}

impl IndexOutOfRangeErr {
  pub(crate) fn new(index: (usize, usize), tbl: &AsciiTable) -> Self {
    IndexOutOfRangeErr {
      index: (Some(index.0), index.1),
      tbl_shape: (Some(tbl.get_shape().0), tbl.get_shape().1),
    }
  }
  pub(crate) fn from_idx(index: (Option<usize>, usize), shape: (Option<usize>, usize)) -> Self {
    IndexOutOfRangeErr { index: index, tbl_shape: shape }
  }
}

#[derive(Debug)]
pub struct ShapeMisMatchErr {
  row_len: usize,
  col_len: usize,
}

impl Error for ShapeMisMatchErr {}
impl Display for ShapeMisMatchErr {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "cannot add a row with {} fields to a table with {} columns",
      self.row_len, self.col_len
    )
  }
}

impl ShapeMisMatchErr {
  pub(crate) fn new(row: &Vec<TableEntry>, tbl: &AsciiTable) -> Self {
    ShapeMisMatchErr { row_len: row.len(), col_len: tbl.get_shape().0 }
  }
}

#[derive(Debug)]
pub struct TypeMisMatchErr {
  wrong_type: TableEntry,
  tbl_type: TableEntry,
}

impl Error for TypeMisMatchErr {}
impl Display for TypeMisMatchErr {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    write!(f, "cannot modify table of type {} with {}", self.tbl_type.type_print(), self.wrong_type)
  }
}

impl TypeMisMatchErr {
  pub(crate) fn new(tbl_type: TableEntry, wrong_type: &TableEntry) -> Self {
    TypeMisMatchErr { wrong_type: wrong_type.clone(), tbl_type: tbl_type }
  }
}

#[derive(Debug, Clone)]
pub struct TblDecodeErr {
  msg: String,
}

impl Error for TblDecodeErr {}
impl Display for TblDecodeErr {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.msg)
  }
}

impl From<TypeMisMatchErr> for TblDecodeErr {
  fn from(err: TypeMisMatchErr) -> Self {
    TblDecodeErr { msg: format!("{err}") }
  }
}

impl From<ShapeMisMatchErr> for TblDecodeErr {
  fn from(err: ShapeMisMatchErr) -> Self {
    TblDecodeErr { msg: format!("{err}") }
  }
}

impl From<IndexOutOfRangeErr> for TblDecodeErr {
  fn from(err: IndexOutOfRangeErr) -> Self {
    TblDecodeErr { msg: format!("{err}") }
  }
}
