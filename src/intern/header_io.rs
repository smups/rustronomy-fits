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
use rustronomy_core::{prelude::MetaContainer, universal_containers::MetaOnly};

use crate::{
  api::io::*,
  err::{
    header_err::{HeaderReadErr, InvalidHeaderErr, UTF8_KEYERR, UTF8_RECERR},
    io_err::FitsReadErr,
  },
};

use super::{fits_consts::*, FitsOptions};

/// Reads header of a FITS Header-Data-Unit and stores all encountered tags in
/// the supplied metadata container.
pub fn read_header(
  reader: &mut impl FitsReader,
  meta: &mut impl MetaContainer,
) -> Result<Box<FitsOptions>, HeaderReadErr> {
  //(1) Start with reading all data that is supposed to
  let bytes = read_header_blocks(reader)?;

  //(2) Split the raw bytes into Key-Value-Comment triplets
  let kvc = bytes.chunks_exact(RECORD_SIZE).map(|x| split_keyword_record(x));

  //(3) Concatenate the Key-Value-Comment triplets into coherent data
  // -> store this in a metacontainer
  let options = concat(kvc, meta)?;

  //(R) return metadata and options
  Ok(options)
}

/// Reads FITS blocks from the reader until encountering the END keyword or until
/// an error occurs. All blocks are appended to a single buffer.
fn read_header_blocks(reader: &mut impl FitsReader) -> Result<Vec<u8>, FitsReadErr> {
  //container to collect into
  let mut header_bytes = Vec::with_capacity(BLOCK_SIZE);

  //read FITS blocks until we find the final one
  let header_bytes = loop {
    let mut block = reader.read_blocks(1)?;
    /* This block is the last block if:
        - the last 80 bytes are all spaces
        - the last 80 bytes contain the END keyword
      If neither of these is true, continue reading FITS blocks
    */
    let last_record = &block[BLOCK_SIZE - RECORD_SIZE..BLOCK_SIZE];
    let last_keyword = std::str::from_utf8(&last_record[0..8]).expect(UTF8_KEYERR).trim();
    if last_record == &[b' '; 80] || last_keyword == END {
      //append the last block and return
      header_bytes.append(&mut block);
      break header_bytes;
    } else {
      //continue looping and reading FITS blocks
      header_bytes.append(&mut block);
    }
  };

  //consistency check before returning: assert that we got a multiple of BLOCK_SIZE
  assert!(
    header_bytes.len() % BLOCK_SIZE == 0,
    "irregularly sized FITS block found while reading -- THIS IS A BUG --"
  );
  Ok(header_bytes)
}

#[test]
fn read_single_header_block() {
  use super::test_io::TestIo;
  let mut test_reader = TestIo::new(&[' ' as u8; BLOCK_SIZE]);
  assert_eq!(read_header_blocks(&mut test_reader).unwrap(), &[' ' as u8; BLOCK_SIZE]);
}

#[test]
fn read_multiple_header_blocks() {
  use super::test_io::mock_data;
  let mut test_reader = mock_data::ASTRO_UIT.clone();
  //Header is 4 FITS blocks long
  const HDR_SIZE: usize = 4 * BLOCK_SIZE;
  assert_eq!(
    &read_header_blocks(&mut test_reader).unwrap().len() / BLOCK_SIZE,
    &mock_data::ASTRO_UIT_BYTES[0..HDR_SIZE].len() / BLOCK_SIZE
  )
}

/// This function takes a 80-byte FITS keyword-record and splits it into a
/// keyword, optional value and optional comment.
fn split_keyword_record(chunk: &[u8]) -> (&str, Option<&str>, Option<&str>) {
  //Key is in the first 8 bytes (trim spaces!)
  let key: &str = std::str::from_utf8(&chunk[0..8]).expect(UTF8_KEYERR).trim();
  let (value, comment) = if key == COMMENT || key == HISTORY {
    //(1): The comment and history keywords are special because they do NOT
    //use the normal value syntax and instead only contain text, including in
    //bytes 9 & 10 which usually contain the value indicator
    (None, Some(std::str::from_utf8(&chunk[8..80]).expect(UTF8_RECERR).trim()))
  } else if &chunk[8..10] == "= ".as_bytes() {
    //(2): There is a value associated with this keyword, and possibly a comment.
    //In the second case, The comment follows the '/' character.
    let body: &str = std::str::from_utf8(&chunk[10..80]).expect(UTF8_KEYERR).trim();
    if body.contains(SEP) {
      //(2a): Same as (2) but we have a comment!
      let (value, comment) =
        body.split_once(SEP).expect("FITS-KR contained \'/\' but also not? BUG!");
      (Some(value.trim()), Some(comment.trim()))
    } else {
      //(2b): Same as (2) but we do NOT have a comment!
      (Some(body), None)
    }
  } else if key == CONTINUE {
    //(3): Yet another special keyword. Does not contain '= ' in bytes 9 & 10
    // but COULD include both a value AND a comment
    let body: &str = std::str::from_utf8(&chunk[10..80]).expect(UTF8_KEYERR).trim();
    if body.contains(SEP) {
      //(3a): String with a comment
      let (value, comment) =
        body.split_once(SEP).expect("FITS-KR contained \'/\' but also not? BUG!");
      (Some(value.trim()), Some(comment.trim()))
    } else {
      //(3b): String without a comment
      (Some(body), None)
    }
  } else {
    //(4): this is an uninitialized or BLANK keyword
    (None, None)
  };
  //(R) (key, value, comment)
  return (key, value, comment);
}

fn concat<'a>(
  kvc: impl Iterator<Item = (&'a str, Option<&'a str>, Option<&'a str>)> + 'a,
  metadata: &mut impl MetaContainer,
) -> Result<Box<FitsOptions>, InvalidHeaderErr> {
  //Make vec of unparsed keyword-value pairs; keep commentary and history separate
  let mut options = Box::new(FitsOptions::new_invalid());
  let mut commentary = String::new();
  let mut history = String::new();

  //Field to keep track of extended string keywords
  let mut extended_string: Option<(String, String)> = None;

  for (key, value, _comment) in kvc {
    /*
     * (1) Deal with CONTINUE keywords
     */
    if key == CONTINUE {
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
        let new_ext = value.ok_or(InvalidHeaderErr::NoValue { key: CONTINUE })?;
        current_string.push_str(&new_ext[1..]); //don´t append leading '
      } else {
        //Interpret this CONTINUE kw as commentary
        commentary.push_str(value.unwrap_or(""));
      }
      continue;
    } else if let Some(current_string) = std::mem::take(&mut extended_string) {
      //If the last keyword was a CONTINUE keyword (extended_string != None), we
      //should push its completed value to the record list since we have now
      //encountered a non-CONTINUE keyword. We should also reset the value of
      //extended_string to None (the mem::take fn does this).
      insert_meta_tag(&current_string.0, &current_string.1, metadata)?;
    }

    /*
     * (2) Parse the FITS-options
     */
    if key.starts_with(NAXIS) {
      //(a) NAXIS{n}
      parse_naxis(key, value, &mut options)?;
      continue;
    }
    if key == SIMPLE {
      //(b) SIMPLE
      parse_simple(key, value, &mut options)?;
      continue;
    }
    if key == BITPIX {
      //(c) BITPIX
      parse_bitpix(key, value, &mut options)?;
      continue;
    }

    /*
     * (3) Deal with commentary keywords
     */
    if key == COMMENT {
      commentary.push_str(value.unwrap_or(""));
      continue;
    }
    if key == HISTORY {
      history.push_str(value.unwrap_or(""));
      continue;
    }

    /* (3b) end the keyword parsing once we hit the END kw */
    if key == END {
      break;
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
        metadata.insert_string_tag(key, value);
      }
    };
  }

  //(3) Push the history and commentary kw's
  metadata.insert_string_tag("HISTORY", &history);
  metadata.insert_string_tag("COMMENT", &commentary);

  //(R) the meta vec
  Ok(options)
}

////////////////////////////////////////////////////////////////////////////////
//////////////////////////// CONCAT HELPER FUNCTIONS ///////////////////////////
////////////////////////////////////////////////////////////////////////////////

/// Helper function that parses NAXIS type keywords
fn parse_naxis(
  key: &str,
  value: Option<&str>,
  options: &mut FitsOptions,
) -> Result<(), InvalidHeaderErr> {
  let idx = std::str::from_utf8(&key.as_bytes()[NAXIS.len()..key.len()]).expect(UTF8_KEYERR);
  let value = value.ok_or(InvalidHeaderErr::NoValue { key: NAXIS })?;
  if idx == "" {
    options.dim = value.parse().map_err(|err| InvalidHeaderErr::fmt_err(NAXIS, err))?;
    options.shape = vec![0; options.dim as usize];
  } else {
    let idx: usize = idx.parse().map_err(|err| InvalidHeaderErr::fmt_err(NAXIS, err))?;
    let value = value.parse().map_err(|err| InvalidHeaderErr::fmt_err(NAXIS, err))?;
    //index in FITS starts with 1, rust starts with 0 so minus one to convert
    *options
      .shape
      .get_mut(idx - 1)
      .ok_or(InvalidHeaderErr::NaxisOob { idx, naxes: options.dim })? = value;
  }
  Ok(())
}

fn parse_simple(
  key: &str,
  value: Option<&str>,
  options: &mut FitsOptions,
) -> Result<(), InvalidHeaderErr> {
  let conforming = value.ok_or(InvalidHeaderErr::NoValue { key: SIMPLE })?;
  options.conforming = super::keyword_utils::parse_fits_bool(conforming)
    .map_err(|err| InvalidHeaderErr::FmtErr { key: SIMPLE, err })?;
  Ok(())
}

fn parse_bitpix(
  key: &str,
  value: Option<&str>,
  options: &mut FitsOptions,
) -> Result<(), InvalidHeaderErr> {
  options.bitpix = value
    .ok_or(InvalidHeaderErr::NoValue { key: BITPIX })?
    .parse()
    .map_err(|err| InvalidHeaderErr::fmt_err(BITPIX, err))?;
  Ok(())
}

fn insert_meta_tag(
  key: &str,
  value: &str,
  meta: &mut impl MetaContainer,
) -> Result<(), InvalidHeaderErr> {
  use super::keyword_utils::*;
  #[cfg_attr(rustfmt, rustfmt_skip)]
  Ok(match key {
    //Reserved kw describing observations
    DATE_OBS => set_creation_date(value, meta)?,
    DATE => set_modified_date(value, meta)?,
    AUTHOR => set_author(value, meta),
    REFERENC => set_refpub_title(value, meta),
    TELESCOP => set_telescope(value, meta),
    INSTRUME => set_instrument(value, meta),
    OBJECT => set_object(value, meta),
    BLANK => (), //do nothing
    other => { meta.insert_string_tag(other, value); }
  })
}

////////////////////////////////////////////////////////////////////////////////
//                                 UNIT TESTS                                 //
////////////////////////////////////////////////////////////////////////////////

#[test]
fn record_split_test() {
  const TEST_BLOCK: &str = "SIMPLE  =                    T  / FLIGHT22 05Apr96 RSH                          BITPIX  =                   16  / SIGNED 16-BIT INTEGERS                        NAXIS   =                    2  / 2-DIMENSIONAL IMAGES                          NAXIS1  =                  512  / SAMPLES PER LINE                              NAXIS2  =                  512  / LINES PER IMAGE                               EXTEND  =                    T  / FILE MAY HAVE EXTENSIONS                      DATATYPE= 'INTEGER*2'           / SAME INFORMATION AS BITPIX                    TELESCOP= 'UIT     '            / TELECOPE USED                                 INSTRUME= 'INTENSIFIED-FILM'    / DETECTOR USED                                 OBJECT  = 'NGC4151 '            / TARGET NAME                                   OBJECT2 = '_       '            / ALTERNATIVE TARGET NAME                       CATEGORY= 'FLIGHT  '            / TARGET CATEGORY                               JOTFID  = '8116-14 '            / ASTRO MISSION TARGET ID                       IMAGE   = 'FUV2582 '            / IMAGE NUMBER                                  ORIGIN  = 'UIT/GSFC'            / WHERE TAPE WRITTEN                            ASTRO   =                    2  / ASTRO MISSION NUMBER                          FRAMENO = 'b0582   '            / ANNOTATED FRAME NUMBER                        CATHODE = 'CSI     '            / IMAGE TUBE PHOTOCATHODE                       FILTER  = 'B1      '            / CAMERA/FILTER IDENTIFIER                      PDSDATIM= '06-JUL-1995  07:20'  / MICRODENSITOMETRY DATE & TIME                 PDSID   =                   21  / MICRODENSITOMETER IDENT                       PDSAPERT=                   20  / MICROD. APERTURE, MICRONS                     PDSSTEP =                   10  / MICROD. STEP SIZE, MICRONS                    PIXELSIZ=        8.0000000E+01  / CURRENT PIXEL SIZE, MICRONS                   EQUINOX =        2.0000000E+03  / EQUINOX OF BEST COORDINATES                   NOMRA   =             182.0044  / 1950 I.P.S.  R.A., DEGREES                    NOMDEC  =              39.6839  / 1950 I.P.S.  DEC., DEGREES                    NOMROLL =             323.9500  / I.P.S. ROLL ANGLE                             NOMSCALE=        5.6832500E+01  / NOMINAL PLATE SCL (ARCSEC/MM)                 CALIBCON=          5.00000E-16  / PREFLIGHT LAB CALIB FOR CAMERA                FEXPTIME= '8355    '            / EXPOSURE TIME, APPLICABLE FRM                 DATE-OBS= '13/03/95'            / DATE OF OBSERVATION (GMT)                     TIME-OBS=        6.2728000E+00  / TIME OF OBS (HOURS GMT)                       BSCALE  =        2.0587209E-16  / CALIBRATION CONST                             BUNIT   = 'ERGS/CM**2/S/ANGSTRM'                                                END     =              0.00000  / ADDITIVE CONST FOR CALIB.                     ";
  let recs: Vec<(&str, Option<&str>, Option<&str>)> = TEST_BLOCK
    .as_bytes()
    .chunks_exact(RECORD_SIZE)
    .map(|chunk| split_keyword_record(chunk))
    .collect();
  assert!(recs[0] == ("SIMPLE", Some("T"), Some("FLIGHT22 05Apr96 RSH")));
  assert!(recs[1] == ("BITPIX", Some("16"), Some("SIGNED 16-BIT INTEGERS")));
}

#[test]
fn continue_record_test() {
  //Setup dummy data
  const TEST_KEY: &str = "TEST";
  const TEST_RECS: [(&str, Option<&str>, Option<&str>); 8] = [
    (
      TEST_KEY,
      Some("'Lorem ipsum dolor sit amet, consectetur adipiscing elit. Aenean viverra rutru&'"),
      None,
    ),
    (
      CONTINUE,
      Some("'m ante nec facilisis. Praesent rutrum ipsum a volutpat lacinia. In hac habita&'"),
      None,
    ),
    (
      CONTINUE,
      Some("'sse platea dictumst. Nulla et volutpat urna. Phasellus luctus congue est, id &'"),
      None,
    ),
    (
      CONTINUE,
      Some("'interdum enim aliquam et. Morbi et ipsum mi. Maecenas pretium a metus sit ame&'"),
      None,
    ),
    (
      CONTINUE,
      Some("'t semper. Suspendisse non scelerisque libero. Pellentesque sit amet lectus ul&'"),
      None,
    ),
    (
      CONTINUE,
      Some("'lamcorper, ullamcorper velit non, feugiat lacus. Vestibulum pellentesque frin&'"),
      None,
    ),
    (
      CONTINUE,
      Some("'gilla ex at scelerisque. Integer vitae tincidunt tortor.'"),
      Some("done with this"),
    ),
    (END, None, None),
  ];
  const TEST_ANSWER: &str = "'Lorem ipsum dolor sit amet, consectetur adipiscing elit. Aenean viverra rutrum ante nec facilisis. Praesent rutrum ipsum a volutpat lacinia. In hac habitasse platea dictumst. Nulla et volutpat urna. Phasellus luctus congue est, id interdum enim aliquam et. Morbi et ipsum mi. Maecenas pretium a metus sit amet semper. Suspendisse non scelerisque libero. Pellentesque sit amet lectus ullamcorper, ullamcorper velit non, feugiat lacus. Vestibulum pellentesque fringilla ex at scelerisque. Integer vitae tincidunt tortor.'";
  let mut test_meta = MetaOnly::default();
  //run concat on the test keys!
  concat(TEST_RECS.into_iter(), &mut test_meta).unwrap();
  dbg!(&test_meta);
  assert!(test_meta.contains_string_tag(TEST_KEY));
  assert_eq!(TEST_ANSWER, test_meta.get_string_tag(TEST_KEY).unwrap());
}

#[test]
fn orphaned_continue_test() {
  const TEST_COMMENT: &str = "this is a comment";
  const TEST_RECS: [(&str, Option<&str>, Option<&str>); 3] = [
    ("TEST_GARBAGE", Some("value"), Some("comment")),
    (CONTINUE, Some(TEST_COMMENT), None),
    (CONTINUE, None, None),
  ];
  const META_ANSWER: (&str, &str) = ("TEST_GARBAGE", "value");
  let mut input_options = FitsOptions::new_invalid();
  todo!()
}

#[test]
fn invalid_novalue_continue_test() {
  const TEST_RECS: [(&str, Option<&str>, Option<&str>); 2] =
    [("GARBAGE", Some("'hmm&'"), None), (CONTINUE, None, None)];
  let mut dummy_options = FitsOptions::new_invalid();
  todo!()
}

#[test]
fn naxis_option_test() {
  //Setup dummy data
  const TEST_RECS: [(&str, Option<&str>, Option<&str>); 4] = [
    (NAXIS, Some("3"), None),
    ("NAXIS1", Some("1000"), None),
    ("NAXIS2", Some("2250"), None),
    ("NAXIS3", Some("272"), None),
  ];
  const TEST_ANSWER: [usize; 3] = [1000, 2250, 272];
  let mut input_options = FitsOptions::new_invalid();
  for (key, value, _comment) in TEST_RECS {
    parse_naxis(key, value, &mut input_options).unwrap();
  }
  assert!(input_options.dim == input_options.shape.len() as u16);
  assert!(input_options.shape.len() == TEST_ANSWER.len());
  assert!(input_options.shape == TEST_ANSWER);
}

#[test]
fn naxis_oob_test() {
  const TEST_RECS: (&str, Option<&str>, Option<&str>) = ("NAXIS1", Some("1200"), None);
  let mut input_options = FitsOptions::new_invalid();
  assert!(matches!(
    parse_naxis(TEST_RECS.0, TEST_RECS.1, &mut input_options),
    Err(InvalidHeaderErr::NaxisOob { idx: 1, naxes: 0 })
  ))
}

#[test]
fn invalid_novalue_simple_test() {
  const TEST_RECS: (&str, Option<&str>, Option<&str>) = (SIMPLE, None, None);
  let mut input_options = FitsOptions::new_invalid();
  assert!(matches!(
    parse_simple(TEST_RECS.0, TEST_RECS.1, &mut input_options),
    Err(InvalidHeaderErr::NoValue { .. })
  ));
}

#[test]
fn simple_option_test() {
  //Setup dummy data
  const TEST_RECS: (&str, Option<&str>, Option<&str>) = (SIMPLE, Some("T"), None);
  const TEST_ANSWER: bool = true;
  let mut input_options = FitsOptions::new_invalid();
  parse_simple(TEST_RECS.0, TEST_RECS.1, &mut input_options).unwrap();
  assert!(input_options.conforming == TEST_ANSWER);
}

#[test]
fn bitpix_option_test() {
  //Setup dummy data
  const TEST_RECS: (&str, Option<&str>, Option<&str>) = (BITPIX, Some("-32"), None);
  const TEST_ANSWER: i8 = -32;
  let mut input_options = FitsOptions::new_invalid();
  parse_bitpix(TEST_RECS.0, TEST_RECS.1, &mut input_options).unwrap();
  assert!(input_options.bitpix == TEST_ANSWER);
}

#[test]
fn invalid_novalue_bitpix_test() {
  const TEST_RECS: (&str, Option<&str>, Option<&str>) = (BITPIX, None, None);
  let mut input_options = FitsOptions::new_invalid();
  assert!(matches!(
    parse_bitpix(TEST_RECS.0, TEST_RECS.1, &mut input_options),
    Err(InvalidHeaderErr::NoValue { .. })
  ));
}
