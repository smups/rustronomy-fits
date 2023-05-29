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

use std::{thread, sync::{Mutex, mpsc::sync_channel, Arc}, ops::DerefMut, error::Error};

use num_traits::Num;

use crate::{intern::{HduOptions, fits_consts::BLOCK_SIZE}, io::FitsReader, hdu::HduData, err::io_err::FitsReadErr};

pub fn read_image_hdu(opts: &HduOptions, reader: &mut (impl FitsReader + Send)) -> HduData {
  //(1a) Caculate the size in bytes of the image
  let n_entries = opts.shape().into_iter().fold(0, |acc, bpx| acc + (*bpx as usize));
  let byte_size = (opts.bitpix().abs() as usize / 8) * n_entries;

  //(1b) Calculate the number of *full* FITS blocks we have to read
  let full_block_size = byte_size / BLOCK_SIZE;

  //(2) Allocate a vec (of the appropriate type) large enough to hold all data


  todo!()
}

fn read_typed_vec<T: FitsNumber>(n_entries: usize, reader: &mut (impl FitsReader + Send)) -> Result<Vec<T>, FitsReadErr> {
  //(1) Pre-allocate output vec
  let mut out = Vec::<T>::with_capacity(n_entries);

  //Calculate number of blocks that we have to read
  let n_full_blocks = std::mem::size_of::<T>() * n_entries / BLOCK_SIZE;

  /*
  * Explanation of IO strategy
  * There will be two threads: one responsible for reading header blocks, and 
  * one responsible for turning the raw bytes into typed data.
  * 
  */
  let mut local_buf = Ok(Box::new([0u8; BLOCK_SIZE]));
  let shared_buf = Mutex::new(Ok(Box::new([0u8; BLOCK_SIZE])));
  let (tx, rx) = sync_channel::<bool>(0);

  //() Create a scope to manage the buffer lifetimes
  thread::scope(|scope| -> Result<(), FitsReadErr> {
    let shared_buf_ref = &shared_buf;
    
    //Set-up the IO thread
    scope.spawn(move || {
      let mut io_buf = Ok(Box::new([0u8; BLOCK_SIZE]));
      while let Ok(true) = rx.recv() {
        //Fill local buffer
        io_buf = reader
          .read_blocks_into(io_buf.as_mut().unwrap().deref_mut())
          .and_then(|_| Ok(io_buf.unwrap()));
        //Swap local and shared buffer
        std::mem::swap(shared_buf_ref.lock().unwrap().deref_mut(), &mut io_buf);
      }
    });

    //Instruct the IO thread to prepare the first read
    tx.send(true);

    //We do the parsing on the main thread (order of these ops is very important!)
    for _ in 0..n_full_blocks {
      //(1) Swap the buffers (read the result from the IO thread)
      std::mem::swap(shared_buf_ref.lock().unwrap().deref_mut(), &mut local_buf);

      //(2) Tell the IO thread to continue
      tx.send(true);

      //(3) Parse the buffer
      if let Err(err) = local_buf {
        return Err(err)
      } else if let Ok(ref buf) = local_buf {
        for raw in buf.chunks_exact(std::mem::size_of::<T>()) {
          out.push(T::fits_decode(raw))
        }
      }
    }

    //Read the last FITS block
    tx.send(true);
    let final_block = std::mem::replace(shared_buf.lock().unwrap().deref_mut(), local_buf)?;
    let remainder = (std::mem::size_of::<T>() * n_entries) % BLOCK_SIZE;
    for raw in final_block[0..=remainder].chunks_exact(std::mem::size_of::<T>()) {
      out.push(T::fits_decode(raw))
    }

    //Kill the IO thread
    tx.send(false);
    Ok(())
  })?;

  //(R) Return the finished filled buffer!
  return Ok(out);
}

#[test]
fn test_read_typed_vec() {
  use crate::intern::test_io as io;
  let mut reader = io::mock_data::HUBBLE_FOC.clone_with_delay::<10>();
  let n_entries = 10000;
  let vec: Vec<i64> = read_typed_vec(n_entries, &mut reader).unwrap();
}

trait FitsNumber: Num {
  fn fits_decode(raw: &[u8]) -> Self;
  fn fits_encode(self, dest: &mut [u8]);
}

macro_rules! impl_fitsnumber {
  ($($type:ty),*) => {$(
    impl FitsNumber for $type {
      #[inline]
      fn fits_decode(raw: &[u8]) -> Self {
        Self::from_be_bytes(raw.try_into().expect("incorrect slice length. This is a bug in rustronomy-fits"))
      }
    
      #[inline]
      fn fits_encode(self, dest: &mut [u8]) {
        dest.copy_from_slice(&self.to_be_bytes())
      }
    }
  )*};
}
impl_fitsnumber!(u8, i16, i32, i64, f32, f64);