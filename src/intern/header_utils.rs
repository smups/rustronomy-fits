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

use super::FitsOptions;

//Mandatory keywords
pub const SIMPLE: &str = "SIMPLE";
pub const BITPIX: &str = "BITPIX";
pub const NAXIS: &str = "NAXIS";
pub const END: &str = "END";

pub const CONTINUE: &str = "CONTINUE";

pub fn decode_records(fits_block: &[u8]) -> Vec<(&str, Option<&str>)> {
  //(1) assert that fits_block is actually a FITS block
  if fits_block.len() != crate::BLOCK_SIZE {
    panic!("irregularly sized FITS block");
  }

  //(2) records are 80 bytes long, so we iterate over the buffer in 80-byte chunks
  let mut recs = Vec::<(&str, Option<&str>)>::with_capacity(6);

  for x in fits_block.chunks_exact(crate::RECORD_SIZE) {
    //Key is in the first 8 bytes (trim spaces!)
    let key: &str = std::str::from_utf8(&x[0..8]).unwrap().trim();
    //Value is present if bytes 9 and 10 are '= '. If so, bytes 11 to 80 contain
    //the value
    let value = if &x[8..10] == "= ".as_bytes() {
      Some(std::str::from_utf8(&x[10..80]).unwrap().trim())
    } else {
      None
    };

    //add the key-value pair to the vec
    recs.push((key, value));

    if key == END {
      //we have reached the end of the header
      break;
    }
  }

  //(R) the records
  recs
}

pub fn parse_records(
  records: &Vec<(&str, Option<&str>)>,
  meta: &mut Vec<(String, String)>,
  options: &mut FitsOptions,
) -> Result<(), Box<dyn std::error::Error>> {
  for (key, value) in records {
    /*
    * (1) Check if we have a FITS option
    */

    //(a) NAXIS{n}
    if key.contains(NAXIS) {
      let n = std::str::from_utf8(&key.as_bytes()[NAXIS.len()..key.len()]).unwrap();
      if n == "" {
        options.dim = value.unwrap().parse()?;
        options.shape = vec![0; options.dim as usize];
      } else {
        let n: usize = n.parse()?;
        options.shape[n] = value.unwrap().parse()?;
      }
      continue;
    }
    //(b) simple
    if *key == SIMPLE {
      options.conforming = value.unwrap().parse()?;
      continue;
    }
    //(c) bitpix
    if *key == BITPIX {
      options.bitpix = value.unwrap().parse()?;
      continue;
    }

    /*
    * (2) Deal with CONTINUE keywords
    */
    if *key == CONTINUE {
      let last_idx = meta.len();
      meta.get_mut(last_idx).unwrap().1.extend(unsafe {
        let bytes = value.unwrap().as_bytes();
        std::str::from_utf8_unchecked(&bytes[0..bytes.len()]).chars()
      });
      continue;
    }

    /*
    * (3) In all other cases we just add the key-value pair to the vec (unless
    * the value is None, in which case we ignore the key)
    */
    if let Some(value) = value {
      meta.push((key.to_string(), value.to_string()))
    };
  }
  Ok(())
}
