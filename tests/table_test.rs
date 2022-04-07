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
use std::path::PathBuf;

use rustronomy_fits as rsf;

static TABLE_FILE: &str = "resources/Hubble_HRS.fits";

#[test]
fn read_test() {

    let mut real = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    real.push(TABLE_FILE);

    let mut fits = rsf::Fits::open(&real).unwrap();
    print!("{fits}");

    //Inspect the table
    let (_h, xt) = fits.remove_hdu(1).unwrap().to_parts();
    let tbl = match xt.unwrap() {
        rsf::Extension::Table(tbl) => tbl,
        _ => panic!()
    };
    println!("{tbl}");
}