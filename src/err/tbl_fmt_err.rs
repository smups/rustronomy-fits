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
    fmt::{self, Display, Formatter}
};

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