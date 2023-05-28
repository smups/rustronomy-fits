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

use std::{thread, sync::{Mutex, mpsc::sync_channel, Arc}, ops::DerefMut};

use crate::{intern::{HduOptions, fits_consts::BLOCK_SIZE}, io::FitsReader, hdu::HduData};

pub fn read_image_hdu(opts: &HduOptions, reader: &mut (impl FitsReader + Send)) -> HduData {
  //(1) Caculate the size in bytes of the image
  let byte_size = opts
    .shape()
    .into_iter()
    .fold(opts.bitpix().abs() as usize / 8, |acc, bpx| acc + (*bpx as usize));

  //(2) Calculate the number of *full* FITS blocks we have to read
  let full_block_size = byte_size / BLOCK_SIZE;

  //Two buffers: buf1 is owned by the main thread, buf2 by the io thread
  let mut buf1 = Box::new([0u8; BLOCK_SIZE]);
  let mut mtx_buf = Arc::new(Mutex::new(buf1.clone()));
  //Channel to tell the io thread if it should continue or not
  let (tx, rx) = sync_channel::<bool>(0);

  //(3) Spawn a thread that will be doing the actual *reading* of the fits file
  thread::scope(|scope| {
    let mut mtx_buf_wrkr = mtx_buf.clone();
    //This is the worker thread
    scope.spawn(move || {
      loop {
        //Check if we have to continue
        if !rx.recv().unwrap() {
          break Ok(());
        }
        //Read into buffer
        if let Err(err) = reader.read_blocks_into(mtx_buf_wrkr.lock().unwrap().as_mut()) {
          break Err(err);
        }
      }
    });

    //This is the parser thread
    for _ in 0..full_block_size {
      //Tell the io thread to read a new block
      tx.send(true).unwrap();
      //Swap the buffers
      std::mem::swap(mtx_buf.lock().unwrap().deref_mut(), &mut buf1);
    }
    //Kill the worker
    tx.send(false).unwrap();
  });

  todo!()
}