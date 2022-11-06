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
  io::{self, Read, Write},
  path::Path,
};

use crate::err::io_err::*;

//Get block size from root
const BLOCK_SIZE: usize = crate::BLOCK_SIZE;

#[derive(Debug)]
pub struct FitsReader {
  pub file_meta: fs::Metadata,
  block_index: usize,
  n_fits_blocks: usize,
  reader_handle: File,
}

impl FitsReader {
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
      return Err(FitsReadErr::FileSize(file_size));
    }
    let n_fits_blocks = file_size / BLOCK_SIZE;

    //Return file as raw FITS
    Ok(FitsReader { file_meta, block_index: 0, n_fits_blocks, reader_handle })
  }

  /// Fills the provided buffer with data from the underlying FITS file. FITS
  /// files may only be read in multiples of 2880 bytes, so this function will
  /// return an error if the provided buffer length is not a multiple of 2880.
  ///
  /// # Returns
  /// Returns number of FITS blocks that were read, or a `FitsReadErr`
  ///
  /// # Panics
  /// Does not panic
  pub fn read_blocks_into(&mut self, buffer: &mut [u8]) -> Result<usize, FitsReadErr> {
    //(1) Calculate how many header blocks we have to read
    let n_blocks = buffer.len() / BLOCK_SIZE;

    //(2) Check if the buffer is an integer multiple of a FITS block
    if n_blocks * BLOCK_SIZE != buffer.len() {
      return Err(FitsReadErr::BufferSize(buffer.len()));
    }

    //(3) Check if the number of header blocks we need to read does not exceed
    //the number of header blocks still left in the file
    if n_blocks > (self.n_fits_blocks - self.block_index) {
      return Err(FitsReadErr::EndOfFile {
        file_size: self.n_fits_blocks,
        blocks_read: n_blocks + self.block_index,
      });
    }

    //(4) Read the data
    self.reader_handle.read_exact(buffer)?;

    //(5) Update the block index
    self.block_index += n_blocks;

    //(R) return the number of blocks read
    Ok(n_blocks)
  }

  /// Creates and fills a buffer with length `n_blocks*BLOCK_SIZE`.
  ///
  /// # Returns
  /// Returns vec filled with data from fits file, or a `FitsReadErr`
  ///  
  /// # Panics
  /// Does not panic
  pub fn read_blocks(&mut self, n_blocks: usize) -> Result<Vec<u8>, FitsReadErr> {
    //(1) Create buffer
    let mut buf = Vec::<u8>::with_capacity(n_blocks * crate::BLOCK_SIZE);

    //(2) Read to buffer
    self.read_blocks_into(&mut buf)?;

    //(R) return the filled buffer
    Ok(buf)
  }
}

#[derive(Debug)]
pub struct FitsWriter {
  pub file_meta: std::fs::Metadata,
  block_index: usize,
  writer_handle: File,
}

impl FitsWriter {
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
    Ok(FitsWriter { file_meta, block_index: 0, writer_handle })
  }

  /// Writes data from buffer into FITS file. Returns an error if buffer size is
  /// not a multiple of FITS block size.
  ///
  /// # Returns
  /// Returns number of FITS blocks written to disk
  ///
  /// # Panics
  /// Does not panic
  pub fn write_blocks_from(&mut self, buffer: &[u8]) -> Result<usize, FitsWriteErr> {
    //(1) Check if the buffer is an integer multiple of BLOCK_LEN
    if buffer.len() % BLOCK_SIZE != 0 {
      return Err(FitsWriteErr::BufferSize(buffer.len()));
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

  /// Writes data from buffer into FITS file. If buffer length is not a multiple
  /// of FITS block size, it will be padded with zeroes.
  ///
  /// # Returns
  /// Returns number of FITS blocks written to disk
  ///
  /// # Panics
  /// Does not panic
  pub fn write_blocks_zeroed(&mut self, buffer: &[u8]) -> Result<usize, FitsWriteErr> {
    //(1) Get number of full blocks and spare bytes
    let n_full_blocks = buffer.len() / BLOCK_SIZE;
    let spare_bytes = buffer.len() % BLOCK_SIZE;

    //(2) Write full buffer to disk
    self.writer_handle.write_all(&buffer)?;

    //(3) Pad with zeroes to ensure that file size is multiple of BLOCK_LEN
    let padding = vec![0x00u8; BLOCK_SIZE - spare_bytes];
    self.writer_handle.write_all(&padding)?;

    //(4) Flush to ensure that buffer and padding are in right order
    self.flush()?;

    //(R) return number of blocks written (= n_full_blocks + 1)
    Ok(n_full_blocks + 1)
  }

  pub(crate) fn flush(&mut self) -> io::Result<()> {
    Ok(self.writer_handle.flush()?)
  }
}
