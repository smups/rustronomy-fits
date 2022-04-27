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

#[derive(Debug)]
pub struct KeywordRecordBufferErr {
    /*
        This error may be thrown when decoding a keyword record. It signifies
        that the provided buffer was not exactly 80 bytes long.
    */
    msg: &'static str
}

//List of possible messages:
pub const BUFFER_LEN: &'static str = "Keyword record buffer was not exactly 80 bytes long";
pub const ILLEGAL_CHAR: &'static str = "Keyword record contains illegal characters";

impl Error for KeywordRecordBufferErr {}
impl Display for KeywordRecordBufferErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl From<Utf8Error> for KeywordRecordBufferErr {
    fn from(_: Utf8Error) -> Self {
        Self::new(ILLEGAL_CHAR)
    }
}

impl KeywordRecordBufferErr {
    pub fn new(msg: &'static str) -> Self {
        Self{ msg: msg }
    }
}

#[derive(Debug)]
pub struct ProtectedKeywordErr {
    /*
        This error may be thrown when an instance tries to modify a protected
        keyword. 
    */
    keyword: &'static str
}

impl Error for ProtectedKeywordErr {}
impl Display for ProtectedKeywordErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f, 
            "Cannot create a keyword record for ({}), since it is a protected keyword. See documentation for more info",
            self.keyword
        )
    }
}

impl ProtectedKeywordErr {
    pub fn new(kw: &'static str) -> Self {
        ProtectedKeywordErr { keyword: kw }
    }
}