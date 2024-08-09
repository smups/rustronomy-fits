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

use anyhow::Result;

use crate::{
  extensions::ExtensionPrint,
  raw::{table_entry_format::TableEntryFormat, BlockSized},
  tbl_err::IndexOutOfRangeErr,
  tbl_err::ShapeMisMatchErr,
};

use super::{column::AsciiCol, TableEntry};

/*  Description:
    This is the abstracted user-facing api for tables. The
*/
#[derive(Debug, Clone)]
pub struct AsciiTable {
  cols: Vec<Box<dyn AsciiCol>>,
  block_size: Option<usize>,
}

impl BlockSized for AsciiTable {
  fn get_block_len(&self) -> usize {
    match self.block_size {
      Some(size) => size,
      None => {
        //We have to calculate the size of the table manually, as it is
        //not currently known (this is the case for user-created tables)
        todo!()
      }
    }
  }
}

impl Display for AsciiTable {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    writeln!(
      f,
      ">================================<|FITS Table|>================================="
    )?;
    writeln!(f, ">Table Layout:")?;
    for (index, col) in self.cols.iter().enumerate() {
      writeln!(f, ">  col#{index:03} - {}", col.as_ref().pretty_print())?
    }
    writeln!(
      f,
      ">==============================================================================="
    )?;
    Ok(())
  }
}

impl ExtensionPrint for AsciiTable {
  fn xprint(&self) -> String {
    format!(
      "(TABLE) - #columns: {}, #rows: {}, size: {}",
      self.cols.len(),
      match self.cols.get(0) {
        None => 0,
        Some(col_ref) => col_ref.len(),
      },
      self.get_block_len()
    )
  }
}

impl AsciiTable {
  /*
      PUBLIC API
  */

  pub fn get_entry(&self, col: usize, row: usize) -> Result<TableEntry, IndexOutOfRangeErr> {
    //returns an entry in the table, if it exists

    //(1) Check if the column index is valid -> if yes, get the column
    if col >= self.cols.len() {
      return Err(IndexOutOfRangeErr::new((col, row), self));
    }
    let column = self.cols.get(col).unwrap().as_ref();

    //(2) get the entry from the column
    match column.get_entry(row) {
      Some(entry) => Ok(entry),
      None => Err(IndexOutOfRangeErr::new((col, row), self)),
    }
  }

  pub fn get_shape(&self) -> (usize, usize) {
    //returns shape (columns, rows) of table
    (self.cols.len(), self.max_col_len())
  }

  pub fn get_fmtd_column(&self, col: usize) -> Option<Vec<String>> {
    match self.cols.get(col) {
      None => None,
      Some(column) => Some(column.to_ascii_vec()),
    }
  }

  /*
      INTERNAL FUNCS
  */

  pub(crate) fn get_col_fmt(&self, col: usize) -> Option<TableEntryFormat> {
    match self.cols.get(col) {
      None => None,
      Some(column) => Some(column.get_col_fmt()),
    }
  }

  pub(crate) fn get_tbl_fmt(&self) -> Vec<TableEntryFormat> {
    self.cols.iter().map(|col| col.get_col_fmt()).collect()
  }

  pub(crate) fn new_sized(cols: Vec<Box<dyn AsciiCol>>, size: usize) -> Self {
    //creates new table with known blocksize
    AsciiTable { cols: cols, block_size: Some(size) }
  }

  pub(crate) fn add_row(&mut self, row: Vec<TableEntry>) -> Result<()> {
    //Adds row to table
    if row.len() != self.cols.len() {
      return Err(ShapeMisMatchErr::new(&row, &self).into());
    }

    //Add row to the table
    for (index, entry) in row.into_iter().enumerate() {
      self.cols[index].push_entry(entry)?;
    }

    //(R) ok
    Ok(())
  }

  pub(crate) fn destroy(self) -> Vec<Vec<String>> {
    //destructs table into columns of strings
    self.cols.into_iter().map(|val| val.to_ascii_vec()).collect()
  }

  pub(crate) fn max_col_len(&self) -> usize {
    //returns size of longest column in table
    self.cols.iter().fold(0, |max_len, col| max_len.max(col.len()))
  }
}
