//Std imports
use std::error::Error;

//external imports
use rustronomy_core::universal_containers::*;

//internal imports
use crate::api::hdu::Hdu;
use header_utils::*;

//module structure
mod fits_io;
mod header_utils;

//re-exports
pub use fits_io::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FitsOptions {
  conforming: bool,
  bitpix: i8, //-64 to +64
  extends: bool,
  dim: u16,        //number of axes. Max is 999
  shape: Vec<u16>, //each axis max size is 999
}

impl FitsOptions {
  pub fn new_invalid() -> Self {
    FitsOptions { conforming: false, bitpix: 0, extends: false, dim: 0, shape: Vec::new() }
  }
}

pub fn read_primary_hdu(
  reader: &mut FitsReader,
) -> Result<(meta_only::MetaOnly, Hdu), Box<dyn Error>> {
  //Max. number of records in a FITS block
  const MAX_RECS: usize = crate::BLOCK_SIZE / crate::RECORD_SIZE;

  //(1) Read all the raw bytes in the header
  let header_bytes = read_header(reader)?;

  //(2) Split the raw bytes into FITS keyword-records. These 80-byte chunks
  //  consist of a key, an optional value and an optional comment.
  let raw_records = split_records(&header_bytes);
  todo!()
}
