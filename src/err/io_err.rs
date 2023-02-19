/*
  Copyright© 2022 Raúl Wolters(1)

  This file is part of rustronomy-fits.

  rustronomy is free software: you can redistribute it and/or modify it under
  the terms of the European Union Public License version 1.2 or later, as
  published by the European Commission.

  rustronomy is distributed in the hope that it will be useful, but WITHOUT ANY
  WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR
  A PARTICULAR PURPOSE. See the European Union Public License for more details.

  You should have received a copy of the EUPL in an/all official language(s) of
  the European Union along with rustronomy.  If not, see
  <https://ec.europa.eu/info/european-union-public-licence_en/>.

  (1) Resident of the Kingdom of the Netherlands; agreement between licensor and
  licensee subject to Dutch law as per article 15 of the EUPL.
*/

use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
/// This error may be thrown by a `FitsReader` when reading from a FITS file
pub enum FitsReadErr {
  /*
    This error may be thrown when opening a FITS file. If the FITS file has
    invalid encoding (for whatever reason), this error will be thrown.
  */
  /// An IO error occurred while decoding the file
  IOErr(std::io::Error),
  /// The size of the byte source is not a clean multiple of BLOCK_SIZE
  SourceNotBLockSized(usize),
  /// The size of the byte target is not a clean multiple of BLOCK_SIZE
  DestNotBlockSized(usize),
  /// Source contained fewer bytes than we requested to read
  EndOfSource { blcks_remain: usize, blcks_req: usize }
}

impl Display for FitsReadErr {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    use FitsReadErr::*;
    write!(f, "Error while reading from FITS file: ")?;
    match self {
      DestNotBlockSized(invalid_size) => {
        write!(
          f,
          "buffer size {invalid_size} not a multiple of BLOCK_SIZE ({} bytes)",
          crate::BLOCK_SIZE
        )
      }
      SourceNotBLockSized(invalid_size) => {
        write!(
          f,
          "file size {invalid_size} not a multiple of BLOCK_SIZE ({} bytes)",
          crate::BLOCK_SIZE
        )
      }
      EndOfSource { blcks_remain: file_size, blcks_req: blocks_read } => {
        write!(
          f,
          "tried to read {blocks_read} FITS blocks, but file is only {file_size} blocks long"
        )
      }
      IOErr(err) => {
        write!(f, "IO error: {err}")
      }
    }
  }
}
impl std::error::Error for FitsReadErr {}

impl From<std::io::Error> for FitsReadErr {
  fn from(err: std::io::Error) -> Self {
    Self::IOErr(err)
  }
}

#[derive(Debug)]
/// This error may be thrown by a `FitsWriter` when writing to a FITS file
pub enum FitsWriteErr {
  /// An IO error occured while writing the FITS file
  IOErr(std::io::Error),
  /// The provided byte source was not a clean multiple of BLOCK_SIZE
  SourceSize(usize)
}

impl Display for FitsWriteErr {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    use FitsWriteErr::*;
    write!(f, "Error while writing to FITS file: ")?;
    match self {
      SourceSize(invalid_size) => {
        write!(
          f,
          "buffer size {invalid_size} not a multiple of BLOCK_SIZE ({} bytes)",
          crate::BLOCK_SIZE
        )
      }
      IOErr(err) => {
        write!(f, "IO error: {err}")
      }
    }
  }
}
impl std::error::Error for FitsWriteErr {}

impl From<std::io::Error> for FitsWriteErr {
  fn from(err: std::io::Error) -> Self {
    Self::IOErr(err)
  }
}
