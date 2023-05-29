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

use std::error::Error;

use crate::{api::io::*, hdu::Hdu};

pub fn read_hdu(reader: &mut (impl FitsReader + Send)) -> Result<Hdu, Box<dyn Error>> {
  //(0) Create a new HDU
  let mut hdu = Hdu::default();

  /*(1)
   * Decode the header. This reads the header part of the Header Data and splits
   * it into fits_options (parameters required to read the data part of the HDU)
   * and metadata, which is added to the hdu (this is why hdu has to be passed as
   * a mutable paramter!)
   */
  let fits_options = super::header_io::read_header(reader, &mut hdu)?;

  /*(2)
   * Decode the data part of the HDU. Which decoding method should be used
   * depends on the data stored in the HDU, which can be derived from the fits
   * options we previously decoded.
   */
  use super::Extension::*;
  let data = match fits_options.determine_data_type()? {
    Image => super::extensions::read_image_hdu(&fits_options, reader)?,
    other => todo!(),
  };

  //(R) Replace the data in the Hdu and return it
  hdu.replace_data(data);
  Ok(hdu)
}

pub fn write_hdu(hdu: Hdu, writer: &mut impl FitsWriter) -> Result<(), Box<dyn Error>> {
  todo!()
}

////////////////////////////////////////////////////////////////////////////////
//                                 UNIT TESTS                                 //
////////////////////////////////////////////////////////////////////////////////

#[test]
pub fn read_header_test() {
  let mut test_writer = super::test_io::mock_data::EUVE.clone();
  let mut hdu0 = Hdu::default();
  let opts = super::header_io::read_header(&mut test_writer, &mut hdu0).unwrap();
  println!("{hdu0:?}");
  println!("{opts:?}");
}
