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

use std::{
    fmt::{self, Formatter, Display}, 
    error::Error
};

use crate::{
    extensions::image::TypedImage,
    bitpix::Bitpix
};

#[derive(Debug)]
pub struct WrongImgTypeErr {
    /*
        This error may be thrown when opening a FITS file. If the FITS
        file has invalid encoding (for whatever reason), this error will be
        thrown.
    */
    img_type: Bitpix,
    wrong_type: Bitpix
}

impl Error for WrongImgTypeErr {}
impl Display for WrongImgTypeErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f,
            "Tried to access {:?} type array as an {} type array. Implicit array type conversions are disallowed",
            self.img_type, self.wrong_type
        )
    }
}

impl WrongImgTypeErr {
    pub(crate) fn new(img: &TypedImage, wrong_type: Bitpix) -> Self {
        WrongImgTypeErr{ img_type: img.bpx(), wrong_type: wrong_type }
    }
}

#[derive(Debug)]
pub struct InvalidMemLayout {

}

impl Error for InvalidMemLayout {}
impl Display for InvalidMemLayout {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Error while encoding image: underlying ndarray uses unsupported memory layout. Only continuous layouts are supported")
    }
}

impl InvalidMemLayout {
    pub(crate) fn new() -> Self{ InvalidMemLayout{} }
}