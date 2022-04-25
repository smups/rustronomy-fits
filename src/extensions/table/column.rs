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

use std::{error::Error, fmt::Debug};

use dyn_clone::{DynClone, clone_trait_object};
use rayon::prelude::*;

use crate::tbl_err::{
    IndexOutOfRangeErr,
    TypeMisMatchErr,
    TblDecodeErr
};

use super::TableEntry;

pub trait AsciiCol: Debug + DynClone {
    /*  PUBLIC API
        End-users will recieve a Table struct containing boxed columns. They
        may modify the entries in each column, or remove/replace/reorder columns.
        All user interaction with columns is defined in this trait.
    */

    //Funcs for modifying/adding/removing entries in the column
    fn push_entry(&mut self, entry: TableEntry) -> Result<(), TypeMisMatchErr>;
    fn pop_entry(&mut self) -> Option<TableEntry>;
    fn set_entry(&mut self, entry: TableEntry, index: usize) -> Result<(), TblDecodeErr>;
    fn get_entry(&self, index: usize) -> Option<TableEntry>;
    fn remove_entry(&mut self, index: usize) -> Option<TableEntry>;

    //Funcs for properly encoding/decoding
    fn to_ascii_vec(&self) -> Vec<String>;

    //Other funcs
    fn len(&self) -> usize;
    fn get_col_label(&self) -> Option<&str>;
    fn pretty_print(&self) -> String;
}

//This macro makes Col a clonable trait object
clone_trait_object!(AsciiCol);

#[derive(Debug, Clone)]
pub(crate) struct Column<T> {
    /*
        Internal datacontainer for columns of a FITS table. All entries in a
        column have the same type. Instead of storing strings (like the FITS
        standard does), we will store actual primitive types and convert the
        Fortran-formatted strings when the table is opened.

        Columns may be labeled as per the FITS standard.
    */
    label: Option<String>,
    container: Vec<T>
}

impl<T> Column<T> {
    pub(crate) fn new(label: Option<String>) -> Self {
        Column { label: label, container: Vec::new() }
    }
}

impl AsciiCol for Column<String> {

    fn push_entry(&mut self, entry: TableEntry) -> Result<(), TypeMisMatchErr> {
        match entry {
            TableEntry::Text(txt) => Ok(self.container.push(txt)),
            other => Err(TypeMisMatchErr::new(TableEntry::txt(), &entry))
        }
    }

    fn pop_entry(&mut self) -> Option<TableEntry> {
        match self.container.pop() {
            Some(val) => Some(TableEntry::Text(val)),
            None => None
        }
    }

    fn set_entry(&mut self, entry: TableEntry, index: usize)
        -> Result<(), TblDecodeErr>
    {
        match entry {
            TableEntry::Text(txt) => {
                if self.container.len() >= index {
                    Err(
                        IndexOutOfRangeErr::from_idx(
                            (None, index), (None, self.container.len())
                        ).into()
                    )
                } else {
                    self.container[index] = txt;
                    Ok(())
                }
            } other => Err(TypeMisMatchErr::new(TableEntry::txt(), &entry).into())
        }
    }

    fn get_entry(&self, index: usize) -> Option<TableEntry> {
        match self.container.get(index) {
            Some(txt) => Some(TableEntry::Text(txt.to_string())),
            None => None
        }
    }

    fn remove_entry(&mut self, index: usize) -> Option<TableEntry> {
        if self.container.len() >= index {None}
        else {Some(TableEntry::Text(self.container.remove(index)))}
    }

    fn len(&self) -> usize {self.container.len()}

    fn to_ascii_vec(&self) -> Vec<String> {
        self.container.par_iter()
            .map(|primitive| primitive.to_string())
            .collect()
    }

    fn get_col_label(&self) -> Option<&str> {
        match &self.label {
            Some(label) => Some(label.as_str()),
            None => None
        }
    }

    fn pretty_print(&self) -> String {
        format!("label: {}, dtype: string", match &self.label {
            Some(label) => label,
            None => "(no label)"
        })
    }

}

impl AsciiCol for Column<i64> {

    fn push_entry(&mut self, entry: TableEntry) -> Result<(),  TypeMisMatchErr> {
        match entry {
            TableEntry::Int(num) => Ok(self.container.push(num)),
            other => Err(TypeMisMatchErr::new(TableEntry::int(), &entry))
        }
    }

    fn pop_entry(&mut self) -> Option<TableEntry> {
        match self.container.pop() {
            Some(val) => Some(TableEntry::Int(val)),
            None => None
        }
    }

    fn set_entry(&mut self, entry: TableEntry, index: usize)
        -> Result<(), TblDecodeErr>
    {
        match entry {
            TableEntry::Int(num) => {
                if self.container.len() >= index {
                    Err(
                        IndexOutOfRangeErr::from_idx(
                            (None, index), (None, self.container.len())
                        ).into()
                    )
                } else {
                    self.container[index] = num;
                    Ok(())
                }
            } other => return Err(TypeMisMatchErr::new(TableEntry::int(), &entry).into())
        }
    }

    fn get_entry(&self, index: usize) -> Option<TableEntry> {
        match self.container.get(index) {
            Some(num) => Some(TableEntry::Int(*num)),
            None => None
        }
    }

    fn remove_entry(&mut self, index: usize) -> Option<TableEntry> {
        if self.container.len() >= index {None}
        else {Some(TableEntry::Int(self.container.remove(index)))}
    }

    fn len(&self) -> usize {self.container.len()}

    fn to_ascii_vec(&self) -> Vec<String> {
        self.container.par_iter()
            .map(|primitive| primitive.to_string())
            .collect()
    }

    fn get_col_label(&self) -> Option<&str> {
        match &self.label {
            Some(label) => Some(label.as_str()),
            None => None
        }
    }

    fn pretty_print(&self) -> String {
        format!("label: {}, dtype: int", match &self.label {
            Some(label) => label,
            None => "(no label)"
        })
    }
    
}

impl AsciiCol for Column<f64> {

    fn push_entry(&mut self, entry: TableEntry) -> Result<(), TypeMisMatchErr> {
        match entry {
            TableEntry::Float(num) => Ok(self.container.push(num)),
            other => Err(TypeMisMatchErr::new(TableEntry::float(), &entry))
        }
    }

    fn pop_entry(&mut self) -> Option<TableEntry> {
        match self.container.pop() {
            Some(val) => Some(TableEntry::Float(val)),
            None => None
        }
    }

    fn set_entry(&mut self, entry: TableEntry, index: usize)
        -> Result<(), TblDecodeErr>
    {
        match entry {
            TableEntry::Float(num) => {
                if self.container.len() >= index {
                    Err(
                        IndexOutOfRangeErr::from_idx(
                            (None, index), (None, self.container.len())
                        ).into()
                    )
                } else {
                    self.container[index] = num;
                    Ok(())
                }
            } other => return Err(TypeMisMatchErr::new(TableEntry::float(), &entry).into())
        }
    }

    fn get_entry(&self, index: usize) -> Option<TableEntry> {
        match self.container.get(index) {
            Some(num) => Some(TableEntry::Float(*num)),
            None => None
        }
    }

    fn remove_entry(&mut self, index: usize) -> Option<TableEntry> {
        if self.container.len() >= index {None}
        else {Some(TableEntry::Float(self.container.remove(index)))}
    }

    fn len(&self) -> usize {self.container.len()}

    fn to_ascii_vec(&self) -> Vec<String> {
        self.container.par_iter()
            .map(|primitive| primitive.to_string())
            .collect()
    }

    fn get_col_label(&self) -> Option<&str> {
        match &self.label {
            Some(label) => Some(label.as_str()),
            None => None
        }
    }

    fn pretty_print(&self) -> String {
        format!("label: {}, dtype: float", match &self.label {
            Some(label) => label,
            None => "(no label)"
        })
    }
    
}