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

/*  Description:
    Entries in a FITS table may have various different types, encoded using Fortran
    formatting codes. These are listed in the header of the table. This file
    contains the machinery to convert these codes to enum variants.
*/

use std::{
    fmt::{Display, Formatter},
    num::ParseIntError
};

use crate::tbl_fmt_err::{
    InvalidFFCode as IFFCErr
};

#[derive(Debug, Clone)]
pub(crate) enum TableEntryFormat {
    Char(usize),
    Int(usize),
    Float((usize, usize)),
    Invalid(String)
}

impl TableEntryFormat {

    pub(crate) fn from_fortran_format_code(ff_code: &str)
        -> Result<TableEntryFormat, ParseIntError>
    {
        use TableEntryFormat::*;

        //Formats are encoded as strings with the usual annoying {'}s around
        //them. We have to remove these and strip the string
        let parsed_code = ff_code.replace("'", "");
        let mut parsed_code = parsed_code.trim();

        //First character of the string determines the data type
        let dtype = parsed_code.chars().next().unwrap();
        parsed_code = parsed_code.strip_prefix(dtype).unwrap();

        //Remainder of the strings are the two integers w and d(optional) indic-
        //ating the length of the field in characters and the number of digits
        //to the right of the decimal point respectivally
        let rem: Vec<&str> = parsed_code.split(".").collect();
       
        if rem.len() == 2 {
            //These format types have both a {w} and a {d} value
            match dtype {
                'F' | 'E' | 'D' => Ok(Float((
                    str::parse::<usize>(rem[0])?,
                    str::parse::<usize>(rem[1])?
                ))),
                _ => Ok(Invalid(String::from(parsed_code)))
            }
        } else if rem.len() == 1 {
            //These format types only have a w value
            match dtype {
                'A' => Ok(Char(str::parse::<usize>(rem[0])?)),
                'I' => Ok(Int(str::parse::<usize>(rem[0])?)),
                _ => Ok(Invalid(String::from(parsed_code)))
            }
        } else {
            //These are nonsensical formatting codes
            Ok(Invalid(String::from(parsed_code)))
        }
    }

    pub(crate) fn to_fortran_format_code(&self) -> Result<String, IFFCErr> {
        use TableEntryFormat::*;
        Ok(match &self {
            Char(w) => format!("A{w}"),
            Int(w) => format!("I{w}"),
            Float((w, d)) => format!("E{w}.{d}"),
            Invalid(val) => return Err(IFFCErr::new(val.to_string()))
        })
    }

    pub(crate) fn get_field_width(&self) -> usize {
        use TableEntryFormat::*;
        match self {
            Char(w) => *w,
            Int(w) => *w,
            Float((w, _d)) => *w,
            Invalid(string) => string.len()
        }
    }
}

impl Display for TableEntryFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use TableEntryFormat::*;
        write!(f, "{}", match self {
            Char(_) => "string",
            Int(_) => "integer",
            Float(_) => "float",
            Invalid(_) => "INVALID"
        })?;
        Ok(())
    }
}