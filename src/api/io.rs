/*
  Copyright© 2023 Raúl Wolters(1)

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

use crate::err::io_err::*;

//Get block size from root
const BLOCK_SIZE: usize = crate::BLOCK_SIZE;

pub trait FitsReader {
  /// Creates and fills a buffer with length `n_blocks*BLOCK_SIZE`.
  ///
  /// # Returns
  /// Returns vec filled with data from fits file, or a `FitsReadErr`
  fn read_blocks(&mut self, n_blocks: usize) -> Result<Vec<u8>, FitsReadErr> {
    //(1) Create buffer
    let mut buffer = vec![0u8; BLOCK_SIZE * n_blocks];

    //(2) Read n_blocks into the buffer
    self.read_blocks_into(&mut buffer)?;

    //(R) return the buffer
    Ok(buffer)
  }

  /// Fills the provided buffer with data from the underlying FITS file. FITS
  /// files may only be read in multiples of 2880 bytes, so this function will
  /// return an error if the provided buffer length is not a multiple of 2880.
  ///
  /// # Returns
  /// Returns number of FITS blocks that were read, or a `FitsReadErr`
  fn read_blocks_into(&mut self, buffer: &mut [u8]) -> Result<usize, FitsReadErr>;
}

pub trait FitsWriter {
  /// Writes data from buffer into FITS file. Returns an error if buffer size is
  /// not a multiple of FITS block size.
  ///
  /// # Returns
  /// Returns number of FITS blocks written to disk
  fn write_blocks_from(&mut self, buffer: &[u8]) -> Result<usize, FitsWriteErr>;

  fn flush(&mut self) -> std::io::Result<()>;

  /// Writes data from buffer into FITS file. If buffer length is not a multiple
  /// of FITS block size, it will be padded with zeroes.
  ///
  /// # Returns
  /// Returns number of FITS blocks written to disk
  fn write_blocks_zeroed(&mut self, buffer: &[u8]) -> Result<usize, FitsWriteErr> {
    //(1) Get number of full blocks and spare bytes
    let n_full_blocks = buffer.len() / BLOCK_SIZE;
    let spare_bytes = buffer.len() % BLOCK_SIZE;

    //(2) Write full buffer to disk
    self.write_blocks_from(&buffer)?;

    //(3) Pad with zeroes to ensure that file size is multiple of BLOCK_LEN
    let padding = vec![0x00u8; BLOCK_SIZE - spare_bytes];
    self.write_blocks_from(&padding)?;

    //(4) Flush to ensure that buffer and padding are in right order
    self.flush()?;

    //(R) return number of blocks written (= n_full_blocks + 1)
    Ok(n_full_blocks + 1)
  }
}