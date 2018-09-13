/*  ************************************************************

    QR-Logo: http://qrlogo.kaarposoft.dk

    Copyright (C) 2011-2018 Henrik Kaare Poulsen

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


//  ************************************************************
//! Web Assembly (wasm) interface for QR-Logo
//  ************************************************************

extern crate wasm_bindgen;
extern crate web_sys;

use wasm_bindgen::prelude::*;
use web_sys::CanvasRenderingContext2D;

#[macro_use]
pub mod logging;

pub mod qr;
pub mod qrencode;
pub mod reedsolomon;


//  ************************************************************
/// Mode (Numeric, Alpha Numeric, 8 bit) as defined by ISO 18004
//  ************************************************************
///
/// Defines how text is encoded into the QR code
///
/// # Note
///
/// We only implement Numeric, Alpha Numeric, and 8 bit.
/// ISO 18004 defines other modes which are not implemented

#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Numeric = 1,
    AlphaNumeric = 2,
    EightBit = 4,
}


//  ************************************************************
/// Error Correction Level (L/M/Q/H) as defined by ISO 18004
//  ************************************************************

#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorCorrectionLevel {
    /// ~ 7% error correction capability
    L = 1,
    /// ~ 15% error correction capability
    M = 0,
    /// ~ 25% error correction capability
    Q = 3,
    /// ~ 30% error correction capability
    H = 2,
}


//  ************************************************************
/// Set logging level
//  ************************************************************

#[wasm_bindgen]
pub fn set_loglevel(lvl: usize) {
    logging::set_loglevel(lvl);
}


//  ************************************************************
/// Encode text to canvas
//  ************************************************************

#[wasm_bindgen]
pub fn encode_to_canvas(
    txt: &str,
    version: u8,
    mode: Mode,
    ec: ErrorCorrectionLevel,
    ctx: &CanvasRenderingContext2D,
    bg_color_str: &str,
    module_color_str: &str,
    pix_per_module: f64,
) {
    let matrix = qrencode::encode(txt[..].as_bytes(), version, mode, ec);
    qrencode::onto_context(&matrix, ctx, bg_color_str, module_color_str, pix_per_module);
}


//  ************************************************************
/// Get QR code `version` from text length
//  ************************************************************

#[wasm_bindgen]
pub fn version_from_length(len: usize, mode: Mode, ec: ErrorCorrectionLevel) -> Option<u8> {
    qr::version_from_length(len, mode, ec)
}
