/*  ************************************************************

    QR-Logo: http://qrlogo.kaarposoft.dk

    Copyright (C) 2018 Henrik Kaare Poulsen

    Licensed under the Apache License, Version 2.0 (the "License");
    you may not use this file except in compliance with the License.
    You may obtain a copy of the License at

     http://www.apache.org/licenses/LICENSE-2.0

    Unless required by applicable law or agreed to in writing, software
    distributed under the License is distributed on an "AS IS" BASIS,
    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
    See the License for the specific language governing permissions and
    limitations under the License.

    ************************************************************ */

extern crate clap;
extern crate image;
extern crate qrlogo_wasm;

use clap::{App, Arg, Error, ErrorKind};
use qrlogo_wasm::logging;
use qrlogo_wasm::qrencode::{encode, Matrix};
use qrlogo_wasm::{qr, ErrorCorrectionLevel, Mode};
use std::io::{self, Write};


//  ************************************************************

const ABOUT: &str = "Encode data into a QR Code

The <DATA> provided is encoded into a QR Code.

By default the QR Code is output as text to stdout.
By specifying --ansi the output to stdout uses ANSI escape codes on stdout.
By specifying --file <FILE> the output will be written to the <FILE> specified.
The file type (eg .png, or .jpg) determines the format of the <FILE>.
";


//  ************************************************************

const ARG_DEBUG: &str = "DEBUG";
const ARG_MODE: &str = "MODE";
const ARG_EC: &str = "EC";
const ARG_VERSION: &str = "VERSION";
const ARG_ANSI: &str = "ANSI";
const ARG_PPM: &str = "PPM";
const ARG_FILE: &str = "FILE";
const ARG_DATA: &str = "DATA";


//  ************************************************************

fn main() {
    let matches = App::new("QR Code encoder")
        .version("0.1")
        .author("Henrik <henrik@kaarposoft.dk>")
        .about(ABOUT)
        .arg(
            Arg::with_name(ARG_DEBUG)
                .short("d")
                .long("debug")
                .help("Specify -d/--debug one or more times to increase debug level")
                .multiple(true),
        ).arg(
            Arg::with_name(ARG_VERSION)
                .short("v")
                .long("version")
                .help("QR Code version [possible values: 1-40]")
                .value_name(ARG_VERSION)
                .takes_value(true),
        ).arg(
            Arg::with_name(ARG_MODE)
                .short("m")
                .long("mode")
                .help("Encoding mode (8bit, Alphanumeric, Numeric)")
                .value_name(ARG_MODE)
                .possible_values(&["8", "A", "a", "N", "n"]),
        ).arg(
            Arg::with_name(ARG_EC)
                .short("e")
                .long("error-correction-level")
                .help("Error correction level")
                .value_name(ARG_EC)
                .possible_values(&["L", "l", "M", "m", "Q", "q", "H", "h"]),
        ).arg(Arg::with_name(ARG_ANSI).short("a").long("ansi").help("Output ansi control codes (when encoding to stdout)"))
        .arg(
            Arg::with_name(ARG_PPM)
                .short("p")
                .long("pixels-per-module")
                .help("Image pixels per QR Code module (when encoding to file) [possible values: 1-16]")
                .takes_value(true)
                .requires(ARG_FILE),
        ).arg(
            Arg::with_name(ARG_FILE)
                .short("f")
                .long("file")
                .help("File (path) to write QR Code image to")
                .value_name(ARG_FILE)
                .takes_value(true)
                .conflicts_with(ARG_ANSI),
        ).arg(Arg::with_name(ARG_DATA).value_name("DATA").help("Data to be encoded").required(true))
        .get_matches();

    let debug_level = matches.occurrences_of(ARG_DEBUG);
    logging::set_loglevel(debug_level as usize);

    let data = matches.value_of(ARG_DATA).unwrap();

    let mode = match matches.value_of(ARG_MODE) {
        None => Mode::EightBit,
        Some("8") => Mode::EightBit,
        Some("A") | Some("a") => Mode::AlphaNumeric,
        Some("N") | Some("n") => Mode::Numeric,
        Some(e) => panic!("unknown  mode {}", e),
    };

    let ec = match matches.value_of(ARG_EC) {
        None => ErrorCorrectionLevel::M,
        Some("L") | Some("l") => ErrorCorrectionLevel::L,
        Some("M") | Some("m") => ErrorCorrectionLevel::M,
        Some("Q") | Some("q") => ErrorCorrectionLevel::Q,
        Some("H") | Some("h") => ErrorCorrectionLevel::H,
        Some(e) => panic!("unknown error correction level {}", e),
    };

    let version = match matches.value_of(ARG_VERSION) {
        Some(v) => match v.parse::<u8>() {
            Err(_) => invalid_exit("invalid version [possible values 1-40]"),
            Ok(v) => {
                if v > 40 {
                    invalid_exit("largest possible version is 40");
                }
                v
            }
        },
        None => match qr::version_from_length(data.len(), mode, ec) {
            None => invalid_exit("data too long for mode/ec"),
            Some(v) => v,
        },
    };

    let ppm = match matches.value_of(ARG_PPM) {
        Some(p) => match p.parse::<usize>() {
            Err(_) => invalid_exit("invalid pixels-per-module [possible values 1-16]"),
            Ok(p) => {
                if p > 16 {
                    invalid_exit("largest possible pixels-per-module is 16");
                }
                p
            }
        },
        None => 4,
    };


    let matrix = encode(data.as_bytes(), version, mode, ec);
    let n_modules = qr::n_modules_from_version(version);

    let ansi = matches.is_present(ARG_ANSI);
    let file = matches.value_of(ARG_FILE);

    match file {
        None => {
            if ansi {
                write_to_stdout_ansi(n_modules, &matrix);
            } else {
                write_to_stdout(n_modules, &matrix);
            }
        }
        Some(f) => {
            write_to_path(n_modules, &matrix, ppm, f);
        }
    }
}


//  ************************************************************

fn write_to_stdout(n_modules: usize, matrix: &Matrix) {
    let n = n_modules;
    for y in 0..n {
        let mut s = String::with_capacity(n + 1);
        for x in 0..n {
            if matrix.get_selected(x, y) {
                s.push('@');
            } else {
                s.push('.');
            }
        }
        s.push('\n');
        io::stdout().write(s.as_bytes()).unwrap();
    }
}


//  ************************************************************

fn write_to_stdout_ansi(n_modules: usize, matrix: &Matrix) {
    let n = n_modules;
    let border = 4;
    let mut border_lines = String::new();
    for _ in 0..border / 2 {
        border_lines.push('\n');
    }
    io::stdout().write(border_lines.as_bytes()).unwrap();

    for y in 0..n {
        let mut s = String::new();
        for _ in 0..border {
            s.push(' ');
        }
        for x in 0..n {
            if matrix.get_selected(x, y) {
                s.push_str("\x1B[40m  ");
            } else {
                s.push_str("\x1B[107m  ");
            }
        }
        s.push_str("\x1B[0m\n");
        io::stdout().write(s.as_bytes()).unwrap();
    }
    io::stdout().write(border_lines.as_bytes()).unwrap();
}


//  ************************************************************

fn write_to_path(n_modules: usize, matrix: &Matrix, ppm: usize, path: &str) {
    let n = n_modules;
    let dark = 48;
    let light = 240;
    let border = 4;
    let mut img_data = Vec::with_capacity((n + 2 * border) * ppm * (n + 2 * border) * ppm);
    for _y in 0..ppm * border {
        for _x in 0..ppm * (n + 2 * border) {
            img_data.push(light);
        }
    }
    for y in 0..n {
        for _ in 1..=ppm {
            for _x in 0..ppm * border {
                img_data.push(light);
            }
            for x in 0..n {
                let gray = if matrix.get_selected(x, y) { dark } else { light };
                for _ in 1..=ppm {
                    img_data.push(gray);
                }
            }
            for _x in 0..ppm * border {
                img_data.push(light);
            }
        }
    }
    for _y in 0..ppm * border {
        for _x in 0..ppm * (n + 2 * border) {
            img_data.push(light);
        }
    }
    image::save_buffer(path, &img_data, (ppm * (n + 2 * border)) as u32, (ppm * (n + 2 * border)) as u32, image::Gray(8)).unwrap()
}


//  ************************************************************

fn invalid_exit(msg: &str) -> ! {
    let err = Error::with_description(msg, ErrorKind::InvalidValue);
    err.exit()
}
