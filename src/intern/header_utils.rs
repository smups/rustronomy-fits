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

use rustronomy_core::universal_containers::MetaDataContainer;

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
      //we have reached the end of the header
      return recs;
    }
  }

  //(R) we've parsed this whole block and not found an END kw -> this is a bug
  assert!(recs.last().unwrap().0 == END, "couldn't find END kw in FITS header -- THIS IS PROBABLY A BUG --");
  return recs;
}

/// This function turns raw &str key-value-comment keywords into owned String
/// records containing only key-value pairs. In addition, normal
pub fn concat_records(
  records: &[(&str, Option<&str>, Option<&str>)],
  options: &mut FitsOptions,
) -> Result<(Vec<(String, String)>, String, String), Box<dyn std::error::Error>> {
  //Make vec of unparsed keyword-value pairs; keep commentary and history seperate
  let mut meta: Vec<(String, String)> = Vec::new();

  let mut extended_string: Option<(String, String)> = None;
  let mut commentary = String::new();
  let mut history = String::new();

  for (key, value, _comment) in records {
    /*
    * (1) Deal with CONTINUE keywords
    */
    println!("{key}::{value:?}");

    if *key == CONTINUE {
      /* -- Things to take into account when parsing CONTINUE keywords --A
        The last two characters of the current extended string value MUST be
        _&'_ and the first character of the new extension MUST be _'_. We won't
        check this, since it doesn't really break the header anyway. Instead, I'll
        just remove the last two characters from the extended string and append
        all characters from the new extension except the first.

        Sometimes extended string values are used to store a long comment and no
        actual string data at all. They should still follow the same formatting
        rules, so an empty comment string might look like '&'/comment. (no special
        handling is required in this case)

        CONTINUE keywords are only valid after other CONTINUE keywords, or after
        a string-valued keyword. Orphaned CONTINUE keywords should be interpreted
        as COMMENT keywords as per the FITS standard.
      */
      if let Some((_, ref mut current_string)) = extended_string {
        current_string.pop(); //pop the ' character
        current_string.pop(); //pop the & character
        let new_ext = &value.unwrap()[1..]; //everything except the leading '
        current_string.push_str(new_ext);
        continue;
      } else {
        //Interpret this CONTINUE kw as commentary
        commentary.push_str(value.unwrap_or(""));
        continue;
      }
    } else if let Some(current_string) = std::mem::take(&mut extended_string) {
      //If the last keyword was a CONTINUE keyword (extended_string != None), we
      //should push its completed value to the record list since we have now
      //encountered a non-CONTINUE keyword. We should also reset the value of
      //extended_string to None.
      meta.push(current_string);
    }

    /*
    * (2) Parse the FITS-options
    */

    //(a) NAXIS{n}
    if key.contains(NAXIS) {
      let n = std::str::from_utf8(&key.as_bytes()[NAXIS.len()..key.len()]).unwrap();
      if n == "" {
        options.dim = value.unwrap().parse().unwrap();
        options.shape = vec![0; options.dim as usize];
      } else {
        //index in FITS starts with 1, rust starts with 0 so minus one to convert
        let n: usize = n.parse().unwrap();
        options.shape[n - 1] = value.unwrap().parse().unwrap();
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
    * (3) Deal with commentary keywords
    */
    if *key == COMMENT {
      commentary.push_str(value.unwrap_or(""));
      continue;
    }
    if *key == HISTORY {
      history.push_str(value.unwrap_or(""));
      continue;
    }

    /*
    * (4) At this point, we're just working with a normal keyword. If it's an
    * extended string keyword, we should set extended_keyword. If not, we simply
    * push it to the meta list. We should also take care to ignore value-less
    * keywords.
    */
    if let Some(value) = value {
      if value.ends_with("&'") {
        //(4a) This is an extended string kw
        extended_string = Some((key.to_string(), value.to_string()));
      } else {
        //(4b) This is not an extended string kw -> push it
        meta.push((key.to_string(), value.to_string()))
      }
    };
  }
  //(R) the meta vec
  Ok((meta, commentary, history))
}

////////////////////////////////////////////////////////////////////////////////
//                                 UNIT TESTS                                 //
////////////////////////////////////////////////////////////////////////////////

#[test]
fn record_split_test() {
  const TEST_BLOCK: &str = "SIMPLE  =                    T  / FLIGHT22 05Apr96 RSH                          BITPIX  =                   16  / SIGNED 16-BIT INTEGERS                        NAXIS   =                    2  / 2-DIMENSIONAL IMAGES                          NAXIS1  =                  512  / SAMPLES PER LINE                              NAXIS2  =                  512  / LINES PER IMAGE                               EXTEND  =                    T  / FILE MAY HAVE EXTENSIONS                      DATATYPE= 'INTEGER*2'           / SAME INFORMATION AS BITPIX                    TELESCOP= 'UIT     '            / TELECOPE USED                                 INSTRUME= 'INTENSIFIED-FILM'    / DETECTOR USED                                 OBJECT  = 'NGC4151 '            / TARGET NAME                                   OBJECT2 = '_       '            / ALTERNATIVE TARGET NAME                       CATEGORY= 'FLIGHT  '            / TARGET CATEGORY                               JOTFID  = '8116-14 '            / ASTRO MISSION TARGET ID                       IMAGE   = 'FUV2582 '            / IMAGE NUMBER                                  ORIGIN  = 'UIT/GSFC'            / WHERE TAPE WRITTEN                            ASTRO   =                    2  / ASTRO MISSION NUMBER                          FRAMENO = 'b0582   '            / ANNOTATED FRAME NUMBER                        CATHODE = 'CSI     '            / IMAGE TUBE PHOTOCATHODE                       FILTER  = 'B1      '            / CAMERA/FILTER IDENTIFIER                      PDSDATIM= '06-JUL-1995  07:20'  / MICRODENSITOMETRY DATE & TIME                 PDSID   =                   21  / MICRODENSITOMETER IDENT                       PDSAPERT=                   20  / MICROD. APERTURE, MICRONS                     PDSSTEP =                   10  / MICROD. STEP SIZE, MICRONS                    PIXELSIZ=        8.0000000E+01  / CURRENT PIXEL SIZE, MICRONS                   EQUINOX =        2.0000000E+03  / EQUINOX OF BEST COORDINATES                   NOMRA   =             182.0044  / 1950 I.P.S.  R.A., DEGREES                    NOMDEC  =              39.6839  / 1950 I.P.S.  DEC., DEGREES                    NOMROLL =             323.9500  / I.P.S. ROLL ANGLE                             NOMSCALE=        5.6832500E+01  / NOMINAL PLATE SCL (ARCSEC/MM)                 CALIBCON=          5.00000E-16  / PREFLIGHT LAB CALIB FOR CAMERA                FEXPTIME= '8355    '            / EXPOSURE TIME, APPLICABLE FRM                 DATE-OBS= '13/03/95'            / DATE OF OBSERVATION (GMT)                     TIME-OBS=        6.2728000E+00  / TIME OF OBS (HOURS GMT)                       BSCALE  =        2.0587209E-16  / CALIBRATION CONST                             BUNIT   = 'ERGS/CM**2/S/ANGSTRM'                                                END     =              0.00000  / ADDITIVE CONST FOR CALIB.                     ";
  let recs = split_records(TEST_BLOCK.as_bytes());
  assert!(recs[0] == ("SIMPLE", Some("T"), Some("FLIGHT22 05Apr96 RSH")));
  assert!(recs[1] == ("BITPIX", Some("16"), Some("SIGNED 16-BIT INTEGERS")));
}
#[test]
fn record_concat_test() {
  //Setup dummy data
  const TEST_KEY: &str = "TEST";
  const TEST_RECS: [(&str, Option<&str>, Option<&str>); 8] = [
    (TEST_KEY, Some("'Lorem ipsum dolor sit amet, consectetur adipiscing elit. Aenean viverra rutru&'"), None),
    (CONTINUE, Some("'m ante nec facilisis. Praesent rutrum ipsum a volutpat lacinia. In hac habita&'"), None),
    (CONTINUE, Some("'sse platea dictumst. Nulla et volutpat urna. Phasellus luctus congue est, id &'"), None),
    (CONTINUE, Some("'interdum enim aliquam et. Morbi et ipsum mi. Maecenas pretium a metus sit ame&'"), None),
    (CONTINUE, Some("'t semper. Suspendisse non scelerisque libero. Pellentesque sit amet lectus ul&'"), None),
    (CONTINUE, Some("'lamcorper, ullamcorper velit non, feugiat lacus. Vestibulum pellentesque frin&'"), None),
    (CONTINUE, Some("'gilla ex at scelerisque. Integer vitae tincidunt tortor.'"), Some("done with this")),
    (END, None, None)
  ];
  const TEST_ANSWER: &str = "'Lorem ipsum dolor sit amet, consectetur adipiscing elit. Aenean viverra rutrum ante nec facilisis. Praesent rutrum ipsum a volutpat lacinia. In hac habitasse platea dictumst. Nulla et volutpat urna. Phasellus luctus congue est, id interdum enim aliquam et. Morbi et ipsum mi. Maecenas pretium a metus sit amet semper. Suspendisse non scelerisque libero. Pellentesque sit amet lectus ullamcorper, ullamcorper velit non, feugiat lacus. Vestibulum pellentesque fringilla ex at scelerisque. Integer vitae tincidunt tortor.'";
  let mut dummy_options = FitsOptions::new_invalid();
  let (meta, _comments, _history) = concat_records(&TEST_RECS, &mut dummy_options).unwrap();
  assert!(&meta[0].0 == TEST_KEY);
  assert!(&meta[0].1 == TEST_ANSWER);
}

#[test]
fn orphaned_continue_test() {
  const TEST_COMMENT: &str = "this is a comment";
  const TEST_RECS: [(&str, Option<&str>, Option<&str>); 3] = [
    ("TEST_GARBAGE", Some("value"), Some("comment")),
    (CONTINUE, Some(TEST_COMMENT), None),
    (CONTINUE, None, None)
  ];
  const META_ANSWER: (&str, &str) = ("TEST_GARBAGE", "value");
  let mut input_options = FitsOptions::new_invalid();
  let (meta, comments, _) = concat_records(&TEST_RECS, &mut input_options).unwrap();
  assert!(meta.len() == 1);
  assert!(&meta[0].0 == META_ANSWER.0 && &meta[0].1 == META_ANSWER.1);
  assert!(&comments == TEST_COMMENT);
}

#[test]
fn naxis_option_test() {
  //Setup dummy data
  const TEST_RECS: [(&str, Option<&str>, Option<&str>); 4] = [
    (NAXIS, Some("3"), None),
    ("NAXIS1", Some("1000"), None),
    ("NAXIS2", Some("2250"), None),
    ("NAXIS3", Some("272"), None)
  ];
  const TEST_ANSWER: [u16; 3] = [1000, 2250, 272]; 
  let mut input_options = FitsOptions::new_invalid();
  let _ = concat_records(&TEST_RECS, &mut input_options).unwrap();
  assert!(input_options.dim == input_options.shape.len() as u16);
  assert!(input_options.shape.len() == TEST_ANSWER.len());
  assert!(input_options.shape == TEST_ANSWER);
}

#[test]
fn simple_option_test() {
  //Setup dummy data
  const TEST_RECS: [(&str, Option<&str>, Option<&str>); 1] = [
    (SIMPLE, Some("T"), None)
  ];
  const TEST_ANSWER: bool = true; 
  let mut input_options = FitsOptions::new_invalid();
  let _ = concat_records(&TEST_RECS, &mut input_options).unwrap();
  assert!(input_options.conforming == TEST_ANSWER);
}
