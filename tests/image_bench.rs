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
  fs::{self, DirEntry},
  io::{self, Write},
  path::PathBuf,
  time::Instant,
};

use progressing::{mapping::Bar, Baring};

use dirs;
use rustronomy_fits as rfs;

//Starfields of M37 taken by myself
const BENCH_FOLDER: &str = "resources/bench_data";

//output folder
const OUTPUT: &str = "resources/bench_out";

#[test]
fn read_write_benchmark() {
  /*  Description:
      This test reads 65 ~7MB FITS files containing Images of
  */

  //Root folder of project
  let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

  let mut data_f = root.clone();
  data_f.push(BENCH_FOLDER);

  let mut result_f = root.clone();
  result_f.push(OUTPUT);

  //Log files and times
  let mut read_times = Vec::new();
  let mut write_times = Vec::new();

  //Get all the test files and setup a progress bar
  let files = fs::read_dir(data_f).unwrap().collect::<Result<Vec<DirEntry>, io::Error>>().unwrap();
  let mut progress_bar = Bar::with_range(0, *&files.len()).timed();
  println!("Performing read/write image benchmark...");

  //These are all ~7MB files. THIS IS CURRENTLY TOO SLOW!
  //Results: (Ryzen 3600X, NVME SSD)
  //  - average read time ~40ms
  //  - average write time ~400ms
  for (index, path) in files.into_iter().enumerate() {
    //(1) Read
    let now = Instant::now();
    let fits = rfs::Fits::open(&path.path()).unwrap();
    let time = now.elapsed().as_millis();
    read_times.push(time);

    //(2) Write
    let mut write_path = dirs::cache_dir().unwrap();
    write_path.push(format!("bench{index}.fits"));
    let now = Instant::now();
    fits.write(&write_path).unwrap();
    let time = now.elapsed().as_millis();
    write_times.push(time);

    //(3) Progressbar
    progress_bar.set(index + 1);
    print!("\r{progress_bar}");
    std::io::stdout().flush().unwrap();
  }

  println!("Average read time: {}ms", read_times.iter().sum::<u128>() as usize / read_times.len());
  println!(
    "Average write time: {}ms",
    write_times.iter().sum::<u128>() as usize / write_times.len()
  );
}
