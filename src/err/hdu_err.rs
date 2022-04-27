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
    fmt::{Display, Formatter, self}, ops::Not
};

#[derive(Debug)]
pub struct MissingRecordError {
    /*
        This error may be thrown when encoding/decoding a header data unit. It
        signifies that a keyword that was mandatory as per the FITS standard is
        missing from a header.
        (*) Example: decoding an image without the NAXIS keyword throws this err
    */
    missing_keyword: String
}

impl Error for MissingRecordError {}
impl Display for MissingRecordError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f,
            "Keyword {} required for decoding/encoding this HDU is missing!",
            self.missing_keyword
        )?;
        Ok(())
    }
}
impl MissingRecordError {
    pub fn new(missing_keyword: &str) -> Self {
        MissingRecordError{missing_keyword: String::from(missing_keyword)}
    }
}

#[derive(Debug)]
pub struct InvalidRecordValueError {
    /*
        This error may be thrown when encoding/decoding a header data unit. It
        signifies that a keyword does not have the fixed value it was assigned
        by the FITS standard.
        (*) Example: SIMPLE = F throws this err
    */
    keyword: String,
    invalid_value: String,
    allowed_values: &'static [&'static str]
}

impl Error for InvalidRecordValueError {}
impl Display for InvalidRecordValueError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f,
            "The value '{}' is not allowed for the keyword ({}). Allowed values are: {:?}",
            self.invalid_value,
            self.keyword,
            self.allowed_values
        )?;
        Ok(())
    }
}
impl InvalidRecordValueError {
    pub fn new(keyword: &str, invalid_value: &str, allowed_values: &'static[&str])
        -> Self
    {
        InvalidRecordValueError{
            keyword: String::from(keyword),
            invalid_value: String::from(invalid_value),
            allowed_values: allowed_values
        }
    }
}

#[derive(Debug)]
pub struct NotImplementedErr {
    //thrown when accessing extension that was not implemented
    xtnsion: String
}

impl Error for NotImplementedErr {}
impl Display for NotImplementedErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Error while constructing HDU: extension {} not yet implemented",
            self.xtnsion
        )
    }
}

impl NotImplementedErr {
    pub fn new(xtnsion: String) -> Self {
        NotImplementedErr { xtnsion: xtnsion }
    }
}