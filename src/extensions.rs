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

use std::{fmt::Display, error::Error};

use crate::{
    raw::{BlockSized, raw_io::RawFitsWriter},
    io_err::{self, InvalidFitsFileErr as IFFErr}
};

use self::{
    image::{TypedImage, ImgParser},
    table::{AsciiTable, AsciiTblParser}
};

//FITS standard-conforming extensions
pub mod image;
pub mod table;

#[derive(Debug, Clone)]
pub enum Extension{
    /*  THIS IS PART OF THE USER-FACING API
        Users receive a FITS struct, within which they may access the header and
        data. The data is provided as a variant of this Extension struct. 
        
        All implementations of this struct are however internal!
    */
    Corrupted,
    Image(TypedImage),
    AsciiTable(AsciiTable)
}

impl BlockSized for Extension {
    fn get_block_len(&self) -> usize {
        match &self {
            Self::Corrupted => 0, //corrupted data is disregarded
            Self::Image(img) => img.get_block_len(),
            Self::AsciiTable(tbl) => tbl.get_block_len()
        }
    }
}

impl Display for Extension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::Corrupted => write!(f, "(CORRUPTED_DATA)"),
            Self::Image(img) => write!(f, "{}", img.xprint()),
            Self::AsciiTable(tbl) => write!(f, "{}", tbl.xprint())
        }
    }
}

impl Extension {
    pub(crate) fn write_to_buffer(self, writer: &mut RawFitsWriter)
        -> Result<(), Box<dyn Error>>
    {
        match self {
            Self::Corrupted => return Err(Box::new(IFFErr::new(io_err::CORRUPTED))),
            Self::Image(img) => {
                ImgParser::encode_img(img, writer)
            },
            Self::AsciiTable(tbl) => {
                AsciiTblParser::encode_tbl(tbl, writer)
            }
        }
    }
}

pub(crate) trait ExtensionPrint{
    //This tiny trait is used for printing concise descriptions of Extensions
    fn xprint(&self) -> String;
}