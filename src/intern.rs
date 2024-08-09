//Std imports
use std::error::Error;

use anyhow::Result;

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

pub struct FitsOptions {
  //Mandatory keywords
  conforming: bool,
  bitpix: i8, //-64 to +64
  dim: u16,        //number of axes. Max is 999
  shape: Vec<u16>, //each axis max size is 999
  pcount: u16,
  gcount: u16
}

pub fn read_primary_hdu(
  reader: &mut FitsReader,
) -> Result<(meta_only::MetaOnly, Hdu)> {
  //Max. number of records in a FITS block
  const MAX_RECS: usize = crate::BLOCK_SIZE / crate::RECORD_SIZE;

  //Metadata container to fill
  let mut metadata = Vec::new();
  let mut fits_options = FitsOptions {
    //all of these options are invalid
    conforming: false,
    bitpix: -9,
    dim: 0, //<- indicates an empty hdu
    //These are defaults
    shape: Vec::new(),
    pcount: 0,
    gcount: 1
  };

  let mut end = false;
  while !end {
    //(1) read a FITS block
    let block = reader.read_blocks(1)?;

    //(2) get all the records from the block
    let recs = decode_records(&block);

    //(3) If the decoder encountered the END keyword, there will be fewer than
    //the maximum number of records in recs.
    end = recs.len() != MAX_RECS;

    //(4) Pass the new records to the record parser
    parse_records(&recs, &mut metadata, &mut fits_options)?;
  }

  todo!()
}
