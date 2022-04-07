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

use std::fmt::{Display, Formatter, self};

use simple_error::{SimpleError, try_with};

use crate::raw::table_entry_format::TableEntryFormat;

#[derive(Debug, Clone)]
pub enum TableEntry {
    Text(String),
    Int(i64),
    Float(f64)
}

impl Display for TableEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            Self::Text(txt) => format!("{txt} (string)"),
            Self::Int(num) => format!("{num} (int)"),
            Self::Float(num) => format!("{num} (float)")
        })
    }
}

impl TableEntry {
    pub(crate) fn from_parts(raw_field: &str, format: &TableEntryFormat)
        -> Result<Self, SimpleError>
    {
        //(1) Check if the field is as long as was specified in the format
        if format.get_field_width() != raw_field.len() {return Err(SimpleError::new(
            "Error while decoding table: field size did not match format size"
        ));}

        //(2) Match the format (and don't forget to strip spaces of the numeric
        //    variants before parsing them!)
        Ok(match format {
            TableEntryFormat::Char(_) => {
                Self::Text(String::from(raw_field))
            } TableEntryFormat::Int(_) => {
                Self::Int(try_with!(str::parse(raw_field.trim()), ""))
            } TableEntryFormat::Float(_) => {
                Self::Float(try_with!(str::parse(raw_field.trim()), ""))
            } TableEntryFormat::Invalid(invalid_format) => { return Err(
                SimpleError::new(format!(
                    "Error while decoding table: '{invalid_format}' is not a valid Fortran formatting code"
                ))
            );}
        })
    }
}