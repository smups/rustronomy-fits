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

//Get block size from root
const BLOCK_SIZE: usize = crate::BLOCK_SIZE; // = 2880B

use std::{
  error::Error,
  num::ParseIntError,
  str::{self, Utf8Error},
};

use anyhow::Result;

use crate::{
  extensions::{table::column::AsciiCol, Extension},
  raw::{
    raw_io::{RawFitsReader, RawFitsWriter},
    table_entry_format::TableEntryFormat,
  },
  tbl_fmt_err::{InvalidFFCode, ParseError},
};

use super::{column::Column, AsciiTable, TableEntry};

use rayon::prelude::*;

pub struct AsciiTblParser {}
impl AsciiTblParser {
  pub(crate) fn decode_tbl(
    reader: &mut RawFitsReader,
    chars_in_row: usize,               //#ASCII characters in a (raw) row
    rows_in_file: usize,               //#raw rows in the table
    fields_in_row: usize,              //#fields in each row
    row_index_col_start: Vec<usize>,   //row index where each column starts
    field_format: Vec<String>,         //data format (incl length) of each field
    field_labels: Option<Vec<String>>, //field labels
  ) -> Result<Extension> {
    /*  (1)
        Tables are usually pretty small compared to images. Hence it's
        probably ok to read the whole table in one go. We should be careful
        with reading to make sure we read a clean multiple of BLOCK_SIZE.
    */
    let byte_size = chars_in_row * rows_in_file;
    let mut num_blocks = byte_size / BLOCK_SIZE;
    if byte_size % BLOCK_SIZE != 0 {
      num_blocks += 1;
    } //leftover block

    //Actual reading
    let mut whole_table = vec![0u8; num_blocks * BLOCK_SIZE];
    reader.read_blocks(&mut whole_table)?;

    /*  (2)
        Next we have to figure out how the fields in each row are encoded.
        This information is contained within the field_format vec.
        Specifically, we want to know how long (in chars) each field in a row
        is and we want to set-up the typed table.
    */
    let fmts = field_format
      .iter()
      .map(|f| TableEntryFormat::from_fortran_format_code(f))
      .collect::<Result<Vec<TableEntryFormat>, ParseIntError>>()?;

    //(2a) Turn the formats into an vec of field sizes
    let field_lengs: Vec<usize> = fmts.iter().map(|fmt| fmt.get_field_width()).collect();

    //(2b) Turn the formats into a typed table
    let mut tbl = Self::setup_table(&fmts, field_labels, num_blocks)?;

    /*  (3)
        We may now divide the total raw file into row-sized chunks and process
        each row in a parallel fashion using rayon.

        Steps:
            1.  slice whole_table into row-sized chunks
            2.  slice row_sized chunks into field-sized chunks. Note that
                not all fields have the same size in bytes.
            3.  convert vector of byte slices to a vector of table-entries
            4.  add this formatted row to the formatted table

        Btw, 1 char = 1 byte in ASCII encoding
    */

    //(3a) first we split the rows in the table (steps 1 and 2)
    let split_rows_err = whole_table
      .par_chunks_exact(chars_in_row)
      .map(|raw| Self::split_row(raw, &row_index_col_start, &field_lengs))
      .collect::<Vec<Result<Vec<&str>, Utf8Error>>>();

    //Because of the limitations of rayon we must unpack the errors sequentially
    let mut split_rows: Vec<Vec<&str>> =
      split_rows_err.into_iter().collect::<Result<Vec<Vec<&str>>, Utf8Error>>()?;

    //Since we read in blocks of 2880 bytes, we might've read too much (some
    //rows may just contain zeroes). We fix this by throwing some rows away.
    split_rows.resize(rows_in_file, vec!["ERROR"]);

    //(3b) and then we decode the rows (step 3)
    let fmtd_rows_err = split_rows
      .into_par_iter()
      .map(|field_vec| {
        field_vec
          .into_iter()
          .enumerate()
          .map(|(i, st)| TableEntry::from_parts(st, &fmts[i]))
          .collect::<Result<Vec<TableEntry>, ParseError>>()
      })
      .collect::<Vec<Result<Vec<TableEntry>, ParseError>>>();

    //Because of the limitations of rayon we must unpack the errors sequentially
    let fmtd_rows =
      fmtd_rows_err.into_iter().collect::<Result<Vec<Vec<TableEntry>>, ParseError>>()?;

    //(3c) fill the table with the formatted rows (step 4)
    for row in fmtd_rows {
      tbl.add_row(row)?;
    }

    //(R) return the filled table
    Ok(Extension::AsciiTable(tbl))
  }

  fn setup_table(
    fmts: &Vec<TableEntryFormat>,
    labels: Option<Vec<String>>,
    size: usize,
  ) -> Result<AsciiTable, InvalidFFCode> {
    //(1) Use the column formats to set-up typed columns
    let mut cols = Vec::<Box<dyn AsciiCol>>::new();
    for i in 0..fmts.len() {
      match &fmts[i] {
        TableEntryFormat::Char(_) => {
          let label = match &labels {
            None => None,
            Some(vec) => Some(vec[i].clone()),
          };
          cols.push(Box::new(Column::<String>::new(label)));
        }
        TableEntryFormat::Int(_) => {
          let label = match &labels {
            None => None,
            Some(vec) => Some(vec[i].clone()),
          };
          cols.push(Box::new(Column::<i64>::new(label)));
        }
        TableEntryFormat::Float(_) => {
          let label = match &labels {
            None => None,
            Some(vec) => Some(vec[i].clone()),
          };
          cols.push(Box::new(Column::<f64>::new(label)));
        }
        TableEntryFormat::Invalid(invld) => {
          return Err(InvalidFFCode::new(invld.clone()));
        }
      }
    }

    //(R) yeet the columns in an (empty) table
    Ok(AsciiTable::new_sized(cols, size))
  }

  fn split_row<'a>(
    raw: &'a [u8],
    field_start: &'a Vec<usize>,
    field_len: &'a Vec<usize>,
  ) -> Result<Vec<&'a str>, Utf8Error> {
    let mut result = Vec::<&str>::new();
    for i in 0..field_start.len() {
      result.push(str::from_utf8(&raw[field_start[i]..(field_start[i] + field_len[i])])?);
    }
    Ok(result)
  }

  pub(crate) fn encode_tbl(
    tbl: AsciiTable,
    writer: &mut RawFitsWriter,
  ) -> Result<()> {
    /*  Note:
        This parser assumes that certain necessary keywords to decode a HDU
        containing a table have already been set while encoding the header
        of the HDU. The values of other table keywords are determined by this
        func, with the function returning their appropriate values.

        Note:
        This function takes ownership of the table, so we can do with it
        whatever we want without worrying about race conditions.

        Note: (some definitions)
        column WIDTH is the number of ascii characters needed to encode a
        single entry in the column.
        column LENGTH is the number of entries in a column.
    */

    /*  (1)
        We start by getting basic info about the table: how many columns
        does it have, what are their formats, etc...
    */
    let tbl_fmts = (&tbl).get_tbl_fmt(); //IN ORDER!
    let tbl_len = (&tbl).max_col_len();
    let mut cols: Vec<Vec<String>> = tbl.destroy(); //IN ORDER

    /*  (2)
        All columns in a FITS table should have the same length. Therefore,
        we need to add empty entries to each column that is smaller than the
        longest column.
    */
    cols.iter_mut().for_each(|col| {
      while col.len() < tbl_len {
        col.push(String::from(""));
      }
    });

    /*  (3)
        Furthermore, all rows in a single column must take up the same width
        in ascii characters. This means that we must extend the co
    */

    todo!()
  }
}
