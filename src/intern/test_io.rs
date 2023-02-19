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

#[cfg(test)]
/// Test FITS files, courtesy of NASA
pub mod mock_data {
  pub const ASTRO_UIT_BYTES: &'static [u8; 538560] = include_bytes!("../../resources/Astro_UIT.fits");
  pub const EUVE_BYTES: &'static [u8; 4291200] = include_bytes!("../../resources/EUVE.fits");
  pub const IUE_LWP_BYTES: &'static [u8; 48960] = include_bytes!("../../resources/IUE_LWP.fits");
  pub const RANDOM_GROUPS_BYTES: &'static [u8; 596160] = include_bytes!("../../resources/RandomGroups.fits");

  pub const HUBBLE_FGS_BYTES: &'static [u8; 2540160] = include_bytes!("../../resources/Hubble_FGS.fits");
  pub const HUBBLE_FOC_BYTES: &'static [u8; 4219200] = include_bytes!("../../resources/Hubble_FOC.fits");
  pub const HUBBLE_FOS_BYTES: &'static [u8; 43200] = include_bytes!("../../resources/Hubble_FOS.fits");
  pub const HUBBLE_HRS_BYTES: &'static [u8; 69120] = include_bytes!("../../resources/Hubble_HRS.fits");
  pub const HUBBLE_NICMOS_BYTES: &'static [u8; 1198080] = include_bytes!("../../resources/Hubble_NICMOS.fits");
  pub const HUBBLE_WFPC2_1_BYTES: &'static [u8; 699840] = include_bytes!("../../resources/Hubble_WFPC2_1.fits");
  pub const HUBBLE_WFPC2_2_BYTES: &'static [u8; 63360] = include_bytes!("../../resources/Hubble_WFPC2_2.fits");

  type TestIo = super::TestIo<'static>;

  pub static ASTRO_UIT: TestIo = TestIo::new_const(ASTRO_UIT_BYTES);
  pub static EUVE: TestIo = TestIo::new_const(EUVE_BYTES);
  pub static IUE_LWP: TestIo = TestIo::new_const(IUE_LWP_BYTES);
  pub static RANDOM_GROUPS: TestIo = TestIo::new_const(RANDOM_GROUPS_BYTES);

  pub static HUBBLE_FGS: TestIo = TestIo::new_const(HUBBLE_FGS_BYTES);
  pub static HUBBLE_FOC: TestIo = TestIo::new_const(HUBBLE_FOC_BYTES);
  pub static HUBBLE_FOS: TestIo = TestIo::new_const(HUBBLE_FOS_BYTES);
  pub static HUBBLE_HRS: TestIo = TestIo::new_const(HUBBLE_HRS_BYTES);
  pub static HUBBLE_NICMOS: TestIo = TestIo::new_const(HUBBLE_NICMOS_BYTES);
  pub static HUBBLE_WFPC2_1: TestIo = TestIo::new_const(HUBBLE_WFPC2_1_BYTES);
  pub static HUBBLE_WFPC2_2: TestIo = TestIo::new_const(HUBBLE_WFPC2_2_BYTES);
}