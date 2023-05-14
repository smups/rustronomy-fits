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

use std::{
  fs::{self, File},
  io::{Read, Write},
  path::Path,
};

use crate::{api::io::*, err::io_err::*};

//Get block size from root
const BLOCK_SIZE: usize = crate::intern::fits_consts::BLOCK_SIZE;

#[derive(Debug)]
pub struct FitsFileReader {
  pub file_meta: fs::Metadata,
  block_index: usize,
  n_fits_blocks: usize,
  reader_handle: File,
}

impl FitsFileReader {
  /// Creates a new `FitsReader` that will read bytes from the specified path.
  /// Returns an error if the provided path is not a file, or if the provided
  /// path does not exist.
  ///
  /// # Returns
  /// Returns `FitsReader` instance if the path is valid, or a `FitsReadErr`
  ///
  /// # Panics
  /// Does not panic
  pub fn new(path: &Path) -> Result<Self, FitsReadErr> {
    //(1) Open the file
    let reader_handle = File::open(path)?;

    //(2) Get metadata -> number of fits blocks
    let file_meta = reader_handle.metadata()?;
    let file_size = file_meta.len() as usize;

    if file_size % BLOCK_SIZE != 0 {
      //Throw an error for files that are not integer multiples of 2880
      return Err(FitsReadErr::SourceNotBLockSized(file_size));
    }
    let n_fits_blocks = file_size / BLOCK_SIZE;

    //Return file as raw FITS
    Ok(FitsFileReader { file_meta, block_index: 0, n_fits_blocks, reader_handle })
  }
}

impl FitsReader for FitsFileReader {
  fn read_blocks_into(&mut self, buffer: &mut [u8]) -> Result<usize, FitsReadErr> {
    //(1) Calculate how many header blocks we have to read
    let n_blocks = buffer.len() / BLOCK_SIZE;

    //(2) Check if the buffer is an integer multiple of a FITS block
    if n_blocks * BLOCK_SIZE != buffer.len() {
      return Err(FitsReadErr::DestNotBlockSized(buffer.len()));
    }

    //(3) Check if the number of header blocks we need to read does not exceed
    //the number of header blocks still left in the file
    if n_blocks > (self.n_fits_blocks - self.block_index) {
      return Err(FitsReadErr::EndOfSource {
        blcks_remain: self.n_fits_blocks,
        blcks_req: n_blocks + self.block_index,
      });
    }

    //(4) Read the data
    self.reader_handle.read_exact(buffer)?;

    //(5) Update the block index
    self.block_index += n_blocks;

    //(R) return the number of blocks read
    Ok(n_blocks)
  }

  fn source_len_bytes(&self) -> usize {
    self.file_meta.len() as usize
  }
}

#[derive(Debug)]
pub struct FitsFileWriter {
  pub file_meta: std::fs::Metadata,
  block_index: usize,
  writer_handle: File,
}

impl FitsFileWriter {
  /// Creates a new file to write FITS data to at the provided path. If a file
  /// with the same name already exists, it will be overwritten. If the provided
  /// path does not exist, this function will error.
  ///
  /// # Returns
  /// Returns `FitsWriter` instance if the path is valid, or a `FitsWriteErr`
  ///
  /// # Panics
  /// Does not panic
  pub fn new(path: &Path) -> Result<Self, FitsWriteErr> {
    //(1) Open the file if it exists, create it if it doesn't
    let writer_handle = File::create(path)?;

    //(2) Create the required derivatives
    let file_meta = writer_handle.metadata()?;

    //(R)
    Ok(FitsFileWriter { file_meta, block_index: 0, writer_handle })
  }
}

impl FitsWriter for FitsFileWriter {
  /// Writes data from buffer into FITS file. Returns an error if buffer size is
  /// not a multiple of FITS block size.
  ///
  /// # Returns
  /// Returns number of FITS blocks written to disk
  ///
  /// # Panics
  /// Does not panic
  fn write_blocks_from(&mut self, buffer: &[u8]) -> Result<usize, FitsWriteErr> {
    //(1) Check if the buffer is an integer multiple of BLOCK_LEN
    if buffer.len() % BLOCK_SIZE != 0 {
      return Err(FitsWriteErr::SourceSize(buffer.len()));
    }

    //(2) Calculate size of buffer in FITS blocks
    let blocks_written = buffer.len() / BLOCK_SIZE;

    //(3) Write data
    self.writer_handle.write_all(&buffer)?;

    //(4) Update block_index
    self.block_index += blocks_written;

    //(R) Number of blocks written
    Ok(blocks_written)
  }

  fn flush(&mut self) -> std::io::Result<()> {
    self.writer_handle.flush()
  }
}
