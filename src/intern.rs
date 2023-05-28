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

//module structure
mod file_io;
mod fits_opts;
mod hdu_io;
mod header_io;
mod keyword_utils;
mod test_io;

pub mod extensions;

//re-exports
pub use file_io::*;
pub use fits_opts::*;
pub use hdu_io::*;

pub mod fits_consts {
  //Constants defined by the FITS standard
  pub const BLOCK_SIZE: usize = 2880;
  pub const RECORD_SIZE: usize = 80;

  //Comment separator
  pub const SEP: char = '/';

  //Valid values for the XTENSION keyword
  pub const XT_IMAGE: &str = "IMAGE   ";
  pub const XT_TABLE: &str = "TABLE   ";
  pub const XT_BINTABLE: &str = "BINTABLE";
  pub const XT_IUEIMAGE: &str = "IUEIMAGE";
  pub const XT_A3DTABLE: &str = "A3DTABLE";
  pub const XT_FOREIGN: &str = "FOREIGN ";
  pub const XT_DUMP: &str = "DUMP    ";

  //Keywords that correspond to FITS options
  pub const SIMPLE: &str = "SIMPLE";
  pub const BITPIX: &str = "BITPIX";
  pub const NAXIS: &str = "NAXIS";
  pub const END: &str = "END";
  pub const GCOUNT: &str = "GCOUNT";
  pub const PCOUNT: &str = "PCOUNT";
  pub const GROUPS: &str = "GROUPS";
  pub const PTYPE: &str = "PTYPE";
  pub const PSCAL: &str = "PSCAL";
  pub const PZERO: &str = "PZERO";
  pub const TFIELDS: &str = "TFIELDS";
  pub const TBCOL: &str = "TBCOL";
  pub const TFORM: &str = "TFORM";
  pub const TTYPE: &str = "TTYPE";
  pub const TUNIT: &str = "TUNIT";
  pub const TSCAL: &str = "TSCAL";
  pub const TZERO: &str = "TZERO";
  pub const TNULL: &str = "TNULL";
  pub const TDISP: &str = "TDISP";
  pub const THEAP: &str = "THEAP";
  pub const EXTEND: &str = "EXTEND";
  pub const INHERIT: &str = "INHERIT";
  pub const XTENSION: &str = "EXTENSION";
  pub const BSCALE: &str = "BSCALE";
  pub const BZERO: &str = "BZERO";
  pub const BUNIT: &str = "BUNIT";
  pub const DATASUM: &str = "DATASUM";
  pub const CHECKSUM: &str = "CHECKSUM";
  pub const CONTINUE: &str = "CONTINUE";
  pub const COMMENT: &str = "COMMENT";
  pub const HISTORY: &str = "HISTORY";
  pub const BLANK: &str = "BLANK";

  //FITS keywords that correspond to keywords that rustronomy understands
  pub const DATE: &str = "DATE";
  pub const DATE_OBS: &str = "DATE-OBS";
  pub const AUTHOR: &str = "AUTHOR";
  pub const REFERENC: &str = "REFERENC";
  pub const TELESCOP: &str = "TELESCOP";
  pub const INSTRUME: &str = "INSTRUME";
  pub const OBJECT: &str = "OBJECT";
  pub const ORIGIN: &str = "ORIGIN";
  pub const OBSERVER: &str = "OBSERVER";
}
