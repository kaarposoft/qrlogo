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
//! Decode QR Code to text
//!
//! A simple QR Code decoder implementation
//!
//! The decoder can only handle images which have
//!
//! - clear separation in color luma (lightness) between "black" and "white" modules/pixels, in particular in the functional areas
//! - QR Code approximatly centered in the image (each finder in the North-West, South-West, and North-East quadrants of the image, respectively)
//! - QR Code axes approximatly aligned to the image axes
//! - QR Code approximatly square, with no skew or stretch
//  ************************************************************

#![allow(clippy::new_without_default_derive)]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::int_plus_one)]

use std::cmp::Ordering;
use std::collections::{BinaryHeap, VecDeque};
use std::f64;
use std::io::Write;
use std::str;

use super::logging;
use super::qr;
use super::qr::BitSeq;
use super::reedsolomon::reed_solomon_decoder;
use super::{ErrorCorrectionLevel, Mode, RGBAImage};


//  ************************************************************
/// Decode QR Code to text
///
/// - If `aggressive` is `false`, a "perfect" QR Code will be identified fast
/// - If `aggressive` is `true`, we try harder to find a (possibly low quality) QR Code, but decoding a "perfect" QR Code may take slightly longer than when `aggressive` is `false`
//  ************************************************************

// TODO: Break this function into smaller pieces...
// As Clippy reports, it has become far too complex.
#[allow(clippy::cyclomatic_complexity)]
pub fn decode_image<T: RGBAImage>(image: &T, aggressive: bool) -> DecodingResult {
    let mut best_res = None;
    let mut best_grade2 = 0.0;
    log!(
        "decode_image: width={} height={} pixels={} aggressive={}",
        image.width(),
        image.height(),
        image.width() * image.height(),
        aggressive
    );

    // Calculate a `gray_matrix`
    // where each pixel represents the luma (lightness) of the corresponding image pixel

    let mut gray_matrix = match GrayMatrix::new(image) {
        Ok(gm) => gm,
        Err(err) => {
            return DecodingResult::from_error(err);
        }
    };
    if log_enabled!(2) {
        gray_matrix.log_stats();
    }
    #[cfg(debug_assertions)]
    {
        if log_enabled!(3) {
            gray_matrix.log_matrix();
        }
    }

    // Loop over different luma (lightness) thresholds.
    // ISO 18004 specifies to use the mid-point between minimum and maximum "reflectance".
    // However, the average luma seems to be a more reasonable alternative.
    // So first try the average of the avereage and the mid-point.
    // If `aggressive`, also try the mid-point and the average itself.

    let (min_luma, max_luma, avg_luma) = gray_matrix.get_min_max_avg_luma();
    let mid_luma = (max_luma + min_luma) / 2.0;
    let mut lights = vec![(mid_luma + avg_luma) / 2.0];
    if aggressive {
        lights.push(mid_luma);
        lights.push(avg_luma);
    }

    'light_iteration: for light in lights {
        debug!("decode_image: light_iteration: light={:.3}", light);
        gray_matrix.threshold_luma = light;
        let hit_matrix = HitMatrix::with_gray_matrix(&gray_matrix, aggressive);
        #[cfg(debug_assertions)]
        {
            if log_enabled!(3) {
                hit_matrix.log_matrix();
            }
        }
        if log_enabled!(2) {
            hit_matrix.log_stats();
        }

        // For each of the three corners (NW, SW, NE) create an iterator over possible finder patterns

        let mut nw_candidate_iterator = FinderCandidateIterator::new(&gray_matrix, &hit_matrix, Corner::NW, aggressive);
        let mut nw_candidates = Vec::new();
        if let Some(c) = nw_candidate_iterator.next() {
            nw_candidates.push(c);
        } else {
            return DecodingResult::from_error("No North-West finder pattern found");
        }

        let mut sw_candidate_iterator = FinderCandidateIterator::new(&gray_matrix, &hit_matrix, Corner::SW, aggressive);
        let mut sw_candidates = Vec::new();
        if let Some(c) = sw_candidate_iterator.next() {
            sw_candidates.push(c);
        } else {
            return DecodingResult::from_error("No South-West finder pattern found");
        }

        let mut ne_candidate_iterator = FinderCandidateIterator::new(&gray_matrix, &hit_matrix, Corner::NE, aggressive);
        let mut ne_candidates = Vec::new();
        if let Some(c) = ne_candidate_iterator.next() {
            ne_candidates.push(c);
        } else {
            return DecodingResult::from_error("No North-East finder pattern found");
        }

        // If we have found a "good" candidate, look no further,
        // unless we are `agressive`,
        // where we might try a bit more to find an even better candidate

        let mut n_after_good = 0;
        let n_after_good_max = if aggressive { 4 } else { 0 };

        // Loop through the possible finder patterns,
        // each time adding a possible finder pattern from NW, SW, or NE corner,
        // and checking of any combination of finder patterns would be a suitable finder area
        // (ie. nicely aligned finder patterns)

        let mut current_corner = Corner::SW;
        'finder_candidate_iteration: loop {
            debug!(
                "decode_image: finder_candidate_iteration: nw.len={} sw.len={} ne.len={}",
                nw_candidates.len(),
                sw_candidates.len(),
                ne_candidates.len()
            );
            for a in FinderAreaIterator::new(&nw_candidates, &sw_candidates, &ne_candidates, current_corner, aggressive) {
                debug!("decode_image: FinderAreaIterator: {:?}", a);
                for sa in SymbolAreaIterator::new(&gray_matrix, &a, aggressive) {
                    debug!("decode_image: SymbolAreaIterator: {:?}", sa);
                    let g = sa.grade();
                    if g > 0.0 {
                        trace!("decode_image: found functional candidate grade={}", g);
                        let mut res = DecodingResult::from_symbol_area(&sa);
                        let snaked = get_snaked_data(&sa, &gray_matrix);
                        insane!("decode_image: get_snaked_data: len={} {:?}", snaked.len(), snaked);
                        let raw = correct_errors(&snaked, qr::ec_blocks(sa.version, sa.ec));
                        insane!("decode_image: correct_errors: {:?}", raw);
                        match raw {
                            Err(e) => res.err = Some(format!("correct_errors failed with {}", e)),
                            Ok((r, g)) => {
                                res.decoding_grade = g;
                                match decode_data(sa.version, sa.ec, &r) {
                                    Ok((mode, data)) => {
                                        res.mode = Some(mode);
                                        res.data = Some(data);
                                    }
                                    Err(err) => {
                                        res.decoding_grade = f64::NAN;
                                        res.err = Some(err);
                                    }
                                }
                            }
                        }
                        let rg2 = res.grade2();
                        if rg2 > best_grade2 {
                            trace!("decode_image: found better result. old grade2={} new grade2={}", best_grade2, rg2);
                            best_grade2 = rg2;
                            best_res = Some(res);
                            let rg2_threshold = if aggressive { 10.0 + 2.0 + 2.0 } else { 10.0 + 4.0 + 4.0 };
                            if rg2 >= rg2_threshold {
                                log!("decode_image: found a quite good result");
                                break 'light_iteration;
                            }
                        } else {
                            trace!("decode_image: did NOT find better result. old grade2={} new grade2={}", best_grade2, rg2);
                        }
                    }


                    trace!(
                        "decode_image: best_grade2 {} n_after_good={} n_after_good_max={}",
                        best_grade2,
                        n_after_good,
                        n_after_good_max
                    );
                    if best_grade2 >= 10.0 {
                        n_after_good += 1;
                        if n_after_good > n_after_good_max {
                            log!("decode_image: found {} candidates after good; try no more", n_after_good - 1);
                            break 'light_iteration;
                        }
                    }
                }
            }

            // Try to find one more finder candidate (preferably in a different corner than the current corner)

            let mut next_try = 0;
            loop {
                trace!("decode_image: end-of-loop: current_corner={:?}", current_corner);
                match current_corner {
                    Corner::NW => {
                        current_corner = Corner::SW;
                        if let Some(c) = sw_candidate_iterator.next() {
                            sw_candidates.push(c);
                            break;
                        } else {
                            next_try += 1;
                        }
                    }
                    Corner::SW => {
                        current_corner = Corner::NE;
                        if let Some(c) = ne_candidate_iterator.next() {
                            ne_candidates.push(c);
                            break;
                        } else {
                            next_try += 1;
                        }
                    }
                    Corner::NE => {
                        current_corner = Corner::NW;
                        if let Some(c) = nw_candidate_iterator.next() {
                            nw_candidates.push(c);
                            break;
                        } else {
                            next_try += 1;
                        }
                    }
                }
                if next_try >= 3 {
                    log!("decode_image: no more finder candidates to try");
                    break 'finder_candidate_iteration;
                }
            }
        }
    }

    if let Some(res) = best_res {
        res
    } else {
        DecodingResult::from_error("unable to decode image")
    }
}


//  ************************************************************
/// The result of trying to decode an image (potentially) containing a QR Code
///
/// A `DecodingResult` should have one of:
/// - `err` describing why the image could not be decoded
/// - `data` containing the decoded data
///
/// Other fields may be `None`
/// or contain information about the best QR Code candidate found.
///
/// Fields representing grades may include `NaN`
/// to signify that no such grade could be calculated.
///
/// Note that according to ISO 18004, a grade is an integer.
/// However here we represent the grades as `f64`,
/// where the fractional part introduces an order between QR Codes
/// with the same ISO 18004 grade.
//  ************************************************************

#[derive(Clone, Debug)]
pub struct DecodingResult {
    pub err: Option<String>,
    pub data: Option<Vec<u8>>,
    pub mode: Option<Mode>,
    pub version: Option<u8>,
    pub mask: Option<u8>,
    pub ec: Option<ErrorCorrectionLevel>,
    pub finder_grades: [f64; 3],
    pub timing_grades: [f64; 2],
    pub alignment_grade: f64,
    pub version_info_grades: [f64; 2],
    pub format_info_grades: [f64; 2],
    pub functional_grade: f64,
    pub decoding_grade: f64,
}

//  ************************************************************

impl Default for DecodingResult {
    fn default() -> Self {
        DecodingResult {
            err: None,
            data: None,
            mode: None,
            version: None,
            mask: None,
            ec: None,
            finder_grades: [f64::NAN; 3],
            timing_grades: [f64::NAN; 2],
            alignment_grade: f64::NAN,
            version_info_grades: [f64::NAN; 2],
            format_info_grades: [f64::NAN; 2],
            functional_grade: f64::NAN,
            decoding_grade: f64::NAN,
        }
    }
}


//  ************************************************************

impl DecodingResult {
    pub fn from_symbol_area(sa: &SymbolArea) -> Self {
        DecodingResult {
            version: Some(sa.version),
            ec: Some(sa.ec),
            mask: Some(sa.mask),
            finder_grades: sa.finder_grades,
            timing_grades: sa.timing_grades,
            alignment_grade: sa.alignment_grade,
            version_info_grades: sa.version_info_grades,
            format_info_grades: sa.format_info_grades,
            functional_grade: sa.grade(),
            ..Default::default()
        }
    }

    pub fn from_error<S: Into<String>>(err: S) -> Self {
        DecodingResult { err: Some(err.into()), ..Default::default() }
    }

    pub fn grade(&self) -> f64 {
        if self.functional_grade.is_nan() {
            return 0.0;
        }
        if self.decoding_grade.is_nan() {
            return 0.0;
        }
        f64::min(self.functional_grade, self.decoding_grade)
    }

    pub fn grade2(&self) -> f64 {
        if self.functional_grade.is_nan() {
            return 0.0;
        }
        if self.decoding_grade.is_nan() {
            return self.functional_grade;
        }
        10.0 + self.functional_grade + self.decoding_grade
    }

    pub fn write<W: Write>(&self, writer: &mut W) {
        if let Some(ref err) = self.err {
            writeln!(writer, "ERROR:   {}", err);
        } else {
            if let Some(ref data) = self.data {
                let mut wrote = false;
                if let Ok(s) = str::from_utf8(data) {
                    if s.find(char::is_control).is_none() {
                        writeln!(writer, "data:    {:?}", s);
                        wrote = true;
                    }
                }
                if !wrote {
                    writeln!(writer, "data:    {} bytes; hex={:02X?}", data.len(), data);
                }
            }
        }
        let _ = match self.mode {
            Some(m) => writeln!(writer, "mode:    {:?}", m),
            None => writer.write_all(b"mode:    n/a\n"),
        };
        let _ = match self.mask {
            Some(m) => writeln!(writer, "mask:    {:?}", m),
            None => writer.write_all(b"mask:    n/a\n"),
        };
        let _ = match self.version {
            Some(v) => writeln!(writer, "version: {:?}", v),
            None => writer.write_all(b"version: n/a\n"),
        };
        let _ = match self.ec {
            Some(e) => writeln!(writer, "ec:      {:?}", e),
            None => writer.write_all(b"ec:      n/a\n"),
        };
        writeln!(writer, "grade:   {:.2?}", self.grade());
        writeln!(writer, "finder_grades:       {:.2?}", self.finder_grades);
        writeln!(writer, "timing_grades:       {:.2?}", self.timing_grades);
        writeln!(writer, "alignment_grade:     {:.2?}", self.alignment_grade);
        writeln!(writer, "version_info_grades: {:.2?}", self.version_info_grades);
        writeln!(writer, "format_info_grades:  {:.2?}", self.format_info_grades);
        writeln!(writer, "functional_grade:    {:.2?}", self.functional_grade);
        writeln!(writer, "decoding_grade:      {:.2?}", self.decoding_grade);
    }
}


//  ************************************************************
/// NW, SW, or SE representing the quadrant in which a finder pattern
/// is found or sought.
//  ************************************************************

#[derive(Clone, Copy, Debug)]
pub enum Corner {
    NW,
    SW,
    NE,
}


//  ************************************************************
/// Matrix of GrayScale pixels
///
/// Each pixel in `data` represents the luma (lightness)
/// of the corresponding pixel in the original image.
///
/// The field `threshold_luma` can be set independently,
/// and indicates the threshold between light and dark modules
/// to be used in the `get...` etc. methods.
//  ************************************************************

pub struct GrayMatrix {
    width: usize,
    height: usize,
    data: Vec<f64>,
    min_luma: f64,
    max_luma: f64,
    avg_luma: f64,
    threshold_luma: f64,
}

impl GrayMatrix {
    pub fn new<T: RGBAImage>(rgba_image: &T) -> Result<Self, String> {
        let width = rgba_image.width() as usize;
        let height = rgba_image.height() as usize;
        let n = width * height;
        debug!("GrayMatrix::new: w={} h={} n={}", width, height, n);
        let mut data = Vec::with_capacity(n);
        let mut min_luma = 1.0;
        let mut max_luma = 0.0;
        let mut sum = 0.0;
        for x in 0..width {
            for y in 0..height {
                let luma = luma_from_rgba(rgba_image.get(x, y));
                sum += luma;
                if luma > max_luma {
                    max_luma = luma;
                }
                if luma < min_luma {
                    min_luma = luma;
                }
                data.push(luma);
            }
        }
        let threshold_delta_luma = 0.1;
        let delta_luma = max_luma - min_luma;
        if delta_luma < threshold_delta_luma {
            return Err(format!(
                "Too little contrast. Luma difference {:.2} found; expected at least {:.2}",
                delta_luma, threshold_delta_luma
            ));
        }
        let avg_luma = sum / (n as f64);
        let mid = (min_luma + max_luma) / 2.0;
        let threshold_luma = mid;
        debug!("GrayMatrix::new: min={:.3} max={:.3} mid={:.3} avg={:.3}", min_luma, max_luma, mid, avg_luma,);
        Ok(GrayMatrix { width, height, data, min_luma, max_luma, avg_luma, threshold_luma })
    }

    pub fn get_min_max_avg_luma(&self) -> (f64, f64, f64) {
        (self.min_luma, self.max_luma, self.avg_luma)
    }

    pub fn get(&self, x: usize, y: usize) -> f64 {
        self.data[x * self.height + y]
    }

    pub fn is_dark(&self, x: usize, y: usize) -> bool {
        self.get(x, y) <= self.threshold_luma
    }

    pub fn lightness(&self, x: i32, y: i32, w: i32, h: i32) -> f64 {
        assert!(w > 0, "lightness: width must be positive");
        assert!(h > 0, "lightness: height must be positive");
        let mut l = 0.0;
        for i in x..x + w {
            if (i < 0) || (i >= self.width as i32) {
                l += f64::from(h);
            } else {
                let mut ii = i as usize * self.height + i32::max(y, 0) as usize;
                for j in y..y + h {
                    if (j < 0) || (j >= self.height as i32) {
                        l += 1.0;
                    } else {
                        // we do not like unsafe code, but this gives a nice performance boost
                        // and we have just verified the bounds above
                        //
                        // l += self.data[ii];
                        //
                        unsafe {
                            l += self.data.get_unchecked(ii);
                        }
                        ii += 1;
                    }
                }
            }
        }
        let raw = l / f64::from(w * h);
        let threshold = self.threshold_luma;
        raw - threshold
    }

    pub fn is_light(&self, x1: f64, y1: f64, x2: f64, y2: f64) -> bool {
        let l = self.lightness(x1.round() as i32, y1.round() as i32, (x2 - x1).round() as i32, (y2 - y1).round() as i32);
        l > 0.0
    }

    /// Evaluate a finder candidate at the specified location in this `GrayMatrix`

    pub fn eval_candidate(
        &self,
        center_x: usize,
        center_y: usize,
        dimension: usize,
        difficulty: usize,
        aggressive: bool,
    ) -> Option<FinderCandidate> {
        let dd = dimension as i32;
        let xx = center_x as i32 - dd / 2;
        let yy = center_y as i32 - dd / 2;
        let mut l1 = 0.0;
        let mut l2 = 0.0;
        let mut l3 = 0.0;
        let mut l4 = 0.0;
        let mut c = 0.0;

        // inner 3x3 dark center
        for i in -1..=1 {
            for j in -1..=1 {
                let light = self.lightness(xx + i * dd, yy + j * dd, dd, dd);
                l1 += light;
                if light <= 0.0 {
                    c += 1.0
                }
            }
        }

        // intermediary 5x5 light border
        for k in -2..2 {
            for (i, j) in [(k, -2), (2, k), (-k, 2), (-2, -k)].iter() {
                let light = self.lightness(xx + i * dd, yy + j * dd, dd, dd);
                l2 += light;
                if light > 0.0 {
                    c += 1.0
                }
            }
        }

        // outer 7x7 dark border
        for k in -3..3 {
            for (i, j) in [(k, -3), (3, k), (-k, 3), (-3, -k)].iter() {
                let light = self.lightness(xx + i * dd, yy + j * dd, dd, dd);
                l3 += light;
                if light <= 0.0 {
                    c += 1.0
                }
            }
        }

        // sourrounding 9x9 border
        for k in -4..4 {
            for (i, j) in [(k, -4), (4, k), (-k, 4), (-4, -k)].iter() {
                let light = self.lightness(xx + i * dd, yy + j * dd, dd, dd);
                l4 += light;
                if light > 0.0 {
                    c += 1.0
                }
            }
        }

        // According to ISO 18004, the grade is 4 if all modules have the correct reflectance,
        // and one is subtracted for each wrongly colored module.
        let mut grade = 4.0 + c - 81.0;
        if grade < 0.0 {
            grade = 0.0;
        }

        // To differentiate between finder patterns with the same grade according to ISO,
        // we add two parts to the grade (each accounting for up to almost 0.5):
        // - the fraction of the modules having the correct color
        // - the combined color value for each area of the finder
        let q = (-l1 / 9.0 + l2 / 16.0 - l3 / 24.0 + l4 / 32.0) / 4.0;
        let quality = c / (81.0 + 1.0) / 2.0 + q / 2.0;
        insane!("GrayMatrix::eval_candidate: x={} y={} d={} grade0={:.3} c={:.3} c/82={:.3} l1/9={:.3} l2/16={:.3} l3/24={:.3} l4/32={:.3} q={:.3} quality={:.3} grade={:.3}",
                    center_x,
                    center_y,
                    dimension,
                    grade,
                    c,
                    c/(81.0 + 1.0),
                    l1 / 9.0,
                    l2 / 16.0,
                    l3 / 24.0,
                    l4 / 32.0,
                    q,
                    quality,
                    grade + quality);
        grade += quality;
        let g_threshold = if aggressive { 0.2 } else { 2.0 };
        if grade >= g_threshold {
            Some(FinderCandidate { center_x, center_y, dimension, grade, difficulty })
        } else {
            None
        }
    }

    #[cfg(debug_assertions)]
    pub fn log_matrix(&self) {
        log!("GrayMatrix:");
        for y in 0..self.height as i32 {
            let mut s = String::with_capacity(self.width);
            for x in 0..self.width as i32 {
                let v = self.lightness(x, y, 1, 1);
                if v <= -0.5 {
                    s.push('@');
                } else if v <= 0.0 {
                    s.push('*');
                } else if v <= 0.5 {
                    s.push(':');
                } else {
                    s.push('.');
                }
            }
            log!("GrayMatrix[{:4}] {}", y, s);
        }
    }

    pub fn log_stats(&self) {
        let mut n_sd = 0;
        let mut n_rd = 0;
        let mut n_sl = 0;
        let mut n_rl = 0;
        for y in 0..self.height as i32 {
            for x in 0..self.width as i32 {
                let v = self.lightness(x, y, 1, 1);
                if v <= -0.2 {
                    n_rd += 1;
                } else if v <= 0.0 {
                    n_sd += 1;
                } else if v <= 0.2 {
                    n_sl += 1;
                } else {
                    n_rl += 1;
                }
            }
        }
        let pct_light = (100.0 * f64::from(n_rl + n_sl)) / ((self.width * self.height) as f64);
        log!("GrayMatrix: n={}, really_dark={} somewhat_dark={} somewhat_light={} really_light={} dark={} light={} L:D = {:.1}% : {:.1}%",
             self.width*self.height, n_rd, n_sd, n_sl, n_rl, n_rd+n_sd, n_rl+n_sl, pct_light, 100.0-pct_light);
    }
}


//  ************************************************************
/// Luma ("lightness") from a RGBA color
///
/// luma represents the brightness in an image
/// (See <https://en.wikipedia.org/wiki/Luma_(video)>)
///
/// The parameters `r`, `g`, `b`, `a` are `u8` values in range [0..255].
///
/// The result in an `f64` in the range [0..1(
//  ************************************************************

fn luma_from_rgba((r, g, b, a): (u8, u8, u8, u8)) -> f64 {
    if a == 0 {
        return 1.0;
    }
    let r = f64::from(r);
    let g = f64::from(g);
    let b = f64::from(b);
    let a = f64::from(a);

    a * 1.0 / 255.0 * (0.2162 / 255.0 * r + 0.7152 / 255.0 * g + 0.0722 / 255.0 * b)

    // Above we used REC 709.
    // An alternative would be REC 601:
    // https://en.wikipedia.org/wiki/CCIR_601
    // a * 1.0 / 255.0 * (0.30 / 255.0 * r + 0.59 / 255.0 * g + 0.11 / 255.0 * b)
}


//  ************************************************************
/// Matrix counting how many times a pixel has been hit by a 1:1:3:1:1 D:L:D:L:D sequence
///
/// Finder patterns are composed of (horizontal or vertical) sequences of
/// "1 Dark : 1 Light : 3 Dark : 1 Light : 1 Dark" modules.
///
/// The `HitMatrix` records, for each (x, y) the number of times
/// such a sequence has been found, centered on the (x, y).
///
/// We also record the minimal and maximal `3s`,
/// ie. the "lenght" of the middle "3 Dark" sequence.
//  ************************************************************

pub struct HitMatrix {
    width: usize,
    height: usize,
    hits: Vec<usize>,
    min3s: Vec<usize>,
    max3s: Vec<usize>,
    hiscore: usize,
    aggressive: bool,
}

impl HitMatrix {
    pub fn new(width: usize, height: usize, aggressive: bool) -> Self {
        let hits = vec![0usize; width * height];
        let min3s = vec![usize::max_value(); width * height];
        let max3s = vec![0usize; width * height];
        let hiscore = 0;
        HitMatrix { width, height, hits, min3s, max3s, hiscore, aggressive }
    }

    pub fn with_gray_matrix(gray_matrix: &GrayMatrix, aggressive: bool) -> Self {
        let mut hit_matrix = HitMatrix::new(gray_matrix.width, gray_matrix.height, aggressive);
        let (min, max) = qr::min_max_module_pixels(gray_matrix.width, gray_matrix.height);
        debug!("HitMatrix::with_gray_matrix: min_module_pixels={}, max_module_pixels={}", min, max);
        let mut pat = FinderHelper::new(min, max, aggressive);
        for rc in [true, false].iter() {
            // same operation for Row and Column
            let ii = if *rc { gray_matrix.width } else { gray_matrix.height };
            let jj = if *rc { gray_matrix.height } else { gray_matrix.width };
            for i in 0..ii {
                pat.reset();
                let mut counting_dark = false;
                let mut n = 0;
                for j in 0..jj {
                    if (i > ii / 2) && (j > jj / 2) {
                        break;
                    }
                    let is_dark = if *rc { gray_matrix.is_dark(i, j) } else { gray_matrix.is_dark(j, i) };
                    if counting_dark == is_dark {
                        n += 1;
                    } else {
                        if let Some((start, three)) = pat.push(counting_dark, n) {
                            insane!("HitMatrix::with_gray_matrix: hit: rc={} i={} j={} start={} three={}", rc, i, j, start, three);
                            if *rc {
                                hit_matrix.hit(i, j - start, 1, three)
                            } else {
                                hit_matrix.hit(j - start, i, three, 1)
                            };
                        };
                        counting_dark = !counting_dark;
                        n = 1;
                    }
                }
                if let Some((start, three)) = pat.push(counting_dark, n) {
                    insane!("HitMatrix::with_gray_matrix: hitlast: rc={} i={} j=END start={} three={}", rc, i, start, three);
                    if *rc {
                        hit_matrix.hit(i, jj - start, 1, three)
                    } else {
                        hit_matrix.hit(jj - start, i, three, 1)
                    };
                }
            }
        }
        hit_matrix
    }

    fn hit_one(&mut self, x: usize, y: usize, dim3s: usize) {
        let idx = x + y * self.width;
        let h = self.hits[idx].saturating_add(1);
        self.hits[idx] = h;
        if h > self.hiscore {
            self.hiscore = h;
        }
        if dim3s < self.min3s[idx] {
            self.min3s[idx] = dim3s;
        }
        if dim3s > self.max3s[idx] {
            self.max3s[idx] = dim3s;
        }
    }

    pub fn hit(&mut self, x1: usize, y1: usize, width3: usize, height3: usize) {
        trace!("GrayMatrix::hit w={} h={} x1={} y1={} w3={} h3={}", self.width, self.height, x1, y1, width3, height3);
        let dim3s = usize::max(width3, height3);

        // Hit the center pixel
        let x = usize::min(self.width - 1, x1 + width3 / 2);
        let y = usize::min(self.height - 1, y1 + height3 / 2);
        self.hit_one(x, y, dim3s);
        if width3 % 2 == 1 {
            let x = usize::min(self.width - 1, x1 + width3 / 2 + 1);
            let y = usize::min(self.height - 1, y1 + height3 / 2 + 1);
            self.hit_one(x, y, dim3s);
        }
        // Hit the pixels in the center third
        let xx1 = usize::min(self.width - 1, x1 + width3 / 3);
        let yy1 = usize::min(self.height - 1, y1 + height3 / 3);
        let xx2 = usize::min(self.width - 1, x1 + (2 * width3) / 3);
        let yy2 = usize::min(self.height - 1, y1 + (2 * height3) / 3);
        for x in xx1..=xx2 {
            for y in yy1..=yy2 {
                self.hit_one(x, y, dim3s);
            }
        }

        // Hit pixels around the actual match
        // (useful if the finder pattern is damaged)
        let footprint = if self.aggressive { 2 } else { 1 };
        for f in 0..=footprint {
            let xx1 = x1.saturating_sub(f);
            let yy1 = y1.saturating_sub(f);
            let xx2 = usize::min(self.width - 1, x1 + width3 - 1 + f);
            let yy2 = usize::min(self.height - 1, y1 + height3 - 1 + f);
            for x in xx1..=xx2 {
                for y in yy1..=yy2 {
                    self.hit_one(x, y, dim3s);
                }
            }
        }
    }

    pub fn idx(&self, i: usize) -> usize {
        self.hits[i]
    }

    pub fn get(&self, x: usize, y: usize) -> usize {
        self.hits[x + y * self.width]
    }

    pub fn min3(&self, x: usize, y: usize) -> usize {
        self.min3s[x + y * self.width]
    }

    pub fn max3(&self, x: usize, y: usize) -> usize {
        self.max3s[x + y * self.width]
    }
    pub fn hiscore(&self) -> usize {
        self.hiscore
    }

    pub fn log_stats(&self) {
        let mut n = 0;
        let mut hit_count = vec![0; self.hiscore];
        for i in 0..self.width {
            for j in 0..self.height {
                let c = self.get(i, j);
                if c > 0 {
                    n += 1;
                    hit_count[c - 1] += 1;
                }
            }
        }
        let pct_hit = 100.0 * f64::from(n) / (self.width as f64) / (self.height as f64);

        log!(
            "HitMatrix: width={} height={} size={} hiscore={} pixels_hit={}={:.2}%",
            self.width,
            self.height,
            self.width * self.height,
            self.hiscore,
            n,
            pct_hit,
        );
        let mut c = 0;
        let mut s = self.hiscore;
        while s > 0 {
            let h = hit_count[s - 1];
            c += h;
            log!("HitMatrix: score={:3} count={:5} cumul_count={:5}", s, h, c);
            s -= 1;
        }
    }


    #[cfg(debug_assertions)]
    pub fn log_matrix(&self) {
        for j in 0..self.height {
            let mut s = String::new();
            for i in 0..self.width {
                let h = self.get(i, j);
                if h == 0 {
                    s.push_str("   ")
                } else {
                    s.push_str(&format!("{:3}", h));
                }
            }
            log!("HIT_[{:4}]: {}", j, s);
        }

        for j in 0..self.height {
            let mut s = String::new();
            for i in 0..self.width {
                let h = self.min3(i, j);
                if h == usize::max_value() {
                    s.push_str("   ")
                } else {
                    s.push_str(&format!("{:3}", h));
                }
            }
            log!("MIN3[{:4}]: {}", j, s);
        }

        for j in 0..self.height {
            let mut s = String::new();
            for i in 0..self.width {
                let h = self.max3(i, j);
                if h == 0 {
                    s.push_str("   ")
                } else {
                    s.push_str(&format!("{:3}", h));
                }
            }
            log!("MAX3[{:4}]: {}", j, s);
        }
    }
}


//  ************************************************************
/// Represents a pixel at (`x`, `y`) hit `pri` times
//  ************************************************************

#[derive(Clone, Copy, Debug)]
pub struct HitItem {
    x: usize,
    y: usize,
    pri: usize,
}

impl PriorityItem for HitItem {
    fn get_priority(&self) -> usize {
        self.pri
    }
}


//  ************************************************************
/// Trait for an item which has an `usize` priority
//  ************************************************************

pub trait PriorityItem {
    fn get_priority(&self) -> usize;
}


//  ************************************************************
/// A queue of items represented by the trait `PriorityItem`.
///
/// When the `BoundedPriorityQueue` is created, the
/// `min_pri` and `max_pri` values are specified.
///
/// The `BoundedPriorityQueue` will have terrible runtime
/// and in particular memory performance,
/// unless `max_pri` is small; say below 100.
///
/// The queue will hold `PriorityItem`s with a priority between
/// `min_pri` and `max_pri`.
///
/// When `truncate` is called, the `BoundedPriorityQueue` will
/// be pruned to the specified number of items,
/// and it will henceforth not be possible to add items
/// with a lower priority, than the current lowest priority.
//  ************************************************************

pub struct BoundedPriorityQueue<Item: PriorityItem> {
    queue: Vec<VecDeque<Item>>,
    min_pri: usize,
    max_pri: usize,
    low_pri: usize,
}

impl<Item: PriorityItem> BoundedPriorityQueue<Item> {
    fn new(min_pri: usize, max_pri: usize, capacity: usize) -> Self {
        let mut queue = Vec::new();
        for _ in min_pri..=max_pri {
            queue.push(VecDeque::with_capacity(capacity));
        }
        BoundedPriorityQueue { queue, min_pri, max_pri, low_pri: min_pri }
    }

    fn push_back(&mut self, item: Item) {
        let p = item.get_priority();
        if p >= self.low_pri {
            let vd = &mut self.queue[p - self.min_pri];
            if vd.len() < vd.capacity() {
                vd.push_back(item);
            }
        }
    }

    fn pop_front(&mut self) -> Option<Item> {
        let mut pri = self.max_pri + 1;
        while pri > self.low_pri {
            pri -= 1;
            if let Some(item) = self.queue[pri - self.min_pri].pop_front() {
                return Some(item);
            }
        }
        None
    }

    fn truncate(&mut self, len: usize) {
        let mut n = 0;
        let mut pri = self.max_pri + 1;
        while (n < len) && (pri > self.low_pri) {
            pri -= 1;
            n += self.queue[pri - self.min_pri].len();
        }
        if self.low_pri != pri {
            trace!("BoundedPriorityQueue::truncate from {} to {}", self.low_pri, pri);
            for p in self.low_pri..pri {
                self.queue[p - self.min_pri].truncate(0);
                self.queue[p - self.min_pri].shrink_to_fit();
            }
        }
        self.low_pri = pri;
    }

    fn log_stats(&self) {
        let mut pri = self.max_pri + 1;
        while pri > self.low_pri {
            pri -= 1;
            log!("BoundedPriorityQueue: pri={} len={}", pri, self.queue[pri - self.min_pri].len());
        }
    }
}


//  ************************************************************
/// Iterator over a hit matrix
///
/// Create a `BoundedPriorityQueue` over a `HitMatrix`,
/// and return (x, y, min3, max3) tuples
/// in priority order.
//  ************************************************************

pub struct HitsIterator<'a> {
    hit_matrix: &'a HitMatrix,
    pri_queue: BoundedPriorityQueue<HitItem>,
}

impl<'a> HitsIterator<'a> {
    pub fn new(hit_matrix: &'a HitMatrix, max_hits: usize, corner: Corner) -> Self {
        let max_score = hit_matrix.hiscore;
        let min_score = max_score / 2;
        debug!("HitsIterator::new: max_hits={} corner={:?} max_score={} min_score={}", max_hits, corner, max_score, min_score);
        let mut pri_queue = BoundedPriorityQueue::new(min_score, max_score, max_hits);
        let (w, h) = (hit_matrix.width, hit_matrix.height);
        let (x0, y0, x1, y1, rev_x, rev_y) = match corner {
            Corner::NW => (0, 0, w / 2 - 1, h / 2 - 1, false, false),
            Corner::SW => (0, h - 1, w / 2 - 1, h / 2, false, true),
            Corner::NE => (w - 1, 0, w / 2, h / 2 - 1, true, false),
        };
        let w = hit_matrix.width;
        let mut y = y0;
        loop {
            let yi = w * y;
            let mut x = x0;
            loop {
                let score = hit_matrix.idx(yi + x);
                if score >= min_score {
                    pri_queue.push_back(HitItem { x, y, pri: score });
                }
                if x == x1 {
                    break;
                }
                if rev_x {
                    x -= 1;
                } else {
                    x += 1
                }
            }
            pri_queue.truncate(max_hits);
            if y == y1 {
                break;
            }
            if rev_y {
                y -= 1;
            } else {
                y += 1
            }
        }
        if log_enabled!(2) {
            pri_queue.log_stats();
        }
        HitsIterator { hit_matrix, pri_queue }
    }
}

impl<'a> Iterator for HitsIterator<'a> {
    type Item = (usize, usize, usize, usize);
    fn next(&mut self) -> Option<Self::Item> {
        let ii = self.pri_queue.pop_front();
        trace!("HitsIterator::next {:?}", ii);
        if let Some(HitItem { x, y, pri: _pri }) = ii {
            Some((x, y, self.hit_matrix.min3(x, y), self.hit_matrix.max3(x, y)))
        } else {
            trace!("HitsIterator::next done");
            None
        }
    }
}


//  ************************************************************
/// A possible Finder Pattern
//  ************************************************************

#[derive(Clone, Copy, Debug)]
pub struct FinderCandidate {
    pub center_x: usize,
    pub center_y: usize,
    pub dimension: usize,
    pub grade: f64,
    pub difficulty: usize,
}

impl Ord for FinderCandidate {
    fn cmp(&self, other: &FinderCandidate) -> Ordering {
        match self.grade.partial_cmp(&other.grade) {
            Some(ord) => match ord {
                Ordering::Equal => other.difficulty.cmp(&self.difficulty),
                ne => ne,
            },
            None => other.difficulty.cmp(&self.difficulty),
        }
    }
}

impl Eq for FinderCandidate {}

impl PartialOrd for FinderCandidate {
    fn partial_cmp(&self, other: &FinderCandidate) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for FinderCandidate {
    fn eq(&self, other: &FinderCandidate) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}


//  ************************************************************
/// Iterator returning potential Finder Patterns
//  ************************************************************

pub struct FinderCandidateIterator<'a, 'b> {
    gray_matrix: &'a GrayMatrix,
    hits_iterator: HitsIterator<'b>,
    aggressive: bool,
    buffer: BinaryHeap<FinderCandidate>,
    exhausted: bool,
    min_buffer: usize,
    max_count: usize,
    try_count: usize,
}

impl<'a, 'b> FinderCandidateIterator<'a, 'b> {
    pub fn new(gray_matrix: &'a GrayMatrix, hit_matrix: &'b HitMatrix, corner: Corner, aggressive: bool) -> Self {
        /*
         * We loop over the pixels in the `HitMatrix` which have most hits.
         *
         * The maximal number of finder candidates returned is `max_count`.
         *
         * We store the candidates in a binary heap,
         * and keep this filled with at least `min_buffer` finder candidates (if available).
         *
         * The binary heap is used so we can return "good" finder candidates
         * first, even if they appear with lower hit count in the `HitMatrix`.
         *
         * Note that every combination of three finder candidates returned here for three different corners,
         * are later tried as a potential finder area.
         *
         * The number of candidate triples to try are defined by the binomial coefficent:
         * binom(3, 3) = 1
         * binom(3, 4) = 4
         * binom(3, 5) = 10
         * binom(3, 6) = 20
         * binom(3, 7) = 35
         * binom(3, 8) = 56
         * binom(3, 9) = 84
         * binom(3, 10) = 120
         * binom(3, 11) = 165
         * binom(3, 12) = 220
         * binom(3, 13) = 286
         * binom(3, 14) = 364
         * binom(3, 15) = 455
         * binom(3, 16) = 560
         * binom(3, 17) = 680
         * binom(3, 18) = 816
         * binom(3, 19) = 969
         * binom(3, 20) = 1140
         * binom(3, 21) = 1330
         * binom(3, 22) = 1540
         * binom(3, 23) = 1771
         * binom(3, 24) = 2024
         * binom(3, 25) = 2300
         * binom(3, 26) = 2600
         * binom(3, 27) = 2925
         * binom(3, 28) = 3276
         * binom(3, 29) = 3654
         * binom(3, 30) = 4060
         * binom(3, 31) = 4495
         * binom(3, 32) = 4960
         * binom(3, 33) = 5456
         * binom(3, 34) = 5984
         * binom(3, 35) = 6545
         * binom(3, 36) = 7140
         * binom(3, 37) = 7770
         * binom(3, 38) = 8436
         * binom(3, 39) = 9139
         * binom(3, 40) = 9880
         * binom(3, 41) = 10660
         * binom(3, 42) = 11480
         * binom(3, 43) = 12341
         * binom(3, 44) = 13244
         * binom(3, 45) = 14190
         * binom(3, 46) = 15180
         * binom(3, 47) = 16215
         * binom(3, 48) = 17296
         * binom(3, 64) = 41664
         * binom(3, 80) = 82160
         * binom(3, 96) = 142880
         * binom(3, 112) = 227920
         * binom(3, 128) = 341376
         * binom(3, 144) = 487344
         * binom(3, 160) = 669920
         * binom(3, 176) = 893200
         * binom(3, 192) = 1161280
         * binom(3, 208) = 1478256
         * binom(3, 224) = 1848224
         * binom(3, 240) = 2275280
         * binom(3, 256) = 2763520
         * So be very careful before increasing `min_buffer` and `max_count`
         */

        let (min_buffer, max_count) = if aggressive { (16, 22) } else { (8, 16) };

        let hits_iterator = HitsIterator::new(hit_matrix, max_count, corner);
        let buffer = BinaryHeap::with_capacity(min_buffer + 9);
        FinderCandidateIterator {
            gray_matrix,
            hits_iterator,
            aggressive,
            buffer,
            exhausted: false,
            min_buffer,
            max_count,
            try_count: 0,
        }
    }

    pub fn fill(&mut self) -> bool {
        if self.try_count > self.max_count {
            trace!("FinderCandidateIterator::fill: quiescing after {} tries", self.try_count);
            return false;
        }
        if let Some((x, y, min3, max3)) = self.hits_iterator.next() {
            insane!("FinderCandidateIterator::fill: hit x={} y={} min3={} max3={}", x, y, min3, max3);
            let diff = if self.aggressive { 2 } else { 0 };
            let d0 = usize::max(1, (min3 / 3).saturating_sub(diff));
            let d1 = usize::max(1, max3 / 3 + diff);
            for d in d0..=d1 {
                self.try_count += 1;
                if let Some(candidate) = self.gray_matrix.eval_candidate(x, y, d, self.try_count, self.aggressive) {
                    insane!("FinderCandidateIterator::fill: good x={} y={} d={} grade={}", x, y, d, candidate.grade);
                    self.buffer.push(candidate);
                } else {
                    insane!("FinderCandidateIterator::fill: bad x={} y={} d={}", x, y, d);
                }
            }
            true
        } else {
            false
        }
    }
}

impl<'a, 'b> Iterator for FinderCandidateIterator<'a, 'b> {
    type Item = FinderCandidate;
    fn next(&mut self) -> Option<Self::Item> {
        trace!("FinderCandidateIterator::next buffer_len_before_fill={}", self.buffer.len());
        while (!self.exhausted) && (self.buffer.len() < self.min_buffer) {
            self.exhausted = !self.fill();
        }
        trace!("FinderCandidateIterator::next buffer_len_after_fill={}", self.buffer.len());
        let candidate = self.buffer.pop();
        trace!("FinderCandidateIterator::next {:?}", candidate);
        candidate
    }
}


//  ************************************************************
/// Helper structure to find 1:1:3:1:1 patterns to identify Finder Patterns
//  ************************************************************

#[derive(Debug)]
pub struct FinderHelper {
    pat: [usize; 6],
    idx: usize,
    cnt: usize,
    min: usize,
    max: usize,
    aggressive: bool,
}

impl FinderHelper {
    pub fn new(min: usize, max: usize, aggressive: bool) -> Self {
        FinderHelper { pat: [0usize; 6], idx: 0, cnt: 0, min: (7 - 2) * min, max: (7 + 2) * max, aggressive }
    }

    pub fn reset(&mut self) {
        self.pat = [0usize; 6];
        self.idx = 0;
        self.cnt = 0;
    }

    pub fn push(&mut self, dark: bool, n: usize) -> Option<(usize, usize)> {
        self.cnt += 1;
        if dark {
            self.push_dark(n)
        } else {
            self.push_light(n)
        }
    }

    pub fn push_light(&mut self, n: usize) -> Option<(usize, usize)> {
        assert!(self.idx % 2 == 0);
        self.pat[self.idx] = n;
        self.idx = (self.idx + 1) % 6;
        None
    }

    pub fn push_dark(&mut self, n: usize) -> Option<(usize, usize)> {
        assert!(self.idx % 2 == 1);
        self.pat[self.idx] = n;
        self.idx = (self.idx + 1) % 6;
        let i = self.idx;
        if self.cnt >= 5 {
            let mut len = 0;
            for j in 1..=5 {
                len += self.pat[(i + j) % 6];
            }
            let flen = len as f64;
            let delta = if self.aggressive { 0.7 } else { 0.5 };
            let one_low = (flen / 7.0 * delta).ceil() as usize;
            let one_high = (flen / 7.0 * (1.0 + delta)).floor() as usize;
            let three_low = (flen / 7.0 * (3.0 - delta)).ceil() as usize;
            let three_high = (flen / 7.0 * (3.0 + delta)).floor() as usize;
            if (len >= self.min)
                && (len <= self.max)
                && (self.pat[(i + 3) % 6] >= three_low)
                && (self.pat[(i + 3) % 6] <= three_high)
            {
                let mut good = true;
                for j in [1, 2, 4, 5].iter() {
                    if (self.pat[(i + j) % 6] < one_low) || (self.pat[(i + j) % 6] > one_high) {
                        good = false
                    }
                }
                if good {
                    insane!("FinderHelper::push_dark: i={} pat={:?}", i, self.pat);
                    let start = self.pat[(i + 3) % 6] + self.pat[(i + 4) % 6] + self.pat[(i + 5) % 6];
                    let three = self.pat[(i + 3) % 6];
                    return Some((start, three));
                }
            }
        };
        None
    }
}


//  ************************************************************
/// Aboslute difference between two `usize`s
//  ************************************************************

fn usize_abs_diff(a: usize, b: usize) -> usize {
    if a > b {
        a - b
    } else {
        b - a
    }
}


//  ************************************************************
/// Area with a finder pattern in NW, SW, NE corners
//  ************************************************************

#[derive(Debug)]
pub struct FinderArea {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    ms: f64,
    grades: [f64; 3],
    difficulty: usize,
}

//  ************************************************************
/// Iterator returning potential Finder Areas
//  ************************************************************

pub struct FinderAreaIterator<'nw, 'sw, 'ne> {
    nw_candidates: &'nw [FinderCandidate],
    sw_candidates: &'sw [FinderCandidate],
    ne_candidates: &'ne [FinderCandidate],
    nw_idx: usize,
    sw_idx: usize,
    ne_idx: usize,
    min_nw_idx: usize,
    min_sw_idx: usize,
    aggressive: bool,
    empty: bool,
}

impl<'nw, 'sw, 'ne> FinderAreaIterator<'nw, 'sw, 'ne> {
    pub fn new(
        nw_candidates: &'nw [FinderCandidate],
        sw_candidates: &'sw [FinderCandidate],
        ne_candidates: &'ne [FinderCandidate],
        corner: Corner,
        aggressive: bool,
    ) -> Self {
        let (nw_idx, sw_idx, ne_idx) = match corner {
            Corner::NW => (nw_candidates.len() - 1, 0, 0),
            Corner::SW => (0, sw_candidates.len() - 1, 0),
            Corner::NE => (0, 0, ne_candidates.len() - 1),
        };
        let (min_nw_idx, min_sw_idx) = (nw_idx, sw_idx);
        FinderAreaIterator {
            nw_candidates,
            sw_candidates,
            ne_candidates,
            nw_idx,
            sw_idx,
            ne_idx,
            min_nw_idx,
            min_sw_idx,
            aggressive,
            empty: false,
        }
    }
    pub fn good(nw: &FinderCandidate, sw: &FinderCandidate, ne: &FinderCandidate, aggressive: bool) -> Option<FinderArea> {
        let d_nw = nw.dimension;
        let d_sw = sw.dimension;
        let d_ne = ne.dimension;
        let d = (d_nw + d_sw + d_ne) / 3;
        let d_diff = if aggressive { d / 7 } else { d / 21 };
        if (usize_abs_diff(d_nw, d_sw) > d_diff) || (usize_abs_diff(d_nw, d_ne) > d_diff) || (usize_abs_diff(d_sw, d_ne) > d_diff) {
            insane!("FinderAreaIterator::good: incompatible dimensions {} {} {}; d_diff={}", d_nw, d_sw, d_ne, d_diff);
            return None;
        }
        let ms = d as f64;

        let nw_x = nw.center_x as f64;
        let nw_y = nw.center_y as f64;
        let ne_x = ne.center_x as f64;
        let ne_y = ne.center_y as f64;
        let sw_x = sw.center_x as f64;
        let sw_y = sw.center_y as f64;

        insane!(
            "FinderAreaIterator::good: NW({:.3}, {:.3}) SW({:.3}, {:.3}) NE({:.3}, {:.3}) MS({:.3})",
            nw_x,
            nw_y,
            sw_x,
            sw_y,
            ne_x,
            ne_y,
            ms
        );
        let unaligned = if aggressive { ms } else { ms / 3.0 };
        if (nw_x - sw_x).abs() > unaligned {
            insane!("FinderAreaIterator::good: unaligned x");
            return None;
        }
        if (nw_y - ne_y).abs() > unaligned {
            insane!("FinderAreaIterator::good: unaligned y");
            return None;
        }

        let w = ne_x - nw_x;
        let h = sw_y - nw_y;
        insane!("FinderAreaIterator::good: ms={:3} w={:.3} h={:.3}", ms, w, h);

        if (w - h).abs() > unaligned {
            insane!("FinderAreaIterator::good: too skew");
            return None;
        }

        if w / ms + 7.0 < (qr::MODULES_MIN as f64) - 0.5 {
            insane!("FinderAreaIterator::good: too narrow");
            return None;
        }
        if h / ms + 7.0 < (qr::MODULES_MIN as f64) - 0.5 {
            insane!("FinderAreaIterator::good: too low");
            return None;
        }
        if w / ms + 7.0 > (qr::MODULES_MAX as f64) + 0.5 {
            insane!("FinderAreaIterator::good: too wide");
            return None;
        }
        if h / ms + 7.0 > (qr::MODULES_MAX as f64) + 0.5 {
            insane!("FinderAreaIterator::good: too high");
            return None;
        }

        let grades = [nw.grade, sw.grade, ne.grade];
        let difficulty = usize::max(nw.difficulty, usize::max(sw.difficulty, ne.difficulty));

        let a = FinderArea {
            x: (nw_x + sw_x) / 2.0 - 3.0 * ms - (ms - 1.0) / 2.0,
            y: (nw_y + ne_y) / 2.0 - 3.0 * ms - (ms - 1.0) / 2.0,
            w: w + 7.0 * ms,
            h: h + 7.0 * ms,
            ms,
            grades,
            difficulty,
        };
        insane!("FinderAreaIterator::good: found {:?}", a);
        Some(a)
    }
}


impl<'nw, 'sw, 'ne> Iterator for FinderAreaIterator<'nw, 'sw, 'ne> {
    type Item = FinderArea;
    fn next(&mut self) -> Option<Self::Item> {
        if self.empty {
            return None;
        }
        loop {
            let a = FinderAreaIterator::good(
                &self.nw_candidates[self.nw_idx],
                &self.sw_candidates[self.sw_idx],
                &self.ne_candidates[self.ne_idx],
                self.aggressive,
            );
            if self.nw_idx < self.nw_candidates.len() - 1 {
                self.nw_idx += 1;
            } else {
                self.nw_idx = self.min_nw_idx;
                if self.sw_idx < self.sw_candidates.len() - 1 {
                    self.sw_idx += 1;
                } else {
                    self.sw_idx = self.min_sw_idx;
                    if self.ne_idx < self.ne_candidates.len() - 1 {
                        self.ne_idx += 1;
                    } else {
                        self.empty = true;
                        #[allow(clippy::question_mark)]
                        // replace_it_with: `a?;`
                        // but we cannot do that, as a is used again below!
                        {
                            if a.is_none() {
                                return None;
                            }
                        }
                    }
                }
            }
            if a.is_some() {
                return a;
            }
        }
    }
}


//  ************************************************************
/// Area with a matching alignment and timing patterns
//  ************************************************************

#[derive(Debug)]
pub struct SymbolArea {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    ms: f64,
    version: u8,
    ec: ErrorCorrectionLevel,
    mask: u8,
    finder_grades: [f64; 3],
    timing_grades: [f64; 2],
    alignment_grade: f64,
    version_info_grades: [f64; 2],
    format_info_grades: [f64; 2],
    difficulty: usize,
}

impl SymbolArea {
    pub fn grade(&self) -> f64 {
        let mut grade = 4.0;
        for i in 0..3 {
            let g = self.finder_grades[i];
            if g < grade {
                grade = g;
            }
        }
        for i in 0..2 {
            let g = self.timing_grades[i];
            if g < grade {
                grade = g;
            }
        }
        let g = self.alignment_grade;
        if g < grade {
            grade = g;
        }
        let g = (self.version_info_grades[0] + self.version_info_grades[1]) / 2.0;
        if g < grade {
            grade = g;
        }
        let g = (self.format_info_grades[0] + self.format_info_grades[1]) / 2.0;
        if g < grade {
            grade = g;
        }
        grade
    }
}


//  ************************************************************
/// Iterator returning potential aligned and timed areas
//  ************************************************************

pub struct SymbolAreaIterator<'a, 'b> {
    gray_matrix: &'a GrayMatrix,
    finder_area: &'b FinderArea,
    aggressive: bool,
    version_candidates: Vec<u8>,
    version_idx: usize,
}

impl<'a, 'b> SymbolAreaIterator<'a, 'b> {
    pub fn new(gray_matrix: &'a GrayMatrix, finder_area: &'b FinderArea, aggressive: bool) -> Self {
        let delta: u8 = if aggressive { 2 } else { 0 };
        let mut version_candidates = Vec::with_capacity(1 + 2 * (delta as usize));
        let nm = (finder_area.w + finder_area.h) / 2.0 / finder_area.ms;
        let vf = f64::round((nm - 17.0) / 4.0);
        let v0 = if vf < f64::from(qr::VERSION_MIN) {
            qr::VERSION_MIN
        } else if vf > f64::from(qr::VERSION_MAX) {
            qr::VERSION_MAX
        } else {
            vf as u8
        };
        version_candidates.push(v0);
        for i in 1..=delta {
            if v0 >= qr::VERSION_MIN + i {
                version_candidates.push(v0 - i);
            };
            if v0 <= qr::VERSION_MAX - i {
                version_candidates.push(v0 + i);
            };
        }
        SymbolAreaIterator { gray_matrix, finder_area, aggressive, version_candidates, version_idx: 0 }
    }

    fn good(&mut self, version: u8) -> Option<SymbolArea> {
        let grade_limit = if self.aggressive { 0.2 } else { 1.0 };
        let tgh = self.eval_timing(version, true);
        let tgv = self.eval_timing(version, false);
        let timing_grades = [tgh, tgv];
        let alignment_grade = self.eval_alignment(version);
        insane!("SymbolAreaIterator::good: timing_grades={:?} alignment_grade={}", timing_grades, alignment_grade);
        if (tgh < grade_limit) || (tgv < grade_limit) || (alignment_grade < grade_limit) {
            return None;
        }
        let mut version_info_grades = [4.0; 2];
        if version >= 7 {
            let g_ne = self.eval_version(version, true);
            let g_sw = self.eval_version(version, false);
            version_info_grades = [g_ne, g_sw];
            insane!("SymbolAreaIterator::good: version_info_grades={:?}", version_info_grades);
            if self.aggressive {
                if (g_ne < grade_limit) && (g_sw < grade_limit) {
                    return None;
                }
            } else if (g_ne < grade_limit) || (g_sw < grade_limit) {
                return None;
            }
        }
        let ef = self.eval_format(version);
        insane!("SymbolAreaIterator::good: eval_format {:?}", ef);
        match ef {
            Err(_) => None,
            Ok((ec, mask, format_info_grades)) => {
                let fa = self.finder_area;
                let sa = SymbolArea {
                    x: fa.x,
                    y: fa.y,
                    w: fa.w,
                    h: fa.h,
                    ms: fa.ms,
                    version,
                    ec,
                    mask,
                    finder_grades: fa.grades,
                    timing_grades,
                    alignment_grade,
                    version_info_grades,
                    format_info_grades,
                    difficulty: fa.difficulty,
                };
                Some(sa)
            }
        }
    }

    fn eval_timing(&mut self, version: u8, horizontal: bool) -> f64 {
        let mut damage = [false; qr::MODULES_MAX - 8 - 8];
        let mut damage_count = 0;
        let n = qr::n_modules_from_version(version);
        let m = n - 8 - 8;
        let nm = n as f64;
        let fa = self.finder_area;
        let ms = if horizontal { fa.w / nm } else { fa.h / nm };
        let mut x = fa.x + ms * (if horizontal { 8.0 } else { 6.0 });
        let mut y = fa.y + ms * (if horizontal { 6.0 } else { 8.0 });
        for i in 0..m {
            let light = self.gray_matrix.is_light(x, y, x + ms, y + ms);
            if light == (i % 2 == 0) {
                damage[i] = true;
                damage_count += 1;
            }
            if horizontal {
                x += ms;
            } else {
                y += ms;
            }
        }
        insane!("SymbolAreaIterator::eval_timing: horizontal={} damage_count={}", horizontal, damage_count);

        #[cfg(debug_assertions)]
        {
            let mut s = String::with_capacity(m);
            for i in 0..m {
                if damage[i] {
                    s.push('?');
                } else {
                    s.push('.');
                }
            }
            insane!("SymbolAreaIterator::eval_timing: damages {}", s);
        }

        let q = 1.0 - (damage_count as f64) / (m as f64);
        if 100 * damage_count > 14 * m {
            return q;
        }
        let g = if 100 * damage_count > 11 * m {
            1.0 + q
        } else if 100 * damage_count > 7 * m {
            2.0 + q
        } else if damage_count > 0 {
            3.0 + q
        } else {
            4.0
        };
        for j in 0..m - 5 {
            let mut dc = 0;
            for i in j..j + 5 {
                if damage[i] {
                    dc += 1;
                }
            }
            if dc >= 2 {
                return q / f64::from(dc);
            }
        }
        g
    }

    fn eval_alignment(&mut self, version: u8) -> f64 {
        if version <= 1 {
            return 4.0;
        }
        let mut damage_count = 0;
        let n = qr::n_modules_from_version(version);
        let mut m = 0;
        let nm = n as f64;
        let fa = self.finder_area;
        let gm = self.gray_matrix;
        let msx = fa.w / nm;
        let msy = fa.h / nm;
        for (i, j) in qr::AlignmentPatternIterator::new(version) {
            m += 1;
            insane!("SymbolAreaIterator::eval_alignment: pattern i={} j={}", i, j);
            let x = fa.x + (i as f64) * msx;
            let y = fa.y + (j as f64) * msy;
            let light = gm.is_light(x, y, x + msx, y + msy);
            insane!(
                "SymbolAreaIterator::eval_alignment: center position x1={:.3} y1={:.3} x2={:.3} x2={:.3}  light={:.3}",
                x,
                y,
                x + msx,
                y + msy,
                light
            );
            if light {
                damage_count += 1;
            }
            for k in -1..1 {
                for (ii, jj) in [(k, -1), (1, k), (-k, 1), (-1, -k)].iter() {
                    let iii: f64 = (*ii).into();
                    let jjj: f64 = (*jj).into();
                    let xx: f64 = x + msx * iii;
                    let yy: f64 = y + msy * jjj;
                    let light = gm.is_light(xx, yy, xx + msx, yy + msy);
                    insane!(
                        "SymbolAreaIterator::eval_alignment: LIGHT1 BORDER x1={:.3} y1={:.3} x2={:.3} y2={:.3}  light={:.3}",
                        xx,
                        yy,
                        xx + msx,
                        yy + msy,
                        light
                    );
                    if !light {
                        damage_count += 1
                    }
                }
            }
            for k in -2..2 {
                for (ii, jj) in [(k, -2), (2, k), (-k, 2), (-2, -k)].iter() {
                    let iii: f64 = (*ii).into();
                    let jjj: f64 = (*jj).into();
                    let xx: f64 = x + msx * iii;
                    let yy: f64 = y + msy * jjj;
                    let light = gm.is_light(xx, yy, xx + msx, yy + msy);
                    insane!(
                        "SymbolAreaIterator::eval_alignment: DARK2 BORDER x1={:.3} y1={:.3} x2={:.3} y2={:.3}  light={:.3}",
                        xx,
                        yy,
                        xx + msx,
                        yy + msy,
                        light
                    );
                    if light {
                        damage_count += 1
                    }
                }
            }
            trace!("SymbolAreaIterator::eval_alignment: damage_count={}", damage_count);
        }
        let q = 1.0 - f64::from(damage_count) / (f64::from(m) * 5.0 * 5.0);
        if 100 * damage_count > 30 * m {
            return q;
        }
        if 100 * damage_count > 20 * m {
            1.0 + q
        } else if 100 * damage_count > 10 * m {
            2.0 + q
        } else if damage_count > 0 {
            3.0 + q
        } else {
            4.0
        }
    }

    fn eval_version(&mut self, version: u8, north_east: bool) -> f64 {
        let ref_pat = qr::version_info(version);
        let mut pat = 0;
        let mut factor = 1;

        let n = qr::n_modules_from_version(version);
        let nm = n as f64;
        let fa = self.finder_area;
        let gm = self.gray_matrix;
        let msx = fa.w / nm;
        let msy = fa.h / nm;

        for i in 0..qr::N_VERSION_BITS {
            let (a, b) = qr::version_bit_pos(i);
            let (i, j) = if north_east { (n - 11 + a, b) } else { (b, n - 11 + a) };
            let x = fa.x + (i as f64) * msx;
            let y = fa.y + (j as f64) * msy;
            if !gm.is_light(x, y, x + msx, y + msy) {
                pat += factor
            }
            factor *= 2;
        }
        let hamming_distance = (ref_pat ^ pat).count_ones();
        insane!("SymbolAreaIterator::eval_version: ref={:X?} pat={:X?} hamming_distance={}", ref_pat, pat, hamming_distance);
        if hamming_distance == 0 {
            4.0
        } else if hamming_distance > 3 {
            f64::from(hamming_distance) / (qr::N_VERSION_BITS as f64)
        } else {
            4.0 - f64::from(hamming_distance)
        }
    }

    fn eval_format(&mut self, version: u8) -> Result<(ErrorCorrectionLevel, u8, [f64; 2]), String> {
        let mut patterns = [0; 2];
        let n = qr::n_modules_from_version(version);
        let nm = n as f64;
        let fa = self.finder_area;
        let gm = self.gray_matrix;
        let msx = fa.w / nm;
        let msy = fa.h / nm;
        let mut factor = 1;
        for b in 0..qr::N_FORMAT_BITS {
            let fbp = qr::format_bit_positions(b, n);
            for p in 0..=1 {
                let (i, j) = fbp[p];
                let x = fa.x + (i as f64) * msx;
                let y = fa.y + (j as f64) * msy;
                if !gm.is_light(x, y, x + msx, y + msy) {
                    patterns[p] += factor
                }
            }
            factor *= 2;
        }
        insane!("SymbolAreaIterator::eval_format: pat0={:X?} pat1={:X?}", patterns[0], patterns[1]);
        let mut grades = [0.0, 0.0];
        let mut res = [None, None];
        for p in 0..=1 {
            'pat: for ec in
                [ErrorCorrectionLevel::L, ErrorCorrectionLevel::M, ErrorCorrectionLevel::Q, ErrorCorrectionLevel::H].iter()
            {
                for mask in 0..8 {
                    let ref_pat = qr::format_info(mask, *ec);
                    let hamming_distance = (ref_pat ^ patterns[p]).count_ones();
                    insane!(
                        "SymbolAreaIterator::eval_format: p={} ec={:?} mask={} ref={:X?} pat={:X?} hamming_distance={}",
                        p,
                        ec,
                        mask,
                        ref_pat,
                        patterns[p],
                        hamming_distance
                    );
                    if hamming_distance <= 3 {
                        grades[p] = 4.0 - f64::from(hamming_distance);
                        res[p] = Some((*ec, mask));
                        break 'pat;
                    }
                }
            }
        }
        match res {
            [None, Some((ec, mask))] => Ok((ec, mask, grades)),
            [Some((ec, mask)), None] => Ok((ec, mask, grades)),
            [Some((ec0, mask0)), Some((ec1, mask1))] => if ec0 != ec1 {
                Err("Conflicting format (ec) information".to_string())
            } else if mask0 != mask1 {
                Err("Conflicting format (mask) information".to_string())
            } else {
                Ok((ec0, mask0, grades))
            },
            [_, _] => Err("Unable to decode format (ec, mask) information".to_string()),
        }
    }
}

impl<'a, 'b> Iterator for SymbolAreaIterator<'a, 'b> {
    type Item = SymbolArea;
    fn next(&mut self) -> Option<Self::Item> {
        while self.version_idx < self.version_candidates.len() {
            let v = self.version_candidates[self.version_idx];
            self.version_idx += 1;
            let sa = self.good(v);
            if sa.is_some() {
                return sa;
            }
        }
        None
    }
}


//  ************************************************************
/// Get the raw data of the symbol as encoded in the snaked pattern
//  ************************************************************

fn get_snaked_data(symbol_area: &SymbolArea, gray_matrix: &GrayMatrix) -> Vec<u8> {
    let mask = symbol_area.mask;
    let version = symbol_area.version;
    let n = qr::n_codewords(version) as usize;
    let nm = qr::n_modules_from_version(version) as f64;
    let (x0, y0) = (symbol_area.x, symbol_area.y);
    let (dx, dy) = (symbol_area.w / nm, symbol_area.h / nm);
    let mut bs = BitSeq::new(n);
    let mut bits = 0;
    let mut rem_bits = 0;
    #[allow(clippy::explicit_counter_loop)]
    #[allow(clippy::needless_range_loop)]
    for (i, j) in qr::SnakeDataIterator::new(version) {
        insane!("get_snaked_data: i={} j={}", i, j);
        bits += 1;
        if bits <= 8 * n {
            let ii = i as f64;
            let jj = j as f64;
            let bit = !gray_matrix.is_light(x0 + ii * dx, y0 + jj * dy, x0 + (1.0 + ii) * dx, y0 + (1.0 + jj) * dy);
            bs.push_bit(bit != qr::mask(mask, i, j));
        } else {
            rem_bits += 1;
        }
    }
    if rem_bits != qr::n_remainder_bits(version) {
        error!("get_snaked_data: WRONG NUMBER OF REMAINDER BITS: GOT {}; expected {}", rem_bits, qr::n_remainder_bits(version));
        panic!("get_snaked_data: WRONG NUMBER OF REMAINDER BITS: GOT {}; expected {}", rem_bits, qr::n_remainder_bits(version));
    }
    bs.into()
}


//  ************************************************************
/// Correct errors and extract the raw data
//  ************************************************************

fn correct_errors(codewords: &[u8], ec_blocks: [qr::ECB; 2]) -> Result<(Vec<u8>, f64), String> {
    let n_codewords = ec_blocks[0].n * ec_blocks[0].c + ec_blocks[1].n * ec_blocks[1].c;
    if codewords.len() != n_codewords {
        error!("correct_errors: WRONG NUMBER OF CODEWORDS: GOT {}; expected {}", codewords.len(), n_codewords);
        panic!("correct_errors: WRONG NUMBER OF CODEWORDS: GOT {}; expected {}", codewords.len(), n_codewords);
    }
    let n_data_codewords = ec_blocks[0].n * ec_blocks[0].k + ec_blocks[1].n * ec_blocks[1].k;
    let mut data_codewords = Vec::with_capacity(n_data_codewords);
    let n_ec_codewords_block = ec_blocks[0].c - ec_blocks[0].k;
    let mut ec_codewords_block = vec![0u8; n_ec_codewords_block];
    let n_blocks = ec_blocks[0].n + ec_blocks[1].n;
    let ec_offset = n_data_codewords;
    let mut grade = 4.0;
    trace!("correct_errors: ec_blocks={:?}", ec_blocks);
    trace!(
        "correct_errors: n_codewords={} n_data_codewords={} n_ec_codewords_block={} n_blocks={} ec_offset={}",
        n_codewords,
        n_data_codewords,
        n_ec_codewords_block,
        n_blocks,
        ec_offset
    );
    for e in 0..=1 {
        let d0 = e * ec_blocks[0].k * ec_blocks[0].n;
        let b0 = e * ec_blocks[0].n;
        for b in 0..ec_blocks[e].n {
            for i in 0..ec_blocks[e].k {
                let db0 = if i > ec_blocks[0].k - 1 { 0 } else { b0 };
                insane!("correct_errors: data: e={} b0={} b={} i={} db0={} idx={}", e, b0, b, i, db0, i * n_blocks + (b + db0));
                data_codewords.push(codewords[i * n_blocks + (b + db0)]);
            }
            for i in 0..n_ec_codewords_block {
                insane!("correct_errors: ec: e={} b0={} b={} i={} idx={}", e, b0, b, i, ec_offset + i * n_blocks + (b + b0));
                ec_codewords_block[i] = codewords[ec_offset + i * n_blocks + (b + b0)];
            }
            let d = d0 + b * ec_blocks[e].k;
            match reed_solomon_decoder::correct(&mut data_codewords[d..d + ec_blocks[e].k], &ec_codewords_block) {
                Ok(g) => {
                    let g: f64 = g.into();
                    if g < grade {
                        grade = g
                    }
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
    }
    Ok((data_codewords, grade))
}


//  ************************************************************
/// Decode raw data into the contained payload
//  ************************************************************

fn decode_data(version: u8, ec: ErrorCorrectionLevel, raw_input: &[u8]) -> Result<(Mode, Vec<u8>), String> {
    trace!(
        "decode_data: version={} ec={:?} raw_input.len={} first_three_bytes=0b_{:08b}_{:08b}_{:08b}",
        version,
        ec,
        raw_input.len(),
        raw_input[0],
        raw_input[1],
        raw_input[2]
    );
    let bs = BitSeq::from(Vec::from(raw_input));
    let m = bs.get_bits(0, 4);
    let mode = match m {
        0b0100 => Mode::EightBit,
        0b0010 => Mode::AlphaNumeric,
        0b0001 => Mode::Numeric,
        _ => return Err(format!("Unsupported mode {:b}", m)),
    };
    let nc = qr::n_count_bits(version, mode) as usize;
    let payload_len = bs.get_bits(4, nc) as usize;
    let payload_len_max = qr::data_capacity(version, mode, ec);
    trace!("decode_data: mode={:?} payload_len={} payload_len_max={}", mode, payload_len, payload_len_max);
    if payload_len > (payload_len_max as usize) {
        return Err(format!(
            "Inconsistent data length. Code has lenght {} encoded, but {} is the maximum",
            payload_len, payload_len_max
        ));
    }
    let res = match mode {
        Mode::EightBit => decode_eight_bit(&bs, 4 + nc, payload_len),
        Mode::AlphaNumeric => decode_alpha_numeric(&bs, 4 + nc, payload_len),
        Mode::Numeric => decode_numeric(&bs, 4 + nc, payload_len),
    };
    Ok((mode, res))
}


//  ************************************************************
/// Decode raw data encoded in 8bit mode
//  ************************************************************

fn decode_eight_bit(bs: &BitSeq, index0: usize, len: usize) -> Vec<u8> {
    let mut res = Vec::with_capacity(len);
    for i in 0..len {
        res.push(bs.get_bits(index0 + i * 8, 8) as u8);
    }
    res
}


//  ************************************************************
/// Decode raw data encoded in alphanumeric mode
//  ************************************************************

fn decode_alpha_numeric(bs: &BitSeq, index0: usize, len: usize) -> Vec<u8> {
    let mut res = Vec::with_capacity(len);
    for i in 0..len / 2 {
        let x = bs.get_bits(index0 + i * 11, 11);
        res.push(qr::alnum_to_ascii((x / 45) as u8));
        res.push(qr::alnum_to_ascii((x % 45) as u8));
    }
    if len % 2 > 0 {
        let x = bs.get_bits(index0 + (len / 2) * 11, 6);
        res.push(qr::alnum_to_ascii(x as u8));
    }
    res
}


//  ************************************************************
/// Decode raw data encoded in numeric mode
//  ************************************************************

fn decode_numeric(bs: &BitSeq, index0: usize, len: usize) -> Vec<u8> {
    let mut res = Vec::with_capacity(len);
    for i in 0..len / 3 {
        let x = bs.get_bits(index0 + i * 10, 10);
        res.push(48 + (x / 100) as u8);
        res.push(48 + ((x % 100) / 10) as u8);
        res.push(48 + (x % 10) as u8);
    }
    if len % 3 == 1 {
        let x = bs.get_bits(index0 + (len / 3) * 10, 4);
        res.push(48 + x as u8);
    } else if len % 3 == 2 {
        let x = bs.get_bits(index0 + (len / 3) * 10, 7);
        res.push(48 + (x / 10) as u8);
        res.push(48 + (x % 10) as u8);
    }
    res
}
