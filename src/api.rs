/*
  Copyright © 2022 Raúl Wolters

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

//! This module contains all elements of the public rustronomy-fits API. The API
//! is exposed by re-exporting modules.
//!
//! The crate-level lib.rs re-exports all elements in this module, in addition
//! to a subset exposed via the prelude for ease-of-use.

pub mod fits;
pub mod hdu;
