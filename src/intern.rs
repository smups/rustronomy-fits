//Std imports
use std::error::Error;

//external imports
use rustronomy_core::universal_containers::*;

//internal imports
use crate::api::hdu::Hdu;

//module structure
mod fits_io;

//re-exports
pub use fits_io::*;

pub fn read_primary_hdu(
  reader: &mut FitsReader,
) -> Result<(meta_only::MetaOnly, Hdu), Box<dyn Error>> {
  //(1) Let's start by reading the first HDU block
  let block0 = reader.read_blocks(1)?;

  
  todo!()
}
