/*
    Copyright (C) 2022 Raúl Wolters

    This file is part of rustronomy-fits.

    rustronomy is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    rustronomy is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with rustronomy.  If not, see <http://www.gnu.org/licenses/>.
*/

//Get block size from root
const BLOCK_SIZE: usize = crate::BLOCK_SIZE; // = 2880B

//IO consts
const MAX_BLOCKS_IN_BUF: usize = 128; // = 369kB
const MIN_BLOCKS_IN_BUF: usize = 1; // = 3kB

use std::{
  error::Error,
  fmt::{Debug, Display},
  mem::size_of,
};

use anyhow::Result;

//Rustronomy Imports
use rustronomy_core::data_type_traits::io_utils::{Decode, Encode};

//External Imports
use ndarray::{Array, ShapeBuilder};
use num_traits::Num;
use rayon::prelude::*;

use crate::{
  bitpix::Bitpix,
  img_err::InvalidMemLayout as IMLErr,
  raw::raw_io::{RawFitsReader, RawFitsWriter},
  Extension,
};

use super::{generic_image::Image, typed_image::TypedImage};

/*
    THIS IS NOT PART OF THE USER-FACING API
    ImgParser (and its implementation) are a helper struct used to decode and
    encode FITS arrays to ndarray arrays. These tasks are performed during reads
    and writes of whole FITS files.
*/
pub(crate) struct ImgParser {}
impl ImgParser {
  //Public decoder for parsing images
  pub(crate) fn decode_img(
    reader: &mut RawFitsReader,
    shape: &Vec<usize>,
    bitpix: Bitpix,
  ) -> Result<Extension> {
    use Bitpix::*;
    use TypedImage::*;

    Ok(Extension::Image(match bitpix {
      Byte => ByteImg(Self::decode_helper::<u8>(reader, shape)?),
      Short => I16Img(Self::decode_helper::<i16>(reader, shape)?),
      Int => I32Img(Self::decode_helper::<i32>(reader, shape)?),
      Long => I64Img(Self::decode_helper::<i64>(reader, shape)?),
      Spf => SpfImg(Self::decode_helper::<f32>(reader, shape)?),
      Dpf => DpfImg(Self::decode_helper::<f64>(reader, shape)?),
    }))
  }

  fn decode_helper<T>(
    reader: &mut RawFitsReader,
    shape: &Vec<usize>,
  ) -> Result<Image<T>>
  where
    T: Debug + Num + Sized + Decode + Encode + Display + Clone + Send,
  {
    /*  (1)
        To create a ndarray we need to provide an underlying data structure.
        For now we'll use the easiest one to implement: a giant 1D vector
        (this happens to also be somewhat efficient). First, we must find
        the number of entries in this vector by folding the array shape.
    */
    let entry_size = size_of::<T>();
    let n_entries = (&shape).iter().fold(1, |prod, &x| prod * x);
    let byte_size = n_entries * entry_size;
    let total_blocks = (byte_size as f64 / BLOCK_SIZE as f64).ceil() as usize;

    /*  Notes:
        FITS supports integers and floats as data types. These are either 1,
        2, 4 or 8 bytes long. Hence BLOCK_SIZE % entry_size == 0 for all data
        types recognized by the FITS standard (we do not have to deal with
        data types spanning multiple FITS blocks).
    */

    /*  (2)
        For performance reasons, we want to read in chunks of at least one
        FITS block, which is 2880 bytes ~3kB. It takes about as many CPU
        cycles to copy 4kB as it does to make a syscall. We do not want to
        make the buffer larger than the L3 cache of the CPU though, so we
        must limit ourselves to below ~1MB. Hence we must determine an
        optimal size for the buffer to pass to the read_blocks function of
        the RawFitsReader.
    */

    //Get the buffer size and the number of times we have to fill the buffer
    let (buf_size, n_reads) = Self::calc_buf_size(total_blocks);

    //Create the vector underpinning the ndarray and the reusable buffer
    let mut flat: Vec<T> = Vec::new();
    let mut buf = vec![0u8; buf_size];

    for _ in 0..n_reads {
      //fill the buffer
      reader.read_blocks(&mut buf)?;

      /*
          Next we'll use rayon to chop the buffer into entry_size sized
          pieces, each of which may then be converted into the type T.
      */
      let mut typed_buf: Vec<T> =
        buf.par_chunks(entry_size).map(|val| T::from_bytes(val)).collect();

      //Add the values to our buffer
      flat.append(&mut typed_buf);
    }

    /*  (3)
        So far we have read an integer multiple of BLOCK_SIZE in bytes.
        Although we are guaranteed to have captured all the data necessary,
        we probably read too many values because the last FITS block may be
        partially empty. Hence we need to pop the difference of the vector.
    */
    flat.truncate(n_entries);

    /*  (4)
        The next step is to convert the flat vector into an array of an
        appropriate size. Note that the FITS specification states that Images
        are represented in the Fortran (column-major) memory-layout, not
        row-major like C and Rust. Hence we have to call the .f() function
        on the shape of the array to tell ndarray that we have a Fortran array.
    */
    let img_data = Array::from_shape_vec(shape.clone().f(), flat)?;

    // (R) return an Image struct
    Ok(Image::<T>::new_sized(shape.clone(), img_data, total_blocks))
  }

  //Encoder for parsing Images. Consumes the image it encodes
  pub(crate) fn encode_img(
    typed_img: TypedImage,
    writer: &mut RawFitsWriter,
  ) -> Result<()> {
    //This function only matches the typed image and calls the appropriate
    //helper function
    use TypedImage::*;

    match typed_img {
      ByteImg(img) => Self::encode_helper(img, writer)?,
      I16Img(img) => Self::encode_helper(img, writer)?,
      I32Img(img) => Self::encode_helper(img, writer)?,
      I64Img(img) => Self::encode_helper(img, writer)?,
      SpfImg(img) => Self::encode_helper(img, writer)?,
      DpfImg(img) => Self::encode_helper(img, writer)?,
    }

    //(R) this went ok
    Ok(())
  }

  fn encode_helper<T>(img: Image<T>, writer: &mut RawFitsWriter) -> Result<()>
  where
    T: Debug + Num + Sized + Decode + Encode + Display + Clone,
  {
    /*  (1)
        ndarray preserves the internal memory-layout that was used to create
        the array. This is nice if the array is already in the Fortran layout,
        but sucks if it's in the C layout, in which case we have to convert
        the indices to the Fortran layout.

        Either way, we end up with a flat, 1D raw vector of types used to
        encode the array
    */

    let mut raw = match img.get_data().is_standard_layout() {
      true => {
        /*  (1a)
            Data is continuously represented in memory using the C layout
            which we will have to convert to Fortran copying all the
            elements from the C array into a new Fortran array (slow!)
        */
        let shape = img.get_shape().clone();
        Array::from_shape_vec(shape.f(), img.get_data_owned().t().iter().cloned().collect())?
          .into_raw_vec()
      }
      false => {
        /*  (1b)
            Data is either discontinuous, OR in Fortran layout. In the
            second case we want to perform a no-op copy!
        */
        match img.get_data().as_slice_memory_order() {
          None => {
            //Data is NOT continuous, return error!
            return Err(IMLErr::new().into());
          }
          _ => {} //Data IS continuous, continue!
        }

        //No-op return the underlying data, since it's already in the
        //Fortran memory layout :)
        img.get_data_owned().into_raw_vec()
      }
    };

    //We have to reverse the raw data to be able to easiliy pop elements
    //of the raw vector in the correct order!
    raw.reverse();

    /*  (2)
        Now that we have the raw array, we still have to divide it up into
        manageble chunks, re-using the methods from the read section.
    */
    let total_byte_size = raw.len() * size_of::<T>();

    /*  Note:
        This total block size rounds down the number of blocks required to
        write the entire array if the total byte size is not cleanly divisible
        by the block size. This is because the final (not completely filled)
        block MUST be filled with zeros after the data, so we need a seperate
        buffer for that data block!
    */
    let total_block_size = total_byte_size / BLOCK_SIZE;
    let (buf_size, _) = Self::calc_buf_size(total_block_size);
    let mut buffer = Vec::new();

    while !raw.is_empty() {
      //If the buffer is full we write it and replace it with an empty buf
      if buffer.len() == buf_size {
        writer.write_blocks(&buffer)?;
        buffer.clear();
      }
      match raw.pop() {
        Some(val) => val.fill_buf(&mut buffer),
        None => {} //loop will break after this
      }
    }

    //If the byte size of the file was not cleanly divided by the block size,
    //the buffer will not be empty after this loop. We have to fill it with
    //zeroes untill it is a multiple of the FITS block size
    if !buffer.is_empty() {
      while buffer.len() % BLOCK_SIZE != 0 {
        buffer.push(0);
      }
      writer.write_blocks(&buffer)?;
    }

    //(R) we sucessfully wrote the Image to the file!
    Ok(())
  }

  fn calc_buf_size(total_blocks: usize) -> (usize, usize) {
    //Return tuple: (buffer size in bytes, #syscalls=reads/writes)

    /* Notes:
        As per the FITS standard, we may only read a FITS block of bytes per
        read. We want the largest integer multiple of the size of a FITS block
        below a maximum buffer size (around ~370kB).

        I decided to tune the maximum buffer size with the fits_bench.rs
        test (reading a bunch of 7MB files).

        These were the results (actual block size listed for clarity)
        [Block size]    [Byte size]     [Read time]
            1           2880B           135ms
            25          72kB            130ms
            107         308kB           137ms
            535         1.54MB          131ms
            2675        7.70MB          134ms

        Conclusion: This limit does not matter **at all*
        Reason: the buffer is heap-allocated, which means that cache optimi-
        zations don't work.
        RFC #1909 will implement this feature in the future, so I'll leave
        the code intact for now...
    */

    let mut n_buf_blocks = 1;

    for i in MIN_BLOCKS_IN_BUF..=MAX_BLOCKS_IN_BUF {
      //If the buffer is the same size as the image, we don't need a bigger buf
      if n_buf_blocks == total_blocks {
        break;
      }

      //If a larger buffer works, use it!
      if total_blocks % i == 0 {
        n_buf_blocks = i;
      }
    }

    let n_accesses = total_blocks / n_buf_blocks;

    //println!("Buffer size: {n_buf_blocks}");

    (n_buf_blocks * BLOCK_SIZE, n_accesses)
  }
}
