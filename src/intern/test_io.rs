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

use crate::{io::FitsReader, err::io_err::FitsReadErr, BLOCK_SIZE};

#[derive(Debug)]
pub struct TestIo<'a> {
  data: &'a [u8],
  cursor: usize
}

impl<'a> TestIo<'a> {
  /// Static const constructor
  pub const fn new_const(data: &'static [u8]) -> TestIo<'static> {
    TestIo { data, cursor: 0 }
  }

  /// Non-static constructor
  pub fn new(data: &'a [u8]) -> Self {
    TestIo { data, cursor: 0 }
  }

  /// Returns a `TestIo<'a>` pointing to the same data as `self`, but with the
  /// cursor reset to zero.
  pub const fn clone(&self) -> Self {
    TestIo { data: self.data, cursor: 0 }
  }
}

impl<'a> FitsReader for TestIo<'a> {
  fn read_blocks_into(&mut self, buffer: &mut [u8]) -> Result<usize, FitsReadErr> {
    //(1) Get the number of bytes we have to read
    let bytes_to_read = buffer.len();

    //(2) Check that the buffer is a multiple of BLOCK_SIZE
    if bytes_to_read % BLOCK_SIZE != 0 {
      Err(FitsReadErr::BufferSize(bytes_to_read))?
    }

    //(3) Check if we have bytes left to yield
    if self.data.len() < bytes_to_read {
      //Not enough bytes in this file (do not modify cursor)
      Err(FitsReadErr::FileSize(self.data.len()))
    } else if self.cursor + bytes_to_read <= self.data.len() {
      //Still bytes left, go ahead and copy them into the buffer (modify cursor)
      buffer.copy_from_slice(&self.data[self.cursor..self.cursor + bytes_to_read]);
      self.cursor += bytes_to_read / BLOCK_SIZE;
      Ok(bytes_to_read / BLOCK_SIZE)
    } else {
      Err(FitsReadErr::EndOfFile{
        file_size: self.data.len() / BLOCK_SIZE,
        blocks_read: (self.cursor + bytes_to_read) / BLOCK_SIZE
      })
    }
  }
}
