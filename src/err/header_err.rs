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
    fmt::{Display, Formatter, self},
    error::Error, str::Utf8Error
};

use crate::keyword_err::KeywordRecordBufferErr;

#[derive(Debug)]
pub struct HeaderBlockBufferErr {
    /*
        This error may be thrown when decoding a header. It signifies
        that the provided buffer was not exactly 80 bytes long.
    */
    msg: String
}

//List of possible messages:
pub const BUFFER_LEN: &'static str = "header buffer was not exactly 1 FITS block (2880b) long";

impl Error for HeaderBlockBufferErr {}
impl Display for HeaderBlockBufferErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl From<KeywordRecordBufferErr> for HeaderBlockBufferErr{
    fn from(err: KeywordRecordBufferErr) -> Self {
        HeaderBlockBufferErr {
            msg: format!("Error while accessing header buffer: '{}'", &err)
        }
    }
}

impl HeaderBlockBufferErr {
    pub fn new(msg: &'static str) -> Self {
        Self{ msg: msg.to_string() }
    }
}