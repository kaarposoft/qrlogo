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

use std::io;

use clap::{App, Arg, Error, ErrorKind};
use image::{GenericImageView, Pixel};
use qrlogo_wasm::logging;
use qrlogo_wasm::qrdecode::decode_image;
use qrlogo_wasm::RGBAImage;


//  ************************************************************

const ABOUT: &str = "Decode data from QR Code image

Decode the image data in <FILE> assuming it is a QR Code
";


//  ************************************************************

const ARG_DEBUG: &str = "DEBUG";
const ARG_AGGRESSIVE: &str = "AGGRESSIVE";
const ARG_FILE: &str = "FILE";


//  ************************************************************

fn main() -> Result<(), String> {
    let matches = App::new("QR Code decoder")
        .version("0.1")
        .author("Henrik <henrik@kaarposoft.dk>")
        .about(ABOUT)
        .arg(
            Arg::with_name(ARG_DEBUG)
                .short("d")
                .long("debug")
                .help("Specify -d/--debug one or more times to increase debug level")
                .multiple(true),
        ).arg(Arg::with_name(ARG_AGGRESSIVE).short("a").long("aggressive").help("Try a little harder to decode an image"))
        .arg(Arg::with_name(ARG_FILE).help("File (path) to read QR Code image from").value_name(ARG_FILE).required(true))
        .get_matches();

    let debug_level = matches.occurrences_of(ARG_DEBUG);
    logging::set_loglevel(debug_level as usize);

    let aggressive = matches.is_present(ARG_AGGRESSIVE);

    let file = matches.value_of(ARG_FILE).unwrap();

    let img = match image::open(file) {
        Ok(i) => i,
        Err(e) => invalid_exit(&format!("Failed to load image {}: {:?}", file, e)),
    };

    let res = decode_image(&RGBAImageWrapper { img }, aggressive);
    res.write(&mut io::stdout());
    match res.err {
        Some(err) => Err(err),
        None => Ok(()),
    }
}


//  ************************************************************

struct RGBAImageWrapper {
    img: image::DynamicImage,
}

impl RGBAImage for RGBAImageWrapper {
    fn width(&self) -> usize {
        self.img.width() as usize
    }
    fn height(&self) -> usize {
        self.img.height() as usize
    }
    fn get(&self, x: usize, y: usize) -> (u8, u8, u8, u8) {
        let pix = self.img.get_pixel(x as u32, y as u32);
        let rgba = pix.to_rgba();
        (rgba[0], rgba[1], rgba[2], rgba[3])
    }
}


//  ************************************************************

fn invalid_exit(msg: &str) -> ! {
    let err = Error::with_description(msg, ErrorKind::InvalidValue);
    err.exit()
}
