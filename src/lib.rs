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

#![feature(tool_lints)]
#![allow(clippy::unnecessary_mut_passed)] // TODO: should we remove the mutability of references?

extern crate js_sys;
extern crate wasm_bindgen;

use wasm_bindgen::prelude::*;
use js_sys::{Array, Object, Reflect};

#[macro_use]
pub mod logging;
pub mod prng;
pub mod qr;
pub mod qrdecode;
pub mod qrencode;
pub mod reedsolomon;
pub mod web_sys_fallback;
use web_sys_fallback::{CanvasRenderingContext2D, ImageData};

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
#[allow(clippy::too_many_arguments)]
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


//  ************************************************************
/// Decode text in QR Code
//  ************************************************************

#[wasm_bindgen]
pub fn decode_from_image_data(image_data: &ImageData, aggressive: bool) -> Object {
    let result_in = qrdecode::decode_image(image_data, aggressive);
    let mut result_out: JsValue = Object::new().into();
    set_optional(&mut result_out, "err", result_in.err);
    set_optional(&mut result_out, "data", result_in.data.map(|d| String::from_utf8_lossy(&d[..]).into_owned()));
    set_optional(&mut result_out, "mode", result_in.mode.map(|m| m as u8));
    set_optional(&mut result_out, "version", result_in.version);
    set_optional(&mut result_out, "mask", result_in.mask);
    set_optional(&mut result_out, "ec", result_in.ec.map(|e| e as u8));
    set_value(&mut result_out, "functional_grade", result_in.functional_grade);
    set_value(&mut result_out, "decoding_grade", result_in.decoding_grade);
    set_array_f64(&mut result_out, "finder_grades", &result_in.finder_grades);
    set_array_f64(&mut result_out, "timing_grades", &result_in.timing_grades);
    set_value(&mut result_out, "alignment_grade", result_in.alignment_grade);
    set_array_f64(&mut result_out, "version_info_grades", &result_in.version_info_grades);
    set_array_f64(&mut result_out, "format_info_grades", &result_in.format_info_grades);
    result_out.into()
}


//  ************************************************************
/// Helper function to assign a value to a field in a JsValue object
//  ************************************************************

fn set_value<F: Into<JsValue>>(object: &mut JsValue, field: &str, value: F) {
    Reflect::set(object, &JsValue::from(field), &value.into());
}


//  ************************************************************
/// Helper function to assign an optional value to a field in a JsValue object, if the optional value is not None
//  ************************************************************

fn set_optional<F: Into<JsValue>>(object: &mut JsValue, field: &str, value: Option<F>) {
    if let Some(v) = value {
        Reflect::set(object, &JsValue::from(field), &v.into());
    }
}


//  ************************************************************
/// Helper function to assign a slice of f64 values to a field in a JsValue object
//  ************************************************************

fn set_array_f64(object: &mut JsValue, field: &str, values: &[f64]) {
    let a = Array::new();
    for v in values.iter() {
        a.push(&JsValue::from(*v));
    }
    Reflect::set(object, &JsValue::from(field), &a.into());
}

//  ************************************************************
/// Generic interface to RGBA images
//  ************************************************************

pub trait RGBAImage {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn get(&self, x: usize, y: usize) -> (u8, u8, u8, u8);
}

impl RGBAImage for ImageData {
    fn width(&self) -> usize {
        self.width() as usize
    }
    fn height(&self) -> usize {
        self.height() as usize
    }
    fn get(&self, x: usize, y: usize) -> (u8, u8, u8, u8) {
        let d = self.data();
        let i = (4 * (x as u32) + 4 * self.width() * (y as u32)) as u32;
        (d.get(i), d.get(i + 1), d.get(i + 2), d.get(i + 3))
    }
}
