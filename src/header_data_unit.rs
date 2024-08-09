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

use core::fmt;
use std::{borrow::Cow, error::Error, fmt::Display};
use anyhow::Result;

use crate::{
  bitpix::Bitpix,
  extensions::{image::ImgParser, table::AsciiTblParser, Extension},
  hdu_err::*,
  header::Header,
  raw::{
    raw_io::{RawFitsReader, RawFitsWriter},
    BlockSized,
  },
};

const VALID_EXTENSION_NAMES: [&'static str; 3] = ["'IMAGE   '", "'TABLE   '", "'BINTABLE'"];

#[derive(Debug, Clone)]
pub struct HeaderDataUnit {
  header: Header,
  data: Option<Extension>,
}

impl HeaderDataUnit {
  /*
      INTERNAL CODE
  */

  pub(crate) fn decode_hdu(raw: &mut RawFitsReader) -> Result<Self> {
    //(1) Read the header
    let header = Header::decode_header(raw)?;

    //(2) Read data, if there is any
    let extension = match &header.get_value("XTENSION") {
      None => {
        /*  (2a)
            This is the primary header (or there is simply no data in
            this hdu). This means that this HDU may contain random
            groups. Random groups and emtpy arrays have the NAXIS
            keyword set to zero.
        */
        if header.get_value_as::<usize>("NAXIS")? == 0 {
          //For now I'll just return None rather than implement random
          //groups
          None
        } else {
          //Image
          Some(Self::read_img(raw, &header)?)
        }
      }
      Some(extension_type) => {
        /*  (2b)
            This is not a primary header, but the header of an extension
            hdu.
        */
        match extension_type.as_str() {
          "'IMAGE   '" => Some(Self::read_img(raw, &header)?),
          _kw @ "'TABLE   '" => Some(Self::read_table(raw, &header)?),
          kw @ "'BINTABLE'" => Err(Self::not_impl(kw))?,
          kw => Err(InvalidRecordValueError::new("XTENSION", kw, &VALID_EXTENSION_NAMES))?,
        }
      }
    };

    //(3) return complete HDU
    Ok(HeaderDataUnit { header: header, data: extension })
  }

  fn read_table(raw: &mut RawFitsReader, header: &Header) -> Result<Extension> {
    /*
        To parse a table we need to know the following keywords:
            TFIELDS => #fields in a row
            NAXIS1 => #characters in a row
            NAXIS2 => #rows in the table
            TBCOL{i} => starting index of field i
            TFORM{i} => data format of field i
            TTYPE{i} => name of field i (not required)
        In addition, we require the following keywords to have been set to:
            NAXIS == 2
            BITPIX == 8
            PCOUNT == 0
            GCOUNT == 1
        We obtain these values from the header
    */

    //(1) check that the mandatory keywords have been set properly
    let naxis: usize = header.get_value_as("NAXIS")?;
    let bitpix: isize = header.get_value_as("BITPIX")?;
    let pcount: usize = header.get_value_as("PCOUNT")?;
    let gcount: usize = header.get_value_as("GCOUNT")?;
    //Here come the if statements :c
    if naxis != 2 {
      Err(InvalidRecordValueError::new("NAXIS", &format!("{naxis}"), &["2"]))?
    }
    if bitpix != 8 {
      Err(InvalidRecordValueError::new("BITPIX", &format!("{bitpix}"), &["8"]))?
    }
    if pcount != 0 {
      Err(InvalidRecordValueError::new("PCOUNT", &format!("{pcount}"), &["0"]))?
    }
    if gcount != 1 {
      Err(InvalidRecordValueError::new("GCOUNT", &format!("{gcount}"), &["1"]))?
    }

    //(2) Obtain the keywords required for decoding the header
    let nfields: usize = header.get_value_as("TFIELDS")?;
    let row_len: usize = header.get_value_as("NAXIS1")?;
    let nrows: usize = header.get_value_as("NAXIS2")?;

    let mut row_index_col_start: Vec<usize> = Vec::new();
    for i in 1..=nfields {
      row_index_col_start.push(
        //We have to substract 1 since FITS indices start at 1 rather
        //than 0
        header.get_value_as::<usize>(&format!("TBCOL{i}"))? - 1,
      );
    }

    let mut field_format: Vec<String> = Vec::new();
    for i in 1..=nfields {
      field_format.push(header.get_value_as(&format!("TFORM{i}"))?)
    }

    let labels = match header.get_value("TTYPE1") {
      None => None,
      Some(_) => {
        /*
            This header contains descriptive keywords for the entries
            in the table. Note that the TTYPE{i} keywords are themselves
            keywords, and the actual desciptions of the columns are
            stored in the header behind these keywords.
        */
        let mut tmp: Vec<String> = Vec::new();
        for i in 1..=nfields {
          tmp.push(header.get_value_as(&format!("TTYPE{i}"))?);
        }
        Some(
          //Before we return, we query keywords we've found so far
          tmp
            .into_iter()
            .map(|mut ttype_keyword| {
              //We still have to strip the keyword of its annoying
              //{'keyword   '} syntax
              ttype_keyword.remove(0);
              ttype_keyword.pop();
              header.get_value_as(ttype_keyword.trim())
            })
            .collect::<Result<Vec<String>>>()?,
        )
      }
    };

    //(3) Decode the image using the table parser
    let tbl = AsciiTblParser::decode_tbl(
      raw,
      row_len,
      nrows,
      nfields,
      row_index_col_start,
      field_format,
      labels,
    )?;

    //(R) return the completed table
    Ok(tbl)
  }

  fn read_img(raw: &mut RawFitsReader, header: &Header) -> Result<Extension> {
    //Let's start by getting the number of axes from the NAXIS keyword
    let naxis: usize = header.get_value_as("NAXIS")?;

    //Axis sizes are encoded in the NAXIS{i} keywords
    let mut axes: Vec<usize> = Vec::new();
    for i in 1..=naxis {
      axes.push(header.get_value_as(&format!("NAXIS{i}"))?);
    }

    //Datatype is encoded in the BITPIX keyword
    let bitpix = Bitpix::from_code(&header.get_value_as("BITPIX")?)?;

    //Now do the actual decoding of the image:
    Ok(ImgParser::decode_img(raw, &axes, bitpix)?)
  }

  pub(crate) fn encode_hdu(self, writer: &mut RawFitsWriter) -> Result<()> {
    //(1) Write header
    self.header.encode_header(writer)?;

    //(2) If we have data, write the data
    match self.data {
      Some(data) => data.write_to_buffer(writer)?,
      _ => {} //no data, do nothing
    }

    //(R) ok
    Ok(())
  }

  fn not_impl(keyword: &str) -> Box<NotImplementedErr> {
    Box::new(NotImplementedErr::new(keyword.to_string()))
  }

  /*
      USER-FACING API STARTS HERE
  */

  //Some simple getters
  pub fn get_header(&self) -> &Header {
    &self.header
  }
  pub fn get_data(&self) -> Option<&Extension> {
    self.data.as_ref()
  }

  //Destructs HDU into parts
  pub fn to_parts(self) -> (Header, Option<Extension>) {
    (self.header, self.data)
  }

  pub fn pretty_print_header(&self) -> String {
    format!(
      "[Header] - #records: {}, size: {}",
      self.header.get_block_len(),
      self.header.get_num_records()
    )
  }

  pub fn pretty_print_data(&self) -> String {
    let data_string: Cow<str> = match &self.data {
      None => "(NO_DATA)".into(),
      Some(data) => format!("{data}").into(),
    };
    format!("[Data] {data_string}")
  }
}

impl BlockSized for HeaderDataUnit {
  fn get_block_len(&self) -> usize {
    self.header.get_block_len()
      + match &self.data {
        None => 0,
        Some(data) => data.get_block_len(),
      }
  }
}

impl Display for HeaderDataUnit {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.pretty_print_header())?;
    write!(f, "{}", self.pretty_print_data())?;
    Ok(())
  }
}
