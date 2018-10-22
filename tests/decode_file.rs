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

//  ************************************************************
//! Test decoding image files with various QR Codes
//  ************************************************************

#![feature(test)]

extern crate image;
extern crate qrlogo_wasm;
extern crate test;

use std::fs;
use std::path::{Path, PathBuf};

use image::{DynamicImage, GenericImageView, Pixel};
use test::Bencher;

use qrlogo_wasm::qrdecode::decode_image;
use qrlogo_wasm::RGBAImage;

mod common;
use common::print_decoding_result;


//  ************************************************************
//  Test decoding image files and verify the the decoded text matches the expected
//  ************************************************************

#[test]
fn decode_file_firefox() {
    decode_file("qr_firefox.png", false);
}

#[test]
fn decode_file_firefox_aggressive() {
    decode_file("qr_firefox.png", true);
}

#[test]
fn decode_file_hamlet() {
    decode_file("hamlet.png", false);
}

#[test]
fn decode_file_hamlet_aggressive() {
    decode_file("hamlet.png", true);
}

#[test]
fn decode_file_youtube_aggressive() {
    decode_file("youtube.png", true);
}

#[test]
fn decode_file_facebook() {
    decode_file("facebook.png", false);
}

#[test]
fn decode_file_facebook_aggressive() {
    decode_file("facebook.png", true);
}

#[test]
fn decode_file_jungle_aggressive() {
    decode_file("jungle.png", true);
}

#[test]
fn decode_file_ninja_aggressive() {
    decode_file("ninja.png", true);
}

#[test]
fn decode_file_rain_aggressive() {
    decode_file("rain.png", true);
}

#[test]
fn decode_file_twitter() {
    decode_file("twitter.png", false);
}

#[test]
fn decode_file_twitter_aggressive() {
    decode_file("twitter.png", true);
}

#[test]
fn decode_file_ibm_aggressive() {
    decode_file("ibm.png", true);
}

#[test]
fn decode_file_ffox() {
    decode_file("ffox.png", false);
}

#[test]
fn decode_file_ffox_aggressive() {
    decode_file("ffox.png", true);
}

#[test]
fn decode_file_qr1() {
    decode_file("Qr-1.png", false);
}

#[test]
fn decode_file_qr1_aggressive() {
    decode_file("Qr-1.png", true);
}
#[test]
fn decode_file_qr2() {
    decode_file("Qr-2.png", false);
}

#[test]
fn decode_file_qr2_aggressive() {
    decode_file("Qr-2.png", true);
}
#[test]
fn decode_file_qr3() {
    decode_file("Qr-3.png", false);
}

#[test]
fn decode_file_qr3_aggressive() {
    decode_file("Qr-3.png", true);
}

#[test]
fn decode_file_qr4() {
    decode_file("Qr-4.png", false);
}

#[test]
fn decode_file_qr4_aggressive() {
    decode_file("Qr-4.png", true);
}

/*
 * TODO: Fix sequence of multiple modes in decoding
 *
#[test]
fn decode_file_qrcodever10() {
    decode_file("Qr-code-ver-10.png", false);
}

#[test]
fn decode_file_qrcodever10_aggressive() {
    decode_file("Qr-code-ver-10.png", true);
}
*/

#[test]
fn decode_file_qrcodever40() {
    decode_file("Qr-code-ver-40.png", false);
}

#[test]
fn decode_file_qrcodever40_aggressive() {
    decode_file("Qr-code-ver-40.png", true);
}

#[test]
fn decode_file_qrdroid2663() {
    decode_file("QR_Droid_2663.png", false);
}

#[test]
fn decode_file_qrdroid2663_aggressive() {
    decode_file("QR_Droid_2663.png", true);
}

#[test]
fn decode_file_qrcodedamaged_aggressive() {
    decode_file("QR_Code_Damaged.jpg", true);
}

#[test]
fn decode_file_caradqrcode() {
    decode_file("Car-ad-qr-code.jpg", false);
}

#[test]
fn decode_file_caradqrcode_aggressive() {
    decode_file("Car-ad-qr-code.jpg", true);
}

#[test]
fn decode_file_qrcodesbarcodeart9_aggressive() {
    decode_file("QR-Codes-Barcode-Art-9.jpg", true);
}

#[test]
fn decode_file_tyoerny_aggressive() {
    decode_file("2_150_150DPI_ty_oerny_08_2011.jpg", true);
}

#[test]
fn decode_file_bpcqrcode_aggressive() {
    decode_file("BPC-QR-Code-300x300.jpg", true);
}

#[test]
fn decode_file_qrlogo01() {
    decode_file("qr_logo_01.png", false);
}

#[test]
fn decode_file_qrlogo01_aggressive() {
    decode_file("qr_logo_01.png", true);
}

#[test]
fn decode_file_kaarpo() {
    decode_file("qr_kaarpo.png", false);
}

#[test]
fn decode_file_kaarpo_aggressive() {
    decode_file("qr_kaarpo.png", true);
}

#[test]
fn decode_file_kaarpo45() {
    decode_file("qr_kaarpo45.png", false);
}

#[test]
fn decode_file_kaarpo45_aggressive() {
    decode_file("qr_kaarpo45.png", true);
}


//  ************************************************************
//  Test trying to decoding image files not containing a QR Code and verify that it fails
//  ************************************************************

#[test]
fn decode_file_bite_fail() {
    decode_file_fail("bite-breakfast-cake-1166418.jpg", false);
}

#[test]
fn decode_file_bite_fail_aggressive() {
    decode_file_fail("bite-breakfast-cake-1166418.jpg", true);
}

#[test]
fn decode_file_tree_fail() {
    decode_file_fail("colorful-tree-logo-icon-31506022.jpg", false);
}

#[test]
fn decode_file_tree_fail_aggressive() {
    decode_file_fail("colorful-tree-logo-icon-31506022.jpg", true);
}


#[test]
fn decode_file_10logomotion_fail() {
    decode_file_fail("10.LogoMotion.jpg", false);
}

#[test]
fn decode_file_10logomotion_fail_aggressive() {
    decode_file_fail("10.LogoMotion.jpg", true);
}


//  ************************************************************
//  Benchmarks for decoding image files
//  ************************************************************

#[bench]
fn benchmark_decode_file_qrcodever10(b: &mut Bencher) {
    benchmark_file(b, "Qr-code-ver-10.png", false);
}

#[bench]
fn benchmark_decode_file_qrcodever10_aggressive(b: &mut Bencher) {
    benchmark_file(b, "Qr-code-ver-10.png", true);
}

#[bench]
fn benchmark_decode_file_qrcodever40(b: &mut Bencher) {
    benchmark_file(b, "Qr-code-ver-40.png", false);
}

#[bench]
fn benchmark_decode_file_qrcodever40_aggressive(b: &mut Bencher) {
    benchmark_file(b, "Qr-code-ver-40.png", true);
}

#[bench]
fn benchmark_decode_file_firefox(b: &mut Bencher) {
    benchmark_file(b, "qr_firefox.png", false);
}

#[bench]
fn benchmark_decode_file_firefox_aggressive(b: &mut Bencher) {
    benchmark_file(b, "qr_firefox.png", true);
}

#[bench]
fn benchmark_decode_file_hamlet(b: &mut Bencher) {
    benchmark_file(b, "hamlet.png", false);
}

#[bench]
fn benchmark_decode_file_hamlet_aggressive(b: &mut Bencher) {
    benchmark_file(b, "hamlet.png", true);
}


#[bench]
fn benchmark_decode_file_ninja_aggressive(b: &mut Bencher) {
    benchmark_file(b, "ninja.png", true);
}

#[bench]
fn benchmark_decode_file_10logomotion_fail_aggressive(b: &mut Bencher) {
    benchmark_file(b, "10.LogoMotion.jpg", true);
}


//  ************************************************************
/// Decode an image file and verify that the result matches the expected  
//  ************************************************************

fn decode_file(filename: &str, aggressive: bool) {
    let (img, expected_data) = load_img_file_with_expected(filename);
    let res = decode_image(&RGBAImageWrapper { img }, aggressive);
    match res.err {
        Some(err) => panic!(err),
        None => {
            print_decoding_result(&res);
            let res_data = res.data.unwrap();
            //println!("RES.DATA {:02X?}", res_data);
            //println!("EXPECTED DATA {:02X?}", expected_data);
            assert!(res_data == expected_data, "Decoded wrong data");
        }
    }
}


//  ************************************************************
/// Try to decode and image fila, and verify that this returns an error
//  ************************************************************

fn decode_file_fail(filename: &str, aggressive: bool) {
    let img = load_img_file(filename);
    let res = decode_image(&RGBAImageWrapper { img }, aggressive);
    assert!(res.err.is_some(), "Expected decoding to fail");
}


//  ************************************************************
/// Benchmark the decoding of an image file
/// (excluding loading and parsing the image file format itself)
//  ************************************************************

fn benchmark_file(b: &mut Bencher, filename: &str, aggressive: bool) {
    let img = load_img_file(filename);
    let wrapper = RGBAImageWrapper { img };
    b.iter({ || decode_image(&wrapper, aggressive) });
}


//  ************************************************************
/// Load an image file
//  ************************************************************

fn load_img_file(filename: &str) -> DynamicImage {
    let path = get_img_path(filename);
    load_img_path(&path)
}


//  ************************************************************
/// Load an image file as well as the file with the expected result
//  ************************************************************

fn load_img_file_with_expected(filename: &str) -> (DynamicImage, Vec<u8>) {
    let mut path = get_img_path(filename);
    let img = load_img_path(&path);
    path.set_extension("txt");
    let expected = load_expected_path(&path);
    (img, expected)
}


//  ************************************************************
/// Load a file containing the expected result of the associated image file
//  ************************************************************

fn load_expected_path(path: &Path) -> Vec<u8> {
    match fs::read(path.clone()) {
        Ok(d) => d,
        Err(e) => panic!("Failed to load data {:?}i {:?}", path, e),
    }
}


//  ************************************************************
/// Load an image file from the given path
//  ************************************************************

fn load_img_path(path: &Path) -> DynamicImage {
    match image::open(path.clone()) {
        Ok(i) => i,
        Err(e) => panic!("Failed to load image {:?}: {:?}", path, e),
    }
}


//  ************************************************************
/// Get the path where an image file with the given filename is found
//  ************************************************************

fn get_img_path(filename: &str) -> PathBuf {
    let mut img_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    img_path.push("www");
    img_path.push("testpics");
    img_path.push(filename);
    println!("get_img_path: file [{}] in path [{:?}]", filename, img_path);
    img_path
}


//  ************************************************************
/// A wrapper around a DynamicImage implementing the RGBAImage trait
//  ************************************************************

struct RGBAImageWrapper {
    img: DynamicImage,
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
