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
//! Fallback bindings for Web APIs
//  ************************************************************
//!
//! We would really like to use
//! [web-sys](https://github.com/rustwasm/wasm-bindgen/tree/master/crates/web-sys),
//! but it is not yet stable enough
//! (as of version [0.2.19](https://github.com/rustwasm/wasm-bindgen/tree/0.2.19/crates/web-sys))
//!
//  ************************************************************


extern crate wasm_bindgen;
use wasm_bindgen::prelude::*;


#[wasm_bindgen]
extern "C" {

    //  ************************************************************
    //  ImageData
    //  ************************************************************
    ///
    /// The `ImageData` interface represents the underlying pixel data of an area of a `canvas` element.
    ///
    /// * <https://developer.mozilla.org/en-US/docs/Web/API/ImageData>
    /// * <https://html.spec.whatwg.org/multipage/canvas.html#imagedata>
    ///
    //  ************************************************************
    pub type ImageData;

    #[wasm_bindgen(constructor)]
    pub fn new(arr: &Uint8ClampedArray, width: u32, height: u32) -> ImageData;
}


#[wasm_bindgen]
extern "C" {

    //  ************************************************************
    //  Uint8ClampedArray
    //  ************************************************************
    ///
    /// The `Uint8ClampedArray` typed array represents an array of 8-bit unsigned integers clamped to 0-255
    ///
    /// * <https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Uint8ClampedArray>
    /// * <https://www.ecma-international.org/ecma-262/6.0/#table-49>
    ///
    //  ************************************************************
    pub type Uint8ClampedArray;

    #[wasm_bindgen(constructor)]
    pub fn new(arr: &[u8]) -> Uint8ClampedArray;
}


#[wasm_bindgen]
extern "C" {

    //  ************************************************************
    //  HTMLCanvasElement
    //  ************************************************************
    ///
    /// The `HTMLCanvasElement` interface provides properties and methods
    /// for manipulating the layout and presentation of canvas elements.
    ///
    /// * <https://developer.mozilla.org/en-US/docs/Web/API/HTMLCanvasElement>
    /// * <https://html.spec.whatwg.org/multipage/canvas.html#htmlcanvaselement>
    ///
    //  ************************************************************
    pub type HTMLCanvasElement;

    #[wasm_bindgen(method, getter)]
    pub fn width(this: &HTMLCanvasElement) -> u32;

    #[wasm_bindgen(method, setter)]
    pub fn set_width(this: &HTMLCanvasElement, width: u32);

    #[wasm_bindgen(method, getter)]
    pub fn height(this: &HTMLCanvasElement) -> u32;

    #[wasm_bindgen(method, setter)]
    pub fn set_height(this: &HTMLCanvasElement, height: u32);
}


#[wasm_bindgen]
extern "C" {

    //  ************************************************************
    //  CanvasRenderingContext2D
    //  ************************************************************
    ///
    /// The `CanvasRenderingContext2D` interface is used for drawing rectangles, text, images and other objects
    /// onto the `canvas` element.
    ///
    /// * <https://developer.mozilla.org/en-US/docs/Web/API/CanvasRenderingContext2D>
    /// * <https://html.spec.whatwg.org/multipage/canvas.html#2dcontext>
    ///
    //  ************************************************************
    pub type CanvasRenderingContext2D;

    #[wasm_bindgen(method, getter)]
    pub fn canvas(this: &CanvasRenderingContext2D) -> HTMLCanvasElement;

    #[wasm_bindgen(method, js_name = putImageData)]
    pub fn put_image_data(this: &CanvasRenderingContext2D, image_data: &ImageData, x: u32, y: u32);

    #[wasm_bindgen(method)]
    pub fn save(this: &CanvasRenderingContext2D);

    #[wasm_bindgen(method)]
    pub fn restore(this: &CanvasRenderingContext2D);

    #[wasm_bindgen(method)]
    pub fn scale(this: &CanvasRenderingContext2D, x: f64, y: f64);

    #[wasm_bindgen(method, setter, js_name = set_fillStyle)]
    pub fn set_fill_style_with_str(this: &CanvasRenderingContext2D, style: &str);

    #[wasm_bindgen(method, js_name = fillRect)]
    pub fn fill_rect(this: &CanvasRenderingContext2D, x: f64, y: f64, w: f64, h: f64);
}


//  ************************************************************
//  console
//  ************************************************************
///
/// The `console` object provides access to the browser's debugging console.
///
/// * <https://developer.mozilla.org/en-US/docs/Web/API/Console>
/// * <https://console.spec.whatwg.org/>
///
//  ************************************************************

pub mod console {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    extern "C" {

        #[wasm_bindgen(js_namespace = console, js_name = error)]
        pub fn error_with_str(s: &str);

        #[wasm_bindgen(js_namespace = console, js_name = log)]
        pub fn log_with_str(s: &str);

        #[wasm_bindgen(js_namespace = console, js_name = warn)]
        pub fn warn_with_str(s: &str);
    }
}
