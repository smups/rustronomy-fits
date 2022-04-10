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
    error::Error
};

use simple_error::SimpleError;

use crate::{raw::BlockSized, extensions::ExtensionPrint};

use super::{column::AsciiCol, TableEntry};

/*  Description:
    This is the abstracted user-facing api for tables. The 
*/
#[derive(Debug, Clone)]
pub struct Table{
    cols: Vec<Box<dyn AsciiCol>>,
    block_size: Option<usize>
}

impl BlockSized for Table {
    fn get_block_len(&self) -> usize {
        match self.block_size {
            Some(size) => size,
            None => {
                //We have to calculate the size of the table manually, as it is
                //not currently known (this is the case for user-created tables)
                todo!()
            }
        }
    }
}

impl Display for Table {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f,">================================<|FITS Table|>=================================")?;
        writeln!(f, ">Table Layout:")?;
        for (index, col) in self.cols.iter().enumerate(){
            writeln!(f, ">  col#{index:03} - {}", col.as_ref().pretty_print())?
        }
        writeln!(f,">===============================================================================")?;
        Ok(())
    }
}

impl ExtensionPrint for Table {
    fn xprint(&self) -> String {
        format!("(TABLE) - #columns: {}, #rows: {}, size: {}",
            self.cols.len(),
            match self.cols.get(0) {
                None => 0,
                Some(col_ref) => col_ref.len() 
            },
            self.get_block_len()
        )
    }
}

impl Table {

    /*
        PUBLIC API
    */

    pub fn get_entry(&self, col: usize, row: usize) -> Result<TableEntry, Box<dyn Error>> {
        
        //(1) Check if the column index is valid -> if yes, get the column
        if col >= self.cols.len() {return Err(Box::new(SimpleError::new(format!(
            "Column index {col} is out of range for a table with {} columns!",
            self.cols.len()
        ))));}
        let column = self.cols.get(col).unwrap().as_ref();

        //(2) get the entry from the column
        match column.get_entry(row) {
            Some(entry) => Ok(entry),
            None => Err(Box::new(SimpleError::new(format!(
                "Row index {row} is out of range for column #{col} with length {}",
                column.len()
            ))))
        }
    }

    pub fn get_column(&self, index: usize) -> Option<&dyn AsciiCol> {
        match self.cols.get(index) {
            Some(boxed) => Some(boxed.as_ref()),
            None => None
        }
    }

    /*
        INTERNAL FUNCS
    */
    pub(crate) fn new_sized(cols: Vec<Box<dyn AsciiCol>>, size: usize) -> Self {
        Table { cols: cols, block_size: Some(size) }
    }

    pub(crate) fn add_row(&mut self, row: Vec<TableEntry>)
        -> Result<(), SimpleError>
    {
        if row.len() != self.cols.len() { return Err(SimpleError::new(
            "Error when adding row to table: number of columns in table and the number of fields in row don't match!"
        ));
        }

        //Add row to the table
        for (index, entry) in row.into_iter().enumerate() {
            self.cols[index].push_entry(entry)?;
        }
        
        //(R) ok
        Ok(())
    }
}