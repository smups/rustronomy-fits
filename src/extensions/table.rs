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

//Module Structure
pub mod table_entry;
pub mod column;
pub mod ascii_table;
pub mod bin_table;
pub(crate) mod ascii_tbl_parser;

//Re-exports for readability
pub use table_entry::TableEntry as TableEntry;
pub use ascii_table::AsciiTable as AsciiTable;
pub(crate) use ascii_tbl_parser::AsciiTblParser as AsciiTblParser;