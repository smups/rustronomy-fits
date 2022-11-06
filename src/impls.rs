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

fn read_blocks(file: &mut std::fs::File, n_blocks: usize) -> Result<Vec<u8>, Box<dyn Error>> {
  //(1) create buffer
  let mut buffer = Vec::with_capacity(n_blocks * crate::BLOCK_SIZE);

  //(2)
}

pub fn read_primary_hdu(
  file: &mut std::fs::File,
) -> Result<(meta_only::MetaOnly, Hdu), Box<dyn Error>> {
  todo!()
}
