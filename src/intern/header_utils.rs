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

use crate::err::io_err::FitsReadErr;

use super::{FitsOptions, FitsReader};

//Comment separator
const SEP: char = '/';

//Mandatory keywords
pub const SIMPLE: &str = "SIMPLE";
pub const BITPIX: &str = "BITPIX";
pub const NAXIS: &str = "NAXIS";
pub const END: &str = "END";

//Special keywords
pub const CONTINUE: &str = "CONTINUE";
pub const COMMENT: &str = "COMMENT";
pub const HISTORY: &str = "HISTORY";

//Error messages
const UTF8_KEYERR: &str = "Could not parse FITS keyword record using UTF-8 encoding";
const UTF8_RECERR: &str = "Could not parse FITS record value using UTF-8 encoding";

/// Reads FITS blocks from the reader until encountering the END keyword or until
/// an error occurs. All blocks are appended to a single buffer.
pub fn read_header(reader: &mut FitsReader) -> Result<Vec<u8>, FitsReadErr> {
  //container to collect into
  let mut header_bytes = Vec::with_capacity(crate::BLOCK_SIZE);

  //read FITS blocks until we find the final one
  let header_bytes = loop {
    let mut block = reader.read_blocks(1)?;
    /* This block is the last block if:
        - the last 80 bytes are all spaces
        - the last 80 bytes contain the END keyword
      If neither of these is true, continue reading FITS blocks
    */
    let last_record = &block[crate::BLOCK_SIZE - crate::RECORD_SIZE..crate::BLOCK_SIZE];
    let last_key = std::str::from_utf8(&last_record[0..8]).expect(UTF8_KEYERR).trim();
    if last_key == "" || last_key == END {
      //append the last block and return
      header_bytes.append(&mut block);
      break header_bytes;
    } else {
      //continue looping and reading FITS blocks
      header_bytes.append(&mut block);
    }
  };

  //consistency check before returning: assert that we got a multiple of BLOCK_SIZE
  assert!(header_bytes.len() % crate::BLOCK_SIZE == 0, "irregularly sized FITS block found while reading -- THIS IS A BUG --");
  Ok(header_bytes)
}

/// This function takes a 2880-byte FITS block and splits it into 80-byte keyword
/// records. The records are then split into a keyword, optional value and optional
/// comment.
pub fn split_records<'a>(
  fits_block: &'a[u8]
) -> Vec<(&'a str, Option<&'a str>, Option<&'a str>)> {
  //(1) assert that fits_block is actually a FITS block
  if fits_block.len() % crate::BLOCK_SIZE != 0 {
    panic!("irregularly sized FITS header found in split_records -- THIS IS A BUG --");
  }

  //Make return vec
  let mut recs = Vec::new();

  //(2) records are 80 bytes long, so we iterate over the buffer in 80-byte chunks
  for x in fits_block.chunks_exact(crate::RECORD_SIZE) {
    //Key is in the first 8 bytes (trim spaces!)
    let key: &str = std::str::from_utf8(&x[0..8]).expect(UTF8_KEYERR).trim();

    let (value, comment) = if key == COMMENT || key == HISTORY {
      //(i): The comment and history keywords are special because they do NOT
      //use the normal value syntax and instead only contain text, including in
      //bytes 9 & 10 which usually contain the value indicator
      (None, Some(std::str::from_utf8(&x[8..80]).expect(UTF8_RECERR).trim()))
    } else if &x[8..10] == "= ".as_bytes() {
      //(ii): There is a value associated with this keyword, and possibly a comment.
      //In the second case, The comment follows the '/' character.
      let body: &str = std::str::from_utf8(&x[10..80]).expect(UTF8_KEYERR).trim();
      if body.contains(SEP) {
        //(iia): Same as (ii) but we have a comment!
        let (value, comment) = body.split_once(SEP).expect("FITS-KR contained \'/\' but also not? BUG!");
        (Some(value.trim()), Some(comment.trim()))
      } else {
        //(iib): Same as (ii) but we do NOT have a comment!
        (Some(body), None)
      }
    } else if key == CONTINUE {
      //(iii): Yet another special keyword. Does not contain '= ' in bytes 9 & 10
      // but COULD include both a value AND a comment
      let body: &str = std::str::from_utf8(&x[10..80]).expect(UTF8_KEYERR).trim();
      if body.contains(SEP) {
        //(iiia): String with a comment
        let (value, comment) = body.split_once(SEP).expect("FITS-KR contained \'/\' but also not? BUG!");
        (Some(value.trim()), Some(comment.trim()))
      } else {
        //(iiib): String without a comment
        (Some(body), None)
      }
    } else {
      //(iv): this is an uninitialized or BLANK keyword
      (None, None)
    };

    //add the key-value pair to the vec
    recs.push((key, value, comment));

    if key == END {
      //we have reached the end of the header -> return true
      return recs;
    }
  }

  //(R) we've parsed this whole block and not found an END kw -> this is a bug
  assert!(recs.last().unwrap().0 == END, "couldn't find END kw in FITS header -- THIS IS PROBABLY A BUG --");
  return recs;
}

pub fn parse_records(
  records: &[(&str, Option<&str>, Option<&str>)],
  meta: &mut Vec<(String, String)>,
  options: &mut FitsOptions,
) -> Result<(), Box<dyn std::error::Error>> {
  for (key, value, comment) in records {
    /*
     * (1) Check if we have a FITS option
    */

    //(a) NAXIS{n}
    if key.contains(NAXIS) {
      let n = std::str::from_utf8(&key.as_bytes()[NAXIS.len()..key.len()]).unwrap();
      if n == "" {
        options.dim = value.unwrap().parse().unwrap();
        options.shape = vec![0; options.dim as usize];
      } else {
        let n: usize = n.parse().unwrap();
        options.shape[n] = value.unwrap().parse().unwrap();
      }
      continue;
    }
    //(b) simple
    if *key == SIMPLE {
      options.conforming = value.unwrap().parse().unwrap();
      continue;
    }
    //(c) bitpix
    if *key == BITPIX {
      options.bitpix = value.unwrap().parse().unwrap();
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

////////////////////////////////////////////////////////////////////////////////
//                                 UNIT TESTS                                 //
////////////////////////////////////////////////////////////////////////////////

#[test]
fn record_decode_test() {
  const TEST_BLOCK: &str = "SIMPLE  =                    T  / FLIGHT22 05Apr96 RSH                          BITPIX  =                   16  / SIGNED 16-BIT INTEGERS                        NAXIS   =                    2  / 2-DIMENSIONAL IMAGES                          NAXIS1  =                  512  / SAMPLES PER LINE                              NAXIS2  =                  512  / LINES PER IMAGE                               EXTEND  =                    T  / FILE MAY HAVE EXTENSIONS                      DATATYPE= 'INTEGER*2'           / SAME INFORMATION AS BITPIX                    TELESCOP= 'UIT     '            / TELECOPE USED                                 INSTRUME= 'INTENSIFIED-FILM'    / DETECTOR USED                                 OBJECT  = 'NGC4151 '            / TARGET NAME                                   OBJECT2 = '_       '            / ALTERNATIVE TARGET NAME                       CATEGORY= 'FLIGHT  '            / TARGET CATEGORY                               JOTFID  = '8116-14 '            / ASTRO MISSION TARGET ID                       IMAGE   = 'FUV2582 '            / IMAGE NUMBER                                  ORIGIN  = 'UIT/GSFC'            / WHERE TAPE WRITTEN                            ASTRO   =                    2  / ASTRO MISSION NUMBER                          FRAMENO = 'b0582   '            / ANNOTATED FRAME NUMBER                        CATHODE = 'CSI     '            / IMAGE TUBE PHOTOCATHODE                       FILTER  = 'B1      '            / CAMERA/FILTER IDENTIFIER                      PDSDATIM= '06-JUL-1995  07:20'  / MICRODENSITOMETRY DATE & TIME                 PDSID   =                   21  / MICRODENSITOMETER IDENT                       PDSAPERT=                   20  / MICROD. APERTURE, MICRONS                     PDSSTEP =                   10  / MICROD. STEP SIZE, MICRONS                    PIXELSIZ=        8.0000000E+01  / CURRENT PIXEL SIZE, MICRONS                   EQUINOX =        2.0000000E+03  / EQUINOX OF BEST COORDINATES                   NOMRA   =             182.0044  / 1950 I.P.S.  R.A., DEGREES                    NOMDEC  =              39.6839  / 1950 I.P.S.  DEC., DEGREES                    NOMROLL =             323.9500  / I.P.S. ROLL ANGLE                             NOMSCALE=        5.6832500E+01  / NOMINAL PLATE SCL (ARCSEC/MM)                 CALIBCON=          5.00000E-16  / PREFLIGHT LAB CALIB FOR CAMERA                FEXPTIME= '8355    '            / EXPOSURE TIME, APPLICABLE FRM                 DATE-OBS= '13/03/95'            / DATE OF OBSERVATION (GMT)                     TIME-OBS=        6.2728000E+00  / TIME OF OBS (HOURS GMT)                       BSCALE  =        2.0587209E-16  / CALIBRATION CONST                             BUNIT   = 'ERGS/CM**2/S/ANGSTRM'                                                END     =              0.00000  / ADDITIVE CONST FOR CALIB.                     ";
  let recs = split_records(TEST_BLOCK.as_bytes());
  assert!(recs[0] == ("SIMPLE", Some("T"), Some("FLIGHT22 05Apr96 RSH")));
  assert!(recs[1] == ("BITPIX", Some("16"), Some("SIGNED 16-BIT INTEGERS")));
}
