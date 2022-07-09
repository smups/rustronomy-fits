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
};

use crate::{
  raw::table_entry_format::TableEntryFormat,
  tbl_fmt_err::{FieldSizeMisMatch, InvalidFFCode, ParseError},
};

#[derive(Debug, Clone)]
pub enum TableEntry {
  Text(String),
  Int(i64),
  Float(f64),
}

impl Display for TableEntry {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    use TableEntry::*;
    write!(
      f,
      "{}",
      match self {
        Text(txt) => format!("{txt} (string)"),
        Int(num) => format!("{num} (int)"),
        Float(num) => format!("{num} (float)"),
      }
    )
  }
}

impl TableEntry {
  pub(crate) fn from_parts(raw_field: &str, format: &TableEntryFormat) -> Result<Self, ParseError> {
    //(1) Check if the field is as long as was specified in the format
    if format.get_field_width() != raw_field.len() {
      return Err(FieldSizeMisMatch::new(format, raw_field).into());
    }

    //(2) Match the format (and don't forget to strip spaces of the numeric
    //    variants before parsing them!)
    use TableEntryFormat::*;

    Ok(match format {
      Char(_) => Self::Text(String::from(raw_field)),
      Int(_) => Self::Int(str::parse(raw_field.trim())?),
      Float(_) => Self::Float(str::parse(raw_field.trim())?),
      Invalid(invalid_format) => {
        return Err(InvalidFFCode::new(invalid_format.to_string()).into());
      }
    })
  }

  pub(crate) fn type_print(&self) -> String {
    use TableEntry::*;
    match &self {
      Text(_) => String::from("(string)"),
      Int(_) => String::from("(int)"),
      Float(_) => String::from("(float)"),
    }
  }

  pub(crate) fn txt() -> Self {
    Self::Text(String::from(""))
  }
  pub(crate) fn int() -> Self {
    Self::Int(0)
  }
  pub(crate) fn float() -> Self {
    Self::Float(0.0)
  }
}
