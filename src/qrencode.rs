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
//! Encode text to QR code
//  ************************************************************


use super::logging;
use super::qr;
use super::qr::BitSeq;
use super::reedsolomon::ReedSolomonEncoder;
use super::web_sys_fallback::CanvasRenderingContext2D;
use super::{ErrorCorrectionLevel, Mode};


//  ************************************************************
/// Encode the given `matrix` onto the given browser 2D rendering context
//  ************************************************************

pub fn onto_context(
    matrix: &Matrix,
    ctx: &CanvasRenderingContext2D,
    bg_color_str: &str,
    module_color_str: &str,
    pix_per_module: f64,
) {
    log!("onto_context: begin transfer qr bitmap onto context");
    let n = matrix.get_dim();
    let dim = f64::ceil(pix_per_module * ((2 * qr::QUIET_ZONE + n) as f64)) as u32;
    let canvas = ctx.canvas(); //.unwrap(); // TODO
    canvas.set_height(dim);
    canvas.set_width(dim);
    ctx.save();
    ctx.set_fill_style_with_str(bg_color_str);
    ctx.fill_rect(0.0, 0.0, canvas.width().into(), canvas.height().into());
    // TODO: hanle scale errors
    ctx.scale(pix_per_module, pix_per_module);
    ctx.set_fill_style_with_str(module_color_str);
    for x in 0..n {
        for y in 0..n {
            if matrix.get_selected(x, y) {
                ctx.fill_rect((qr::QUIET_ZONE + x) as f64, (qr::QUIET_ZONE + y) as f64, 1.0, 1.0);
            }
        }
    }
    ctx.restore();
    log!("onto_context: done transfer qr bitmap onto context");
}


/* note
 * (x,y) = (row, column)  -  as usual
 * (i,j) = (column, row)  -  as in ISO std
*/


//  ************************************************************
/// Encode `text` (with `Mode` and `ErrorCorrectionLevel` to a `Matrix`
//  ************************************************************

pub fn encode(text: &[u8], version: u8, mode: Mode, ec: ErrorCorrectionLevel) -> Matrix {
    log!("encode: begin encoding qr code");
    debug!("encode: text.len={} version={} mode={:?} ec={:?}", text.len(), version, mode, ec);
    let n_modules = qr::n_modules_from_version(version);
    let n_codewords = qr::n_codewords(version) as usize;
    let n_ec_codewords = qr::n_ec_codewords(version, ec) as usize;
    let n_data_codewords = n_codewords - n_ec_codewords as usize;
    let data_capacity = qr::data_capacity(version, mode, ec);
    debug!(
        "encode: n_modules={} n_codewords={} n_ec_codewords={} n_data_codewords={} data_capacity={}",
        n_modules, n_codewords, n_ec_codewords, n_data_codewords, data_capacity,
    );
    let mut matrix = Matrix::new(n_modules);

    // Finder
    let n7 = n_modules - 7;
    set_finder_pattern(&mut matrix, 0, 0);
    set_finder_pattern(&mut matrix, n7, 0);
    set_finder_pattern(&mut matrix, 0, n7);

    // Timing
    set_timing_patterns(&mut matrix, n_modules);

    if version >= 7 {
        set_version(&mut matrix, version, n_modules);
    }

    // Alignment
    set_alignment_patterns(&mut matrix, version);

    // Masks
    let mask = create_mask_patterns();

    // Format
    set_format(&mut matrix, n_modules, ec);

    // Text
    let mut text_bits = BitSeq::new(n_data_codewords);
    text_bits.append_bits(mode as u16, 4);
    let n_count_bits = qr::n_count_bits(version, mode) as usize;
    let count = text.len() as u16;
    text_bits.append_bits(count as u16, n_count_bits);
    match mode {
        Mode::Numeric => encode_numeric(&mut text_bits, text),
        Mode::AlphaNumeric => encode_alpha_numeric(&mut text_bits, text),
        Mode::EightBit => encode_eight_bit(&mut text_bits, text),
    };
    set_padding(&mut text_bits, version, ec);

    let text_bytes = text_bits.into_bytes();
    let data_bytes = add_error_correction(text_bytes, version, ec);
    debug!("encode: data_bytes: len={} {:X?}", data_bytes.len(), data_bytes);
    let data_bits = BitSeq::from(data_bytes);

    set_data_snaked(&mut matrix, &mask, &data_bits, version);
    let p = best_penalty(&matrix, n_modules);
    matrix.select(p);
    log!("encode: done encoding qr code");
    matrix.select(0);
    matrix
}

//  ************************************************************
fn set_finder_pattern(matrix: &mut Matrix, x: usize, y: usize) {
    // Outer 7x7 black boundary
    for i in 0..=5 {
        matrix.set_all(x + i, y);
        matrix.set_all(x + 6, y + i);
        matrix.set_all(x + 6 - i, y + 6);
        matrix.set_all(x, y + 6 - i);
    }
    // Inner 3*3 black box
    for i in 2..=4 {
        for j in 2..=4 {
            matrix.set_all(x + i, y + j);
        }
    }
}

//  ************************************************************
fn set_timing_patterns(matrix: &mut Matrix, n_modules: usize) {
    for i in 8..n_modules - 8 {
        if i % 2 == 0 {
            matrix.set_all(i, 6);
            matrix.set_all(6, i);
        }
    }
}

//  ************************************************************
fn set_version(matrix: &mut Matrix, version: u8, n_modules: usize) {
    let mut pattern = qr::version_info(version);
    for i in 0..qr::N_VERSION_BITS {
        let (a, b) = qr::version_bit_pos(i);
        let x = n_modules - 11 + a;
        let y = b;
        if (pattern & 1) > 0 {
            matrix.set_all(x, y);
            matrix.set_all(y, x);
        }
        pattern /= 2;
    }
}


//  ************************************************************
fn set_alignment_patterns(matrix: &mut Matrix, version: u8) {
    for (x, y) in qr::AlignmentPatternIterator::new(version) {
        let x = x - 2;
        let y = y - 2;
        // Outer 5x5 black boundary
        for i in 0..=3 {
            matrix.set_all(x + i, y);
            matrix.set_all(x + 4, y + i);
            matrix.set_all(x + 4 - i, y + 4);
            matrix.set_all(x, y + 4 - i);
        }
        // center black
        matrix.set_all(x + 2, y + 2);
    }
}


//  ************************************************************
fn set_format(matrix: &mut Matrix, n_modules: usize, ec: ErrorCorrectionLevel) {
    trace!("set_format: n_modules={} ec={:?}", n_modules, ec);
    let mut formats = [0u16; 8];
    for f in 0..8 {
        formats[f] = qr::format_info(f as u8, ec);
    }
    trace!("set_format: formats={:?}", formats);
    let mut bytes = [0u8; 16];
    let mut m = 1u16;
    for i in 0..16 {
        let mut b = 0u8;
        for j in 0..8 {
            insane!("set_format: j={}", j);
            b |= (((formats[j] & m) >> i) << j) as u8;
            insane!("set_format: b={}", b);
        }
        insane!("set_format: B={}", b);
        bytes[i as usize] = b;
        m <<= 1;
    }
    trace!("set_format: bytes={:?}", bytes);

    for i in 0..qr::N_FORMAT_BITS {
        let [(x0, y0), (x1, y1)] = qr::format_bit_positions(i, n_modules);
        matrix.set(x0, y0, bytes[i]);
        matrix.set(x1, y1, bytes[i]);
    }
    let (xb, yb) = qr::format_bit_black_position(n_modules);
    matrix.set_all(xb, yb);
}


//  ************************************************************
fn encode_numeric(bits: &mut BitSeq, text: &[u8]) {
    let n = text.len();
    if n == 0 {
        return;
    }
    for i in 0..n {
        if (text[i] < 48) || (text[i] > 57) {
            panic!("Invalid character for Numeric encoding");
        }
    }
    if n > 2 {
        for i in (0..n - 2).step_by(3) {
            let val = 100 * (text[i] as u16 - 48) + 10 * (text[i + 1] as u16 - 48) + (text[i + 2] as u16 - 48);
            bits.append_bits(val, 10);
        }
    }
    if n % 3 == 1 {
        let val = text[n - 1] as u16 - 48;
        bits.append_bits(val, 4);
    } else if n % 3 == 2 {
        let val = 10 * (text[n - 2] as u16 - 48) + (text[n - 1] as u16 - 48);
        bits.append_bits(val, 7);
    }
}

//  ************************************************************
fn encode_alpha_numeric(bits: &mut BitSeq, text: &[u8]) {
    let n = text.len();
    if n == 0 {
        return;
    }
    for i in (0..n - 1).step_by(2) {
        let val = 45 * (qr::ascii_to_alnum(text[i]) as u16) + (qr::ascii_to_alnum(text[i + 1]) as u16);
        bits.append_bits(val, 11);
    }
    if n % 2 == 1 {
        let val = qr::ascii_to_alnum(text[n - 1]) as u16;
        bits.append_bits(val, 6);
    }
}

//  ************************************************************
fn encode_eight_bit(bits: &mut BitSeq, text: &[u8]) {
    for ch in text {
        bits.append_bits(*ch as u16, 8);
    }
}

//  ************************************************************
fn set_padding(bits: &mut BitSeq, version: u8, ec: ErrorCorrectionLevel) {
    let pad: [u8; 2] = [0xEC, 0x11];
    let mut pi = 0;
    let n = (qr::n_codewords(version) - qr::n_ec_codewords(version, ec)) as usize;
    for i in bits.next_byte_idx()..n {
        bits.set_u8(pad[pi], i);
        pi = 1 - pi;
    }
}

//  ************************************************************
fn create_mask_patterns() -> Matrix {
    let mut m = Matrix::new(6);
    for i in 0..6 {
        for j in 0..6 {
            let mut val = 0;
            let mut pat = 1;
            for n in 0..8 {
                if qr::mask(n, i, j) {
                    val |= pat
                };
                pat <<= 1;
            }
            m.set(i, j, val);
        }
    }
    m
}

//  ************************************************************
fn add_error_correction(text_bytes: Vec<u8>, version: u8, ec: ErrorCorrectionLevel) -> Vec<u8> {
    debug!("add_error_correction: text_bytes.len={} version={} ec={:?}", text_bytes.len(), version, ec);
    trace!("add_error_correction: text_bytes={:?}", text_bytes);
    let [ecb1, ecb2] = qr::ec_blocks(version, ec);
    trace!("add_error_correction: ecb1={:?} ecb2={:?}", ecb1, ecb2);
    let [e1, e2] = [ecb1.c - ecb1.k, ecb2.c - ecb2.k];
    let n_ecw = e1;
    let n_dcw = ecb1.n * ecb1.k + ecb2.n * ecb2.k;
    let n_blocks = ecb1.n + ecb2.n;
    let n_out_codewords = ecb1.n * ecb1.c + ecb2.n * ecb2.c;
    trace!(
        "add_error_correction: e1={} e2={} n_ecw={} n_dcw={} n_blocks={} n_out_codewords={}",
        e1,
        e2,
        n_ecw,
        n_dcw,
        n_blocks,
        n_out_codewords
    );
    if e2 > 1 {
        if e1 != e2 {
            warn!("add_error_correction: INCONSISTENT NUMBER OF ERROR CORRECTION WORDS: e1={} e2={}", e1, e2);
        }
    }
    let l = text_bytes.len();
    if l != n_dcw {
        warn!("add_error_correction: INCONSISTENT INPUT LENGTH: text.len={} n_dcw={}", l, n_dcw);
    }
    let [rs1, rs2] = [ReedSolomonEncoder::new(e1), ReedSolomonEncoder::new(e2)];
    let mut ec_codewords = Vec::<Vec<u8>>::with_capacity(ecb1.n + ecb2.n);
    let mut n = 0;
    for _ in 0..(ecb1.n) {
        ec_codewords.push(rs1.encode(&text_bytes[n..n + ecb1.k]));
        n += ecb1.k;
    }
    for _ in 0..(ecb2.n) {
        ec_codewords.push(rs2.encode(&text_bytes[n..n + ecb2.k]));
        n += ecb2.k;
    }

    let mut out_bytes = Vec::<u8>::with_capacity(n_out_codewords);
    for i in 0..ecb1.k {
        for j in 0..ecb1.n {
            out_bytes.push(text_bytes[i + j * ecb1.k]);
        }
        for j in 0..ecb2.n {
            out_bytes.push(text_bytes[i + j + (j + ecb1.n) * ecb1.k]);
        }
    }
    let k = ecb1.k;
    for j in ecb1.n..ecb1.n + ecb2.n {
        out_bytes.push(text_bytes[k + j * ecb1.k]);
    }
    for i in 0..n_ecw {
        for j in 0..n_blocks {
            out_bytes.push(ec_codewords[j][i]);
        }
    }
    trace!("add_error_correction: out_bytes.len={} out_bytes={:?}", out_bytes.len(), out_bytes);
    let l = out_bytes.len();
    if l != n_out_codewords {
        warn!(
            "add_error_correction: INCONSISTENT NUMBER OF OUTPUT CODE WORDS: out_bytes.len={} n_out_codewords={}",
            l, n_out_codewords
        );
    }
    out_bytes
}

//  ************************************************************
fn set_data_snaked(matrix: &mut Matrix, mask: &Matrix, bits: &BitSeq, version: u8) {
    debug!("set_data_snaked: begin");
    let mut bi = bits.into_iter();
    let mut rem_bits = 0;
    for (x, y) in qr::SnakeDataIterator::new(version) {
        let m = mask.get(x % 6, y % 6);
        match bi.next() {
            None => {
                matrix.set(x, y, m);
                rem_bits += 1;
            }
            Some(v) => {
                let b = if v { !m } else { m };
                insane!("set_data_snaked: set v={} m={} b={}", v, m, b);
                matrix.set(x, y, b);
            }
        }
    }
    let rem_bits_expected = qr::n_remainder_bits(version);
    if rem_bits != rem_bits_expected {
        error!("WRONG NUMBER OF REMINDER BITS got={}; expected={}", rem_bits, rem_bits_expected);
        panic!("WRONG NUMBER OF REMINDER BITS got={}; expected={}", rem_bits, rem_bits_expected);
    }
    if rem_bits == 0 {
        if bi.next().is_some() {
            error!("TOO MUCH DATA FOR SNAKE");
            panic!("TOO MUCH DATA FOR SNAKE");
        }
    }
    debug!("set_data_snaked: done");
}

//  ************************************************************
pub fn best_penalty(matrix: &Matrix, n_modules: usize) -> u8 {
    let mut best_penalty = u32::max_value();
    let mut best_mask = 0u8;
    for mask in 0..8 {
        let p = penalty(matrix, n_modules, mask);
        if p < best_penalty {
            best_penalty = p;
            best_mask = mask;
        }
    }
    debug!("best_penalty: best_penalty={} best_mask={}", best_penalty, best_mask);
    best_mask
}

//  ************************************************************
pub fn penalty(matrix: &Matrix, n_modules: usize, mask: u8) -> u32 {
    let p_adjacent = penalty_adjacent(matrix, n_modules, mask);
    let p_blocks = penalty_blocks(matrix, n_modules, mask);
    let p_ratio = penalty_ratio(matrix, n_modules, mask);
    let p_dark = penalty_dark(matrix, n_modules, mask);
    let p_total = p_adjacent + p_blocks + p_ratio + p_dark;
    trace!(
        "penalty: mask={} p_adjacent={} p_blocks={} p_ratio={} p_dark={} p_total={}",
        mask,
        p_adjacent,
        p_blocks,
        p_ratio,
        p_dark,
        p_total
    );
    p_total
}

const PENALTY_N1: u32 = 3;
const PENALTY_N2: u32 = 3;
const PENALTY_N3: u32 = 40;
const PENALTY_N4: u32 = 10;

//  ************************************************************
/// Calculate penalty for adjacent modules in row/column in same color
///
/// # Feature:
/// Adjacent modules in row/column in same color
///
/// # Evaluation condition:
/// No. of modules = (5 + i)
///
/// # Points:
/// N1 + i
//  ************************************************************
pub fn penalty_adjacent(matrix: &Matrix, n_modules: usize, mask: u8) -> u32 {
    let mut points = 0;
    for rc in [true, false].iter() {
        for i in 0..n_modules {
            let mut n = 0;
            let mut counting_dark = false;
            for j in 0..n_modules {
                let is_dark = if *rc { matrix.get_one(i, j, mask) } else { matrix.get_one(j, i, mask) };
                if j == 0 {
                    counting_dark = is_dark;
                    n = 0;
                }
                if counting_dark == is_dark {
                    n += 1;
                } else {
                    if n >= 5 {
                        insane!("penalty_adjacent: rc={} i={} j={} n={}", rc, i, j, n);
                        points += PENALTY_N1 + n - 5;
                    }
                    counting_dark = !counting_dark;
                    n = 1;
                }
            }
            if n >= 5 {
                insane!("penalty_adjacent: rc={} i={} j=END n={}", rc, i, n);
                points += PENALTY_N1 + n - 5;
            }
        }
    }
    points
}


//  ************************************************************
/// Calculate penalty for block of modules in same color
///
/// # Feature:
/// Block of modules in same color
///
/// # Evaluation condition:
/// Block size = m * n
///
/// # Points:
/// N2 * (m-1) * (n-1)
//  ************************************************************
pub fn penalty_blocks(matrix: &Matrix, n_modules: usize, mask: u8) -> u32 {
    // Not clear from ISO standard, if blocks have to be rectangular?
    // Here we give 3 penalty to every 2x2 block, so odd shaped areas will have penalties as well as rectangles
    let mut p = 0;
    for i in 0..n_modules - 1 {
        for j in 0..n_modules - 1 {
            let mut b = 0;
            if matrix.get_one(i, j, mask) {
                b += 1;
            }
            if matrix.get_one(i + 1, j, mask) {
                b += 1;
            }
            if matrix.get_one(i, j + 1, mask) {
                b += 1;
            }
            if matrix.get_one(i + 1, j + 1, mask) {
                b += 1;
            }
            if (b == 0) || (b == 4) {
                p += PENALTY_N2;
            }
        }
    }
    p
}

//  ************************************************************
/// Calculate penalty for 1:1:3:1:1 ratio (dark:light:dark:light:dark) pattern in row/column
///
/// # Feature:
/// 1:1:3:1:1 ratio (dark:light:dark:light:dark) pattern in row/column,
/// preceded or followed by light area 4 modules wide
///
/// # Evaluation condition:
/// Existence of the pattern
///
/// # Points:
/// N3
//  ************************************************************
pub fn penalty_ratio(matrix: &Matrix, n_modules: usize, mask: u8) -> u32 {
    let mut points = 0;
    let mut pat = PenaltyPattern::new();
    for rc in [true, false].iter() {
        for i in 0..n_modules {
            pat.reset();
            let mut counting_dark = false;
            let mut n = 0;
            for j in 0..n_modules {
                let is_dark = if *rc { matrix.get_one(i, j, mask) } else { matrix.get_one(j, i, mask) };
                if counting_dark == is_dark {
                    n += 1;
                } else {
                    if pat.push(counting_dark, n) {
                        insane!("penalty_ratio: rc={} i={} j={} {:?}", rc, i, j, pat);
                        points += PENALTY_N3;
                    }
                    counting_dark = !counting_dark;
                    n = 1;
                }
            }
            if pat.push(counting_dark, n) {
                insane!("penalty_ratio: rc={} i={} j=END {:?}", rc, i, pat);
                points += PENALTY_N3;
            }
        }
    }
    points
}

//  ************************************************************
/// Calculate penalty for proportion of dark modules in entire symbol
///    
/// # Feature:
/// Proportion of dark modules in entire symbol
///
/// # Evaluation condition:
/// 50 +/- (5*k)% to 50 +/- (5*(k+1))%
///
/// # Points:
/// N4 x k
//  ************************************************************
pub fn penalty_dark(matrix: &Matrix, n_modules: usize, mask: u8) -> u32 {
    let mut dark = 0;
    for x in 0..n_modules {
        for y in 0..n_modules {
            if matrix.get_one(x, y, mask) {
                dark += 1;
            }
        }
    }
    let diff = (20 * dark as usize) / (n_modules * n_modules);
    let p = if diff < 10 { 10 - diff } else { diff - 10 };
    (p as u32) * PENALTY_N4
}


//  ************************************************************
/// Helper structure to find 1:1:3:1:1 patterns to give penalty
//  ************************************************************

#[derive(Debug)]
pub struct PenaltyPattern {
    pat: [u8; 8],
    idx: usize,
    cnt: usize,
}

//  ************************************************************
impl PenaltyPattern {
    //  ************************************************************
    pub fn new() -> Self {
        PenaltyPattern { pat: [0; 8], idx: 0, cnt: 0 }
    }

    //  ************************************************************
    pub fn reset(&mut self) {
        self.pat = [0; 8];
        self.idx = 0;
        self.cnt = 0;
    }

    //  ************************************************************
    pub fn push(&mut self, dark: bool, n: u8) -> bool {
        self.cnt += 1;
        match dark {
            true => self.push_dark(n),
            false => self.push_light(n),
        }
    }

    //  ************************************************************
    pub fn push_light(&mut self, n: u8) -> bool {
        assert!(self.idx % 2 == 0);
        self.pat[self.idx] = n;
        self.idx = (self.idx + 1) % 8;
        let i = self.idx;
        if (self.cnt >= 5 + 1) && (n >= 4) && (self.pat[(i + 1) % 8] < 4) {
            let n1 = self.pat[(i + 2) % 8];
            (n1 > 0)
                && (self.pat[(i + 3) % 8] == n1)
                && (self.pat[(i + 4) % 8] == 3 * n1)
                && (self.pat[(i + 5) % 8] == n1)
                && (self.pat[(i + 6) % 8] == n1)
        } else {
            false
        }
    }

    //  ************************************************************
    pub fn push_dark(&mut self, n: u8) -> bool {
        assert!(self.idx % 2 == 1);
        self.pat[self.idx] = n;
        self.idx = (self.idx + 1) % 8;
        let i = self.idx;
        if (n > 0) && (self.cnt >= 5 + 1) && (self.pat[(i + 2) % 8] >= 4) {
            (self.pat[(i + 3) % 8] == n)
                && (self.pat[(i + 4) % 8] == n)
                && (self.pat[(i + 5) % 8] == 3 * n)
                && (self.pat[(i + 6) % 8] == n)
        } else {
            false
        }
    }
}


//  ************************************************************
/// Rectangular matrix of `u8` (byte) values
///
/// Each (x,y) coordinate in the matrix references a byte
///
/// Bytes can be `set` or `get` as a whole, or individual bits can be targetd.
///
/// A specific bit may be designated as `selected`
//  ************************************************************

#[derive(Debug)]
pub struct Matrix {
    dim: usize,
    selected: u8,
    data: Vec<u8>,
}

//  ************************************************************
impl Matrix {
    pub fn new(dim: usize) -> Self {
        Matrix { dim, selected: 0, data: vec![0; dim * dim] }
    }
    pub fn set_all(&mut self, x: usize, y: usize) {
        self.data[x + y * self.dim] = 0xFF;
    }
    pub fn set(&mut self, x: usize, y: usize, b: u8) {
        self.data[x + y * self.dim] = b;
    }
    pub fn select(&mut self, i: u8) {
        self.selected = i;
    }
    pub fn get_dim(&self) -> usize {
        self.dim
    }
    pub fn get(&self, x: usize, y: usize) -> u8 {
        self.data[x + y * self.dim]
    }
    pub fn get_one(&self, x: usize, y: usize, i: u8) -> bool {
        (self.data[x + y * self.dim] & (1 << i)) > 0
    }
    pub fn get_selected(&self, x: usize, y: usize) -> bool {
        (self.data[x + y * self.dim] & (1 << self.selected)) > 0
    }
}


//  ************************************************************
#[cfg(test)]
//  ************************************************************

mod qrencode {
    use super::*;
    use std::cmp;

    //  ************************************************************

    #[test]
    fn test_encode_8bit_l_le1000() {
        test_helper_seq(ErrorCorrectionLevel::L, Mode::EightBit, 1000, 111)
    }
    #[test]
    fn test_encode_anum_l_le2000() {
        test_helper_seq(ErrorCorrectionLevel::L, Mode::AlphaNumeric, 2000, 112)
    }
    #[test]
    #[allow(non_snake_case)]
    fn test_encode_num__l_le3000() {
        test_helper_seq(ErrorCorrectionLevel::L, Mode::Numeric, 3000, 113)
    }

    #[test]
    fn test_encode_8bit_m_le1000() {
        test_helper_seq(ErrorCorrectionLevel::M, Mode::EightBit, 1000, 121)
    }
    #[test]
    fn test_encode_anum_m_le2000() {
        test_helper_seq(ErrorCorrectionLevel::M, Mode::AlphaNumeric, 2000, 122)
    }
    #[test]
    #[allow(non_snake_case)]
    fn test_encode_num__m_le3000() {
        test_helper_seq(ErrorCorrectionLevel::M, Mode::Numeric, 3000, 123)
    }

    #[test]
    fn test_encode_8bit_q_le1000() {
        test_helper_seq(ErrorCorrectionLevel::Q, Mode::EightBit, 1000, 131)
    }
    #[test]
    fn test_encode_anum_q_le2000() {
        test_helper_seq(ErrorCorrectionLevel::Q, Mode::AlphaNumeric, 2000, 132)
    }
    #[test]
    #[allow(non_snake_case)]
    fn test_encode_num__q_le3000() {
        //test_helper_8bit(ErrorCorrectionLevel::Q, 3)
        test_helper_seq(ErrorCorrectionLevel::Q, Mode::Numeric, 3000, 133)
    }

    #[test]
    fn test_encode_8bit_h_le1000() {
        test_helper_seq(ErrorCorrectionLevel::H, Mode::EightBit, 1000, 141)
    }
    #[test]
    fn test_encode_anum_h_le2000() {
        test_helper_seq(ErrorCorrectionLevel::H, Mode::AlphaNumeric, 2000, 142)
    }
    #[test]
    #[allow(non_snake_case)]
    fn test_encode_num__h_le3000() {
        test_helper_seq(ErrorCorrectionLevel::H, Mode::Numeric, 3000, 143)
    }

    //  ************************************************************

    #[test]
    fn test_encode_8bit_l_gt1000() {
        test_helper_arr(ErrorCorrectionLevel::L, Mode::EightBit, &BIT8_LEN_ARR, 211)
    }
    #[test]
    fn test_encode_anum_l_gt2000() {
        test_helper_arr(ErrorCorrectionLevel::L, Mode::AlphaNumeric, &ALNUM_LEN_ARR, 212)
    }
    #[test]
    #[allow(non_snake_case)]
    fn test_encode_num__l_gt3000() {
        test_helper_arr(ErrorCorrectionLevel::L, Mode::Numeric, &NUM_LEN_ARR, 213)
    }

    #[test]
    fn test_encode_8bit_m_gt1000() {
        test_helper_arr(ErrorCorrectionLevel::M, Mode::EightBit, &BIT8_LEN_ARR, 221)
    }
    #[test]
    fn test_encode_anum_m_gt2000() {
        test_helper_arr(ErrorCorrectionLevel::M, Mode::AlphaNumeric, &ALNUM_LEN_ARR, 222)
    }
    #[test]
    #[allow(non_snake_case)]
    fn test_encode_num__m_gt3000() {
        test_helper_arr(ErrorCorrectionLevel::M, Mode::Numeric, &NUM_LEN_ARR, 223)
    }

    #[test]
    fn test_encode_8bit_q_gt1000() {
        test_helper_arr(ErrorCorrectionLevel::Q, Mode::EightBit, &BIT8_LEN_ARR, 231)
    }
    #[test]
    fn test_encode_anum_q_gt2000() {
        test_helper_arr(ErrorCorrectionLevel::Q, Mode::AlphaNumeric, &ALNUM_LEN_ARR, 232)
    }
    #[test]
    #[allow(non_snake_case)]
    fn test_encode_num__q_gt3000() {
        test_helper_arr(ErrorCorrectionLevel::Q, Mode::Numeric, &NUM_LEN_ARR, 233)
    }

    #[test]
    fn test_encode_8bit_h_gt1000() {
        test_helper_arr(ErrorCorrectionLevel::H, Mode::EightBit, &BIT8_LEN_ARR, 241)
    }
    #[test]
    fn test_encode_anum_h_gt2000() {
        test_helper_arr(ErrorCorrectionLevel::H, Mode::AlphaNumeric, &ALNUM_LEN_ARR, 242)
    }
    #[test]
    #[allow(non_snake_case)]
    fn test_encode_num__h_gt3000() {
        test_helper_arr(ErrorCorrectionLevel::H, Mode::Numeric, &NUM_LEN_ARR, 243)
    }

    //  ************************************************************

    #[test]
    fn test_encode_8bit_l_rnd() {
        test_helper_rnd(ErrorCorrectionLevel::L, Mode::EightBit, 311)
    }
    #[test]
    fn test_encode_anum_l_rnd() {
        test_helper_rnd(ErrorCorrectionLevel::L, Mode::AlphaNumeric, 312)
    }
    #[test]
    #[allow(non_snake_case)]
    fn test_encode_num__l_rnd() {
        test_helper_rnd(ErrorCorrectionLevel::L, Mode::Numeric, 313)
    }

    #[test]
    fn test_encode_8bit_m_rnd() {
        test_helper_rnd(ErrorCorrectionLevel::M, Mode::EightBit, 321)
    }
    #[test]
    fn test_encode_anum_m_rnd() {
        test_helper_rnd(ErrorCorrectionLevel::M, Mode::AlphaNumeric, 322)
    }
    #[test]
    #[allow(non_snake_case)]
    fn test_encode_num__m_rnd() {
        test_helper_rnd(ErrorCorrectionLevel::M, Mode::Numeric, 323)
    }

    #[test]
    fn test_encode_8bit_q_rnd() {
        test_helper_rnd(ErrorCorrectionLevel::Q, Mode::EightBit, 331)
    }
    #[test]
    fn test_encode_anum_q_rnd() {
        test_helper_rnd(ErrorCorrectionLevel::Q, Mode::AlphaNumeric, 332)
    }
    #[test]
    #[allow(non_snake_case)]
    fn test_encode_num__q_rnd() {
        test_helper_rnd(ErrorCorrectionLevel::Q, Mode::Numeric, 333)
    }

    #[test]
    fn test_encode_8bit_h_rnd() {
        test_helper_rnd(ErrorCorrectionLevel::H, Mode::EightBit, 341)
    }
    #[test]
    fn test_encode_anum_h_rnd() {
        test_helper_rnd(ErrorCorrectionLevel::H, Mode::AlphaNumeric, 342)
    }
    #[test]
    #[allow(non_snake_case)]
    fn test_encode_num__h_rnd() {
        test_helper_rnd(ErrorCorrectionLevel::H, Mode::Numeric, 343)
    }


    //  ************************************************************

    fn test_helper_seq(ec: ErrorCorrectionLevel, mode: Mode, max: usize, seed: u32) {
        let mut rng = Rng::new(seed);
        let n = cmp::min(max, qr::data_capacity(qr::VERSION_MAX, mode, ec) as usize);
        let mut txt = Vec::<u8>::with_capacity(n);
        for i in 0..n {
            let m = 0; // TODO: consider increasing m
            let m0 = qr::version_from_length(i, mode, ec).unwrap();
            let m1 = cmp::min(m0 + m, qr::VERSION_MAX);

            for v in m0..=m1 {
                println!("\n\n========== TEST ==========> n={} i={} v={} txt.len={}", n, i, v, txt.len());
                println!("txt={:?}", txt);
                encode(&txt[..], v, mode, ec);
            }
            txt.push(rng.get_u8_with_mode(mode));
        }
    }


    //  ************************************************************

    fn test_helper_rnd(ec: ErrorCorrectionLevel, mode: Mode, seed: u32) {
        let mut rng = Rng::new(seed);
        let cap = qr::data_capacity(qr::VERSION_MAX, mode, ec) as usize;
        let mut txt = Vec::<u8>::with_capacity(cap);
        let n = 100;
        for j in 0..n {
            let i = rng.get_usize_with_modulus(cap);
            for _ in txt.len()..i {
                txt.push(rng.get_u8_with_mode(mode));
            }
            let m = 3;
            let m0 = qr::version_from_length(i, mode, ec).unwrap();
            let m1 = cmp::min(m0 + m, qr::VERSION_MAX);

            for v in m0..=m1 {
                println!("\n\n========== TEST ==========> j={} i={} v={} txt.len={}", j, i, v, txt.len());
                println!("txt={:?}", txt);
                encode(&txt[0..i], v, mode, ec);
            }
            txt.push(rng.get_u8_with_mode(mode));
        }
    }


    //  ************************************************************

    fn test_helper_arr(ec: ErrorCorrectionLevel, mode: Mode, arr: &[usize], seed: u32) {
        let mut rng = Rng::new(seed);
        let mut txt = Vec::<u8>::with_capacity(arr[arr.len() - 1]);
        for j in arr.iter() {
            for i in *j - 1..=*j {
                if i <= qr::data_capacity(qr::VERSION_MAX, mode, ec) as usize {
                    for _ in txt.len()..i {
                        txt.push(rng.get_u8_with_mode(mode));
                    }

                    let m = 2;
                    let m0 = qr::version_from_length(i, mode, ec).unwrap();
                    let m1 = cmp::min(m0 + m, qr::VERSION_MAX);

                    for v in m0..=m1 {
                        println!("\n\n========== TEST ==========> i={} v={} txt.len={}", i, v, txt.len());
                        println!("txt={:?}", txt);
                        encode(&txt[..], v, mode, ec);
                    }
                }
            }
        }
    }


    //  ************************************************************

    const BIT8_LEN_ARR: [usize; 47] = [
        1003, 1030, 1051, 1059, 1091, 1093, 1112, 1125, 1139, 1168, 1171, 1190, 1219, 1228, 1264, 1273, 1283, 1351, 1367, 1370,
        1423, 1452, 1465, 1499, 1528, 1538, 1579, 1628, 1663, 1722, 1732, 1809, 1840, 1911, 1952, 1989, 2068, 2099, 2188, 2213,
        2303, 2331, 2431, 2563, 2699, 2809, 2953,
    ];

    const ALNUM_LEN_ARR: [usize; 27] = [
        2071, 2113, 2132, 2181, 2223, 2238, 2298, 2369, 2420, 2506, 2520, 2632, 2677, 2780, 2840, 2894, 3009, 3054, 3183, 3220,
        3351, 3391, 3537, 3729, 3927, 4087, 4296,
    ];

    const NUM_LEN_ARR: [usize; 33] = [
        3035, 3057, 3081, 3244, 3283, 3289, 3417, 3486, 3517, 3599, 3669, 3693, 3791, 3909, 3993, 4134, 4158, 4343, 4417, 4588,
        4684, 4775, 4965, 5039, 5253, 5313, 5529, 5596, 5836, 6153, 6479, 6743, 7089,
    ];


    //  ************************************************************
    /// Very simple XORSHIFT pseudo random number generator
    ///
    /// # References
    ///
    /// - <https://en.wikipedia.org/wiki/Xorshift>
    /// - <http://www.jstatsoft.org/v08/i14/paper>
    //  ************************************************************

    struct Rng {
        state: u32,
    }

    //  ************************************************************
    impl Rng {
        pub fn new(seed: u32) -> Self {
            Rng { state: seed }
        }
        pub fn get_u32(&mut self) -> u32 {
            self.state ^= self.state << 13;
            self.state ^= self.state >> 17;
            self.state ^= self.state << 5;
            self.state
        }
        pub fn get_u8(&mut self) -> u8 {
            self.get_u32() as u8
        }
        pub fn get_u8_with_mode(&mut self, mode: Mode) -> u8 {
            let u = self.get_u8();
            match mode {
                Mode::EightBit => u,
                Mode::AlphaNumeric => qr::alnum_to_ascii(u % 45),
                Mode::Numeric => 0x30 + u % 10,
            }
        }
        pub fn get_usize_with_modulus(&mut self, divisor: usize) -> usize {
            (self.get_u32() as usize) % (divisor + 1)
        }
    }
}
