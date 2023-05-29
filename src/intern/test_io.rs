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

use std::{thread, time::Duration};

use crate::{intern::fits_consts::BLOCK_SIZE, io::FitsReader};

// Shorthand error type
type Error = crate::err::io_err::FitsReadErr;

#[derive(Debug, PartialEq, Eq)]
pub struct TestIo<'a, const DELAY: u64 = 0> {
  data: &'a [u8],
  cursor: usize,
}

impl<'a> TestIo<'a> {
  /// Non-static constructor
  pub const fn new(data: &'a [u8]) -> Self {
    TestIo { data, cursor: 0 }
  }

  /// Returns a `TestIo<'a>` pointing to the same data as `self`, but with the
  /// cursor reset to zero.
  pub const fn clone(&self) -> Self {
    TestIo { data: self.data, cursor: 0 }
  }

  /// Returns a `TestIo<'a>` pointing to the same data as `self`, but with the
  /// cursor reset to zero and a different input delay.
  pub const fn clone_with_delay<const DELAY: u64>(&self) -> TestIo<'_, DELAY> {
    TestIo { data: self.data, cursor: 0 }
  }
}

#[test]
fn test_testio_new() {
  let data = &[1, 2, 3, 4, 5];
  assert_eq!(TestIo::new(data), TestIo { data, cursor: 0 });
}

#[test]
fn test_testio_clone() {
  let x = TestIo { data: &[1, 2, 3, 4, 5], cursor: 12 };
  let y = x.clone();
  assert_ne!(x, y);
  assert_eq!(x.data, y.data);
}

impl<'a, const DELAY: u64> FitsReader for TestIo<'a, DELAY> {
  fn read_blocks_into(&mut self, buffer: &mut [u8]) -> Result<usize, Error> {
    //(1) Get the number of bytes we have to read
    let bytes_to_read = buffer.len();

    //(2) Check that the buffer is a multiple of BLOCK_SIZE
    if bytes_to_read % BLOCK_SIZE != 0 {
      Err(Error::DestNotBlockSized(bytes_to_read))?
    }

    //(3) Check that the file is large enough to fill the buffer
    let blcks_req = buffer.len() / BLOCK_SIZE;
    let blcks_remain = (self.data.len() / BLOCK_SIZE) - self.cursor;
    if blcks_remain < blcks_req {
      Err(Error::EndOfSource { blcks_remain, blcks_req })?
    }

    //(4) Read the blocks from cursor to cursor + blcks_req into the buffer
    let start = self.cursor * BLOCK_SIZE;
    let stop = (self.cursor + blcks_req) * BLOCK_SIZE;
    buffer.copy_from_slice(&self.data[start..stop]);

    //(5) Update the cursor
    self.cursor += blcks_req;

    //(6) Simulate input delay (if configured to do so)
    thread::sleep(Duration::from_millis(DELAY));

    //(R) the amount of blocks read
    Ok(blcks_req)
  }

  fn source_len_bytes(&self) -> usize {
    self.data.len()
  }
}

#[test]
fn test_testio_fitsreader_dest_not_block_sized() {
  let mut rdr = TestIo::new("hello world!".as_bytes());
  assert!(matches!(rdr.read_blocks_into(&mut [0u8; 12]), Err(Error::DestNotBlockSized(_))))
}

#[test]
fn test_testio_fitsreader_source_st_dest() {
  let mut rdr = TestIo::new(&[0; BLOCK_SIZE]);
  assert!(matches!(rdr.read_blocks_into(&mut [0; 2 * BLOCK_SIZE]), Err(Error::EndOfSource { .. })));
}

#[test]
fn test_testio_fitsreader_read_too_much() {
  let mut rdr = TestIo::new(&[0; BLOCK_SIZE]);
  //This read should work
  rdr.read_blocks_into(&mut [0; BLOCK_SIZE]).unwrap();
  //This one should not
  assert!(matches!(rdr.read_blocks_into(&mut [0; 2 * BLOCK_SIZE]), Err(Error::EndOfSource { .. })));
}

#[test]
fn test_testio_fitsreader_read() {
  let source = &[132; BLOCK_SIZE];
  let mut dest = [0; BLOCK_SIZE];
  let mut rdr = TestIo::new(source);

  rdr.read_blocks_into(&mut dest).unwrap();
  assert_eq!(source, &dest);
  assert_eq!(rdr.cursor, 1);
}

#[cfg(test)]
/// Test FITS files, courtesy of NASA
pub mod mock_data {
  pub const ASTRO_UIT_BYTES: &'static [u8; 538560] =
    include_bytes!("../../resources/Astro_UIT.fits");
  pub const EUVE_BYTES: &'static [u8; 4291200] = include_bytes!("../../resources/EUVE.fits");
  pub const IUE_LWP_BYTES: &'static [u8; 48960] = include_bytes!("../../resources/IUE_LWP.fits");
  pub const RANDOM_GROUPS_BYTES: &'static [u8; 596160] =
    include_bytes!("../../resources/RandomGroups.fits");

  pub const HUBBLE_FGS_BYTES: &'static [u8; 2540160] =
    include_bytes!("../../resources/Hubble_FGS.fits");
  pub const HUBBLE_FOC_BYTES: &'static [u8; 4219200] =
    include_bytes!("../../resources/Hubble_FOC.fits");
  pub const HUBBLE_FOS_BYTES: &'static [u8; 43200] =
    include_bytes!("../../resources/Hubble_FOS.fits");
  pub const HUBBLE_HRS_BYTES: &'static [u8; 69120] =
    include_bytes!("../../resources/Hubble_HRS.fits");
  pub const HUBBLE_NICMOS_BYTES: &'static [u8; 1198080] =
    include_bytes!("../../resources/Hubble_NICMOS.fits");
  pub const HUBBLE_WFPC2_1_BYTES: &'static [u8; 699840] =
    include_bytes!("../../resources/Hubble_WFPC2_1.fits");
  pub const HUBBLE_WFPC2_2_BYTES: &'static [u8; 63360] =
    include_bytes!("../../resources/Hubble_WFPC2_2.fits");

  type TestIo = super::TestIo<'static>;

  pub static ASTRO_UIT: TestIo = TestIo::new(ASTRO_UIT_BYTES);
  pub static EUVE: TestIo = TestIo::new(EUVE_BYTES);
  pub static IUE_LWP: TestIo = TestIo::new(IUE_LWP_BYTES);
  pub static RANDOM_GROUPS: TestIo = TestIo::new(RANDOM_GROUPS_BYTES);

  pub static HUBBLE_FGS: TestIo = TestIo::new(HUBBLE_FGS_BYTES);
  pub static HUBBLE_FOC: TestIo = TestIo::new(HUBBLE_FOC_BYTES);
  pub static HUBBLE_FOS: TestIo = TestIo::new(HUBBLE_FOS_BYTES);
  pub static HUBBLE_HRS: TestIo = TestIo::new(HUBBLE_HRS_BYTES);
  pub static HUBBLE_NICMOS: TestIo = TestIo::new(HUBBLE_NICMOS_BYTES);
  pub static HUBBLE_WFPC2_1: TestIo = TestIo::new(HUBBLE_WFPC2_1_BYTES);
  pub static HUBBLE_WFPC2_2: TestIo = TestIo::new(HUBBLE_WFPC2_2_BYTES);
}
