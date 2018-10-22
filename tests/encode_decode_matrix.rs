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
//! Test encoding and then decoding QR Codes
//  ************************************************************

extern crate qrlogo_wasm;

use qrlogo_wasm::logging;
use qrlogo_wasm::prng::Rng;
use qrlogo_wasm::qrdecode::decode_image;
use qrlogo_wasm::qrencode::{encode, Matrix};
use qrlogo_wasm::{qr, ErrorCorrectionLevel, Mode, RGBAImage};

mod common;
use common::print_decoding_result;


//  ************************************************************
//  Test encoding and decoding "perfect" QR Codes with no damages
//  ************************************************************

#[test]
fn encode_decode_matrix_good_anum_l() {
    matrix_test(Mode::AlphaNumeric, ErrorCorrectionLevel::L, 1, 1, 11, good_damage_iterator_new);
}

#[test]
fn encode_decode_matrix_good_anum_m() {
    matrix_test(Mode::AlphaNumeric, ErrorCorrectionLevel::M, 1, 1, 12, good_damage_iterator_new);
}

#[test]
fn encode_decode_matrix_good_anum_q() {
    matrix_test(Mode::AlphaNumeric, ErrorCorrectionLevel::Q, 1, 1, 13, good_damage_iterator_new);
}

#[test]
fn encode_decode_matrix_good_anum_h() {
    matrix_test(Mode::AlphaNumeric, ErrorCorrectionLevel::H, 1, 1, 14, good_damage_iterator_new);
}

#[test]
#[allow(non_snake_case)]
fn encode_decode_matrix_good_num__l() {
    matrix_test(Mode::Numeric, ErrorCorrectionLevel::L, 1, 1, 21, good_damage_iterator_new);
}

#[test]
#[allow(non_snake_case)]
fn encode_decode_matrix_good_num__m() {
    matrix_test(Mode::Numeric, ErrorCorrectionLevel::M, 1, 1, 22, good_damage_iterator_new);
}

#[test]
#[allow(non_snake_case)]
fn encode_decode_matrix_good_num__q() {
    matrix_test(Mode::Numeric, ErrorCorrectionLevel::Q, 1, 1, 23, good_damage_iterator_new);
}

#[test]
#[allow(non_snake_case)]
fn encode_decode_matrix_good_num__h() {
    matrix_test(Mode::Numeric, ErrorCorrectionLevel::H, 1, 0, 24, good_damage_iterator_new)
}

#[test]
fn encode_decode_matrix_good_8bit_l() {
    matrix_test(Mode::EightBit, ErrorCorrectionLevel::L, 4, 4, 31, good_damage_iterator_new);
}

#[test]
fn encode_decode_matrix_good_8bit_m() {
    matrix_test(Mode::EightBit, ErrorCorrectionLevel::M, 4, 4, 32, good_damage_iterator_new);
}

#[test]
fn encode_decode_matrix_good_8bit_q() {
    matrix_test(Mode::EightBit, ErrorCorrectionLevel::Q, 4, 4, 33, good_damage_iterator_new);
}

#[test]
fn encode_decode_matrix_good_8bit_h() {
    matrix_test(Mode::EightBit, ErrorCorrectionLevel::H, 4, 4, 34, good_damage_iterator_new);
}


//  ************************************************************
//  Test decoding QR Codes, where damage has been introduced in the SouthEast corner
//  ************************************************************

#[test]
fn encode_decode_matrix_se_sqare_8bit_l() {
    matrix_test(Mode::EightBit, ErrorCorrectionLevel::L, 1, 1, 44, se_square_damage_iterator_new);
}

#[test]
fn encode_decode_matrix_se_sqare_8bit_m() {
    matrix_test(Mode::EightBit, ErrorCorrectionLevel::M, 1, 1, 44, se_square_damage_iterator_new);
}

#[test]
fn encode_decode_matrix_se_sqare_8bit_q() {
    matrix_test(Mode::EightBit, ErrorCorrectionLevel::Q, 1, 1, 44, se_square_damage_iterator_new);
}

#[test]
fn encode_decode_matrix_se_sqare_8bit_h() {
    matrix_test(Mode::EightBit, ErrorCorrectionLevel::H, 1, 1, 44, se_square_damage_iterator_new);
}


//  ************************************************************
//  Test decoding QR codes with random introduced damages
//  ************************************************************

#[test]
fn encode_decode_matrix_random_8bit_l() {
    matrix_test(Mode::EightBit, ErrorCorrectionLevel::L, 1, 1, 54, random_damage_iterator_new);
}

#[test]
fn encode_decode_matrix_random_8bit_m() {
    matrix_test(Mode::EightBit, ErrorCorrectionLevel::M, 1, 1, 54, random_damage_iterator_new);
}

#[test]
fn encode_decode_matrix_random_8bit_q() {
    matrix_test(Mode::EightBit, ErrorCorrectionLevel::Q, 1, 1, 54, random_damage_iterator_new);
}

#[test]
fn encode_decode_matrix_random_8bit_h() {
    matrix_test(Mode::EightBit, ErrorCorrectionLevel::H, 1, 1, 54, random_damage_iterator_new);
}


//  ************************************************************
/// Trait for iterator introducing "damage" to a QR Code
//
//  With this trait, we can have the same main test function `matrix_test`
//  but iterate over different kind of inflicted damages
//  ************************************************************

trait DamageIterator {
    fn inflict_damage(&mut self, &mut Matrix) -> bool;
}


//  ************************************************************
//  A `DamageIterator` which just does nothing once (ie. leaving the encoded QR Code alone)
//  ************************************************************

struct GoodDamageIterator {
    done: bool,
}

impl DamageIterator for GoodDamageIterator {
    fn inflict_damage(&mut self, _matrix: &mut Matrix) -> bool {
        if self.done {
            return false;
        }
        self.done = true;
        true
    }
}

fn good_damage_iterator_new(_dimension: usize, _ec: ErrorCorrectionLevel) -> impl DamageIterator {
    GoodDamageIterator { done: false }
}


//  ************************************************************
//  A `DamageIterator` which inflicts damage in a square in the South East corner of the QR Code
//  ************************************************************

struct SeSquareDamageIterator {
    dim: usize,
    max: usize,
    idx: usize,
}

impl DamageIterator for SeSquareDamageIterator {
    fn inflict_damage(&mut self, matrix: &mut Matrix) -> bool {
        if self.idx > self.max {
            return false;
        }
        println!("\n========== INFLICT SE_SQUARE DAMAGE ==========> idx={}", self.idx);
        let color = if self.idx % 2 == 1 { 0xFF } else { 0x00 };
        for x in self.dim - self.idx..self.dim {
            matrix.set(x, self.dim - self.idx, color);
        }
        for y in self.dim - self.idx - 1..self.dim {
            matrix.set(self.dim - self.idx, y, color);
        }
        self.idx += 1;
        true
    }
}

fn se_square_damage_iterator_new(dim: usize, ec: ErrorCorrectionLevel) -> impl DamageIterator {
    let max = match ec {
        ErrorCorrectionLevel::L => dim / 5,
        ErrorCorrectionLevel::M => dim / 5,
        ErrorCorrectionLevel::Q => dim / 4,
        ErrorCorrectionLevel::H => dim / 3,
    };
    println!("\n ======= ========== SQUARE DAMAGE ITERATOR ===========> dim={}, ec={:?}, max={}", dim, ec, max);
    SeSquareDamageIterator { dim, max, idx: 1 }
}


//  ************************************************************
//  A `DamageIterator` which inflicts damage at random points
//  ************************************************************

struct RandomDamageIterator {
    rng: Rng,
    dim: usize,
    max: usize,
    idx: usize,
}

impl DamageIterator for RandomDamageIterator {
    fn inflict_damage(&mut self, matrix: &mut Matrix) -> bool {
        if self.idx > self.max {
            return false;
        }
        self.idx += 1;
        let color = if self.idx % 2 == 1 { 0xFF } else { 0x00 };
        let x = self.rng.get_usize_clamped(0, self.dim - 1);
        let y = self.rng.get_usize_clamped(0, self.dim - 1);
        println!(
            "\n========== INFLICT RANDOM DAMAGE ==========> max={} idx={} x={} y={} color={}",
            self.max, self.idx, x, y, color
        );
        matrix.set(x, y, color);
        true
    }
}

fn random_damage_iterator_new(dim: usize, ec: ErrorCorrectionLevel) -> impl DamageIterator {
    let max = 2 + dim * dim / 123;
    println!("\n======= ========== RANDOM DAMAGE ITERATOR ==========> dim={}, ec={:?}, max={}", dim, ec, max);
    RandomDamageIterator { rng: Rng::new(8 * dim as u32 + ec as u32), dim, max, idx: 0 }
}


//  ************************************************************
//  Test QR encode then decode for all versions; possibly introducing damage to the QR Code along the way
//  ************************************************************


fn matrix_test<D, I>(mode: Mode, ec: ErrorCorrectionLevel, n_fixed: usize, n_random: usize, seed: u32, damage_iterator_new: D)
where
    I: DamageIterator,
    D: Fn(usize, ErrorCorrectionLevel) -> I,
{
    logging::set_loglevel(1);
    let mut rng = Rng::new(seed);
    for version in qr::VERSION_MIN..=qr::VERSION_MAX {
        let capacity = qr::data_capacity(version, mode, ec) as usize;
        let mut lengths: Vec<usize> = vec![capacity, capacity - 1, 1, 2].iter().take(n_fixed).map(|x| *x).collect();
        for _ in 0..n_random {
            lengths.push(rng.get_usize_clamped(0, capacity));
        }
        for len in lengths {
            println!(
                "\n\n========== ========== MATRIX TEST ==========> mode={:?} ec={:?} version={} capacity={} len={}",
                mode, ec, version, capacity, len
            );
            let mut data = Vec::with_capacity(len);
            for _ in 0..len {
                data.push(rng.get_u8_with_mode(mode));
            }

            let mut matrix = encode(&data, version, mode, ec);
            let mut damage_iterator = damage_iterator_new(matrix.get_dim(), ec);
            while damage_iterator.inflict_damage(&mut matrix) {
                let img = MatrixWrapper::new(&matrix);
                let aggressive = true; // TODO
                let res = decode_image(&img, aggressive);
                print_decoding_result(&res);
                assert!(res.err.is_none(), "decode_image failed: {}", res.err.unwrap());
                let decoded_data = res.data.unwrap();
                assert!(decoded_data == data, "decoded to wrong data: expected {:X?}; got {:X?}", data, decoded_data);
            }
        }
    }
}


//  ************************************************************
/// A wrapper around a Matrix implementing the RGBAImage trait
//  ************************************************************

pub struct MatrixWrapper<'a> {
    matrix: &'a Matrix,
}

impl<'a> MatrixWrapper<'a> {
    pub fn new(matrix: &'a Matrix) -> Self {
        MatrixWrapper { matrix }
    }
}

impl<'a> RGBAImage for MatrixWrapper<'a> {
    fn width(&self) -> usize {
        self.matrix.get_dim() as usize
    }
    fn height(&self) -> usize {
        self.matrix.get_dim() as usize
    }
    fn get(&self, x: usize, y: usize) -> (u8, u8, u8, u8) {
        if self.matrix.get_selected(x, y) {
            (0x10, 0x10, 0x10, 0xFF)
        } else {
            (0xF0, 0xF0, 0xF0, 0xFF)
        }
    }
}
