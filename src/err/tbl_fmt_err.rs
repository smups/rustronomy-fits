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
    error::Error,
    fmt::{self, Display, Formatter},
    num::{ParseIntError, ParseFloatError}
};

use crate::raw::table_entry_format::TableEntryFormat;

#[derive(Debug)]
pub struct InvalidFFCode {
    /*
        This error is thrown when an instance tries to access a table format
        containing an invalid Fortran Formatting code.
    */
    invld_code: String
}

impl Error for InvalidFFCode {}
impl Display for InvalidFFCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Error while accessing table entry: '{}' is not a valid Fortran Formatting code. See documentation for more info",
            self.invld_code
        )
    }
}

impl InvalidFFCode {
    pub(crate) fn new(invalid_code: String) -> Self {
        InvalidFFCode { invld_code: invalid_code }
    }
}

#[derive(Debug)]
pub struct FieldSizeMisMatch {
    buf_size: usize,
    fmt_field_size: usize
}

impl Error for FieldSizeMisMatch {}
impl Display for FieldSizeMisMatch {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f,
            "Error while decoding table entry: format specifies field size {}, but received buffer with size {}",
            self.fmt_field_size, self.buf_size
        )
    }
}

impl FieldSizeMisMatch {
    pub(crate) fn new(fmt: &TableEntryFormat, field: &str) -> Self {
        FieldSizeMisMatch{
            buf_size: field.len(),
            fmt_field_size: fmt.get_field_width()
        }
    }
}

#[derive(Debug)]
pub enum ParseError {
    FieldSizeMisMatch(FieldSizeMisMatch),
    ParseIntError(ParseIntError),
    ParseFloatError(ParseFloatError),
    InvalidFFCode(InvalidFFCode)
}

impl Error for ParseError {}
impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Error while parsing table entry: '{}'", self)
    }
}

impl From<FieldSizeMisMatch> for ParseError {
    fn from(err: FieldSizeMisMatch) -> Self {
        ParseError::FieldSizeMisMatch(err)
    }
}

impl From<ParseIntError> for ParseError {
    fn from(err: ParseIntError) -> Self {
        ParseError::ParseIntError(err)
    }
}

impl From<ParseFloatError> for ParseError {
    fn from(err: ParseFloatError) -> Self {
        ParseError::ParseFloatError(err)
    }
}

impl From<InvalidFFCode> for ParseError {
    fn from(err: InvalidFFCode) -> Self {
        ParseError::InvalidFFCode(err)
    }
}