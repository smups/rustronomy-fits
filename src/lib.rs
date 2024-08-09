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

#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

//Module structure
mod bitpix;
mod err;
mod extensions;
mod fits;
mod header;
mod header_data_unit;
mod raw;

//Constants defined by the FITS standard
pub(crate) const BLOCK_SIZE: usize = 2880;

//Public api re-exports
pub use err::*;
pub use extensions::Extension;
pub use fits::Fits;
pub use header::Header;
pub use header_data_unit::HeaderDataUnit;

//prelude (kinda pointless rn but whatev)
pub mod prelude {
  pub use crate::err::*;
  pub use crate::extensions::Extension;
  pub use crate::fits::Fits;
  pub use crate::header::Header;
  pub use crate::header_data_unit::HeaderDataUnit;
}
