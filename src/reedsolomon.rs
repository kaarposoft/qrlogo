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
//! Reed Solomon error correction algorithms
//  ************************************************************

use std::fmt;
use std::fmt::Debug;
use std::ops::{Add, AddAssign, Div, Index, IndexMut, Mul, MulAssign, Not, ShlAssign, Sub, SubAssign};

use super::logging;


//  ************************************************************
/// Reed Solomon decoding
///
/// References:
/// - <https://content.sakai.rutgers.edu/access/content/user/ak892/Reed-SolomonProjectReport.pdf>
/// - <https://crypto.stanford.edu/~mironov/cs359/massey.pdf>
//  ************************************************************

pub mod reed_solomon_decoder {
    use super::logging;
    use super::Poly;
    use super::G;

    //  ************************************************************
    /// Decode message `msg` with parity / error correction `parity`
    ///
    /// The `msg` passed in will be corrected if possible, and the `grade` 1-4 returned.
    /// If correction failed, an error message will be returned
    //  ************************************************************

    pub fn correct(msg: &mut [u8], parity: &[u8]) -> Result<u32, String> {
        let n_parity_bytes = parity.len();
        let n_error_correction_capability = n_parity_bytes / 2;
        trace!("ReedSolomonDecoder: msg.len={} + parity_len={} = {}", msg.len(), parity.len(), msg.len() + parity.len());
        match syndromes(msg, parity) {
            None => {
                debug!("ReedSolomonDecoder: no errors found");
                Ok(4)
            }
            Some(syndromes) => {
                trace!("ReedSolomonDecoder: syndromes={:?}", syndromes);
                let error_poly = berlekamp_massey(&syndromes, n_parity_bytes);
                let n_errors = error_poly.degree();
                insane!("ReedSolomonDecoder: n_parity_bytes={} n_errors={} error_poly={:?}", parity.len(), n_errors, error_poly);
                if n_errors > n_error_correction_capability {
                    return Err(format!(
                        "ReedSolomon failed: Too many errors. found {}; can only correct up to {}",
                        n_errors, n_error_correction_capability
                    ));
                }
                let n_remaining_error_correction_capability = n_error_correction_capability - n_errors;
                let grade = if n_remaining_error_correction_capability > 2 {
                    3
                } else if n_remaining_error_correction_capability > 1 {
                    2
                } else {
                    1
                };
                insane!("ReedSolomonDecoder: n_errors={} n_error_correction_capability={} n_remaining_error_correction_capability={} grade={}", n_errors, n_error_correction_capability, n_remaining_error_correction_capability, grade);

                let roots = error_poly.find_roots();
                let n_roots = roots.len();
                trace!("ReedSolomonDecoder: n_roots={} roots={:?}", n_roots, roots);
                if n_roots != n_errors {
                    return Err(format!("ReedSolomon failed: Wrong number of roots: got {}; expected {}", n_roots, n_errors));
                }

                let error_locators: Vec<G> = roots.iter().map(|r| r.log_inv()).collect();
                trace!("ReedSolomonDecoder: error_locators={:?}", error_locators);

                let mut error_eval_poly: Poly = &error_poly * &syndromes;
                error_eval_poly.truncate(parity.len() / 2 - 1);
                trace!("ReedSolomonDecoder: error_eval_poly={:?}", error_eval_poly);
                let error_values = forney(&roots, &error_poly, &error_eval_poly);
                trace!("ReedSolomonDecoder: error_values={:?}=0x{:2X?}", error_values, error_values);
                let error_locators: Vec<G> = roots.iter().map(|r| r.log_inv()).collect();
                trace!("ReedSolomonDecoder: error_locators={:?}", error_locators);
                for i in 0..error_values.len() {
                    let loc = error_locators[i].0 as usize;
                    if loc > msg.len() + parity.len() - 1 {
                        trace!("ReedSolomonDecoder: Correction loc={} is out of scope", loc);
                    } else {
                        let pos = msg.len() + parity.len() - loc - 1;
                        trace!(
                            "ReedSolomonDecoder: error[{:2}] loc={:3} pos={:3} value=0x{:02X}=0b_{:08b} in_data={}",
                            i,
                            loc,
                            pos,
                            error_values[i].0,
                            error_values[i].0,
                            pos < msg.len()
                        );
                        if pos < msg.len() {
                            msg[pos] ^= error_values[i].0;
                        }
                    }
                }

                Ok(grade)
            }
        }
    }


    //  ************************************************************
    /// Calculate the syndromes from `msg` with parity / error correction `parity`
    ///
    /// The syndromes represent the errors in the received message
    ///
    /// If no syndromes are found the message is perfect, and `None` is returned.
    /// Otherwise the syndromes are returned.
    //  ************************************************************

    fn syndromes(msg: &[u8], parity: &[u8]) -> Option<Poly> {
        let mut zero = true;
        let mut syn = Vec::with_capacity(parity.len());
        let n = parity.len() as u8;
        for j in 0..n {
            let exp = G(j).exp();
            let mut sum = G(0);
            for b in msg.iter() {
                sum = G(*b) + exp * sum;
            }
            for b in parity.iter() {
                sum = G(*b) + exp * sum;
            }
            if sum != G(0) {
                zero = false;
            }
            syn.push(sum);
        }
        if zero {
            None
        } else {
            Some(syn.into())
        }
    }


    //  ************************************************************
    /// Calculate connection polynomial using the Berlekamp-Massey algorithm
    ///
    /// The connection polynomial represents the difference between the original message and the
    /// received message.
    ///
    /// References:
    /// - <https://content.sakai.rutgers.edu/access/content/user/ak892/Reed-SolomonProjectReport.pdf>
    /// - <https://crypto.stanford.edu/~mironov/cs359/massey.pdf>
    //  ************************************************************

    pub fn berlekamp_massey(syndromes: &Poly, n_max: usize) -> Poly {
        let mut connection_poly = Poly::new_one(n_max);
        let mut prev_connection_poly = Poly::new_one(n_max);
        let mut n_errors = 0;
        let mut m = 1;
        let mut prev_d = G(1);
        for n in 0..n_max {
            let d = descrepancy(&connection_poly, syndromes, n_errors, n);
            insane!(
                "berlekamp_massey: n={} n_errors={} m={} d={:?} prev_d={:?} complex_branch={}",
                n,
                n_errors,
                m,
                d,
                prev_d,
                (d != G(0)) && (2 * n_errors <= n)
            );
            if d == G(0) {
                m += 1;
            } else if 2 * n_errors <= n {
                let tmp = connection_poly.clone();
                prev_connection_poly <<= m;
                prev_connection_poly *= d / prev_d;
                connection_poly -= &prev_connection_poly;
                prev_connection_poly = tmp;
                prev_d = d;
                n_errors = (n + 1) - n_errors;
                m = 1;
            } else {
                let mut tmp = prev_connection_poly.clone();
                tmp <<= m;
                tmp *= d / prev_d;
                connection_poly -= &tmp;
                m += 1;
            }
            insane!("berlekamp_massey: prev_connection_poly={:?}", prev_connection_poly);
            insane!("berlekamp_massey: connection_poly={:?}", connection_poly);
        }
        connection_poly.simplify();
        if n_errors != connection_poly.degree() {
            insane!("!!!!!!!!!!!!!!!! n_errors={} connection_poly.degree={}", n_errors, connection_poly.degree()); // TODO
            let dd = descrepancy(&connection_poly, syndromes, connection_poly.degree(), n_max - 1);
            if dd != G(0) {
                insane!("!!!!!!!!!!!!!!!! still descrepancy: {:?}", dd); // TODO
            }
        }
        connection_poly
    }


    //  ************************************************************
    /// Calculate the difference between the connection polynomial and the syndromes
    //  ************************************************************

    pub fn descrepancy(connection_poly: &Poly, syndromes: &Poly, n_errors: usize, n: usize) -> G {
        let mut d = syndromes[n];
        for i in 1..=n_errors {
            d += connection_poly[i] * syndromes[n - i];
        }
        d
    }


    //  ************************************************************
    /// Calculate the position of each error
    ///
    /// Given:
    /// - `roots`: the roots of the connection polonymial
    /// - `error_poly`: the difference between the connection poly and the syndromes
    /// - `error_eval_poly`: evaluating to the difference between the connection poly and the syndromes
    ///
    /// Return the errors at each position
    ///
    /// Reference:
    /// - <https://en.wikipedia.org/wiki/Forney_algorithm>
    //  ************************************************************

    pub fn forney(roots: &[G], error_poly: &Poly, error_eval_poly: &Poly) -> Vec<G> {
        let mut error_values = Vec::with_capacity(roots.len());
        let derivative = error_poly.derivative();
        trace!("ReedSolomon:forney: derivative={:?}", derivative);
        for &r in roots {
            let mut numerator = error_eval_poly.eval(r);
            let mut denominator = derivative.eval(r);
            let error_value = (!r) * (numerator / denominator);
            insane!("ReedSolomon:forney: r={:?} !r={:?} num={:?} den={:?} err={:?}", r, !r, numerator, denominator, error_value);
            error_values.push(error_value);
        }
        error_values
    }
}


//  ************************************************************
/// Reed Solomon encoder
//  ************************************************************

pub struct ReedSolomonEncoder {
    n_ec_bytes: usize,
    gen_poly: Poly,
}

impl ReedSolomonEncoder {
    //  ************************************************************
    pub fn new(n_ec_bytes: usize) -> Self {
        let gen_poly = Poly::generator(n_ec_bytes);
        ReedSolomonEncoder { n_ec_bytes, gen_poly }
    }

    //  ************************************************************
    pub fn encode(&self, msg: &[u8]) -> Vec<u8> {
        let n = self.n_ec_bytes;
        trace!("ReedSolomonEncoder::encode begin; n={}", n);
        let mut lfsr = [G(0); 68 + 1];

        for m in msg.iter() {
            let b = G(*m) + lfsr[n - 1];
            let mut j = n;
            while j > 1 {
                j -= 1;
                lfsr[j] = lfsr[j - 1] + (self.gen_poly[j] * b);
            }
            lfsr[0] = self.gen_poly[0u8] * b;
        }


        let mut parity = Vec::with_capacity(self.n_ec_bytes);
        let mut i = self.n_ec_bytes;
        while i > 0 {
            i -= 1;
            parity.push(lfsr[i].0);
        }
        trace!("ReedSolomonEncoder::encode done; n={}, parity={:?}", n, parity);
        parity
    }
}


//  ************************************************************
#[cfg(test)]
//  ************************************************************

mod encode_decode {
    use super::super::ErrorCorrectionLevel;
    use super::*;
    use prng::Rng;
    use qr;

    #[test]
    fn enc_dec_ok() {
        enc_dec(true)
    }

    #[test]
    fn enc_dec_err() {
        enc_dec(false)
    }

    fn enc_dec(ok: bool) {
        // If tests are failing, the loglevel can be increased to identify the cause
        // logging::set_loglevel(3);

        // Collect all the Error Correction Blocks used by QR Codes
        let mut ecb_vec = Vec::with_capacity(2 * 40 * 4);
        for version in qr::VERSION_MIN..=qr::VERSION_MAX {
            for &ec in [ErrorCorrectionLevel::L, ErrorCorrectionLevel::M, ErrorCorrectionLevel::Q, ErrorCorrectionLevel::H].iter() {
                let [ecb1, ecb2] = qr::ec_blocks(version, ec);
                if ecb1.c > 0 {
                    ecb_vec.push(ecb1);
                }
                if ecb2.c > 0 {
                    ecb_vec.push(ecb2);
                }
            }
        }

        // Remove duplicates, so we only test each Error Correction Block type once
        ecb_vec.sort_unstable_by_key(ecb_ord);
        ecb_vec.dedup_by_key(ecb_ord_mut);
        println!("{} Error Correction Blocks to test: {:?}", ecb_vec.len(), ecb_vec);

        for &seed in [1, 2, 3, 5, 7, 11, 13].iter() {
            let mut rng = Rng::new(seed);
            let mut prev_e = 7;
            let mut encoder = ReedSolomonEncoder::new(prev_e);
            for ecb in ecb_vec.iter() {
                let e = ecb.c - ecb.k;
                println!("\n\n========== TEST ECB ==========> seed={} c={} k={} e={} r={}", seed, ecb.c, ecb.k, e, ecb.r);
                if e != prev_e {
                    prev_e = e;
                    encoder = ReedSolomonEncoder::new(prev_e);
                }
                let mut data = rng.get_u8_vec(ecb.k);
                let mut parity = encoder.encode(&data);
                assert!(parity.len() == e, "wrong number of error correction bytes produced by encode");
                let (n_min, n_max) = if ok { (0, e / 2) } else { (e / 2 + 1, e - 1) };
                for n_introduced_errors in n_min..n_max {
                    println!(
                        "\n========== TEST ONE ==========> seed={} c={} k={} e={} r={}, n_introduced_errors={}",
                        seed, ecb.c, ecb.k, e, ecb.r, n_introduced_errors
                    );
                    let mut noisy_data = data.clone();
                    let mut noisy_parity = parity.clone();
                    let error_positions = rng.get_usize_unique_clamped_vec(n_introduced_errors, 0, ecb.c - 1);
                    for pos in error_positions {
                        let noise = rng.get_u8_clamped(1, 255);
                        trace!(
                            "========== introducing noise: pos={:3} noise=0x{:02X}=0b_{:08b} in_data={}",
                            pos,
                            noise,
                            noise,
                            pos < ecb.k
                        );
                        if pos < ecb.k {
                            noisy_data[pos] ^= noise;
                        } else {
                            noisy_parity[pos - ecb.k] ^= noise;
                        }
                    }
                    let res = reed_solomon_decoder::correct(&mut noisy_data, &noisy_parity);
                    println!(
                        "========== RESULT ==========> seed={} c={} k={} e={} r={}, n_introduced_errors={} res={:?}",
                        seed, ecb.c, ecb.k, e, ecb.r, n_introduced_errors, res
                    );
                    if ok {
                        match res {
                            Ok(grade) => {
                                if n_introduced_errors == 0 {
                                    assert!(grade == 4, "expected grade 4 when no errors where introduced");
                                } else {
                                    assert!(grade > 0, "expected grade>0 when only few errors where introduced");
                                }
                                assert!(data == noisy_data, "errors not corrected correctly");
                            }
                            Err(e) => panic!(e),
                        }
                    } else {
                        assert!(
                            res.is_err(),
                            "introduced {} errors; but only {} should have been correctable",
                            n_introduced_errors,
                            e / 2
                        );
                    }
                }
            }
        }
    }

    fn ecb_ord(ecb: &qr::ECB) -> (usize, usize, usize) {
        (ecb.c - ecb.k, ecb.k, ecb.r)
    }

    fn ecb_ord_mut(ecb: &mut qr::ECB) -> (usize, usize, usize) {
        (ecb.c - ecb.k, ecb.k, ecb.r)
    }

}

//  ************************************************************
/// Polynomial over finite field
//  ************************************************************

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Poly {
    c: Vec<G>,
}

//  ************************************************************
impl Poly {
    //  ************************************************************
    pub fn new(nbytes: usize) -> Self {
        let mut c = Vec::with_capacity(nbytes);
        for _ in 0..nbytes {
            c.push(G(0));
        }
        Poly { c }
    }

    //  ************************************************************
    pub fn new_one(nbytes: usize) -> Self {
        let mut c = Vec::with_capacity(nbytes);
        c.push(G(1));
        for _ in 0..nbytes - 1 {
            c.push(G(0));
        }
        Poly { c }
    }

    //  ************************************************************
    pub fn degree(&self) -> usize {
        let mut i = self.c.len();
        while i > 1 {
            i -= 1;
            if self.c[i] != G(0) {
                return i;
            }
        }
        0
    }

    //  ************************************************************
    pub fn simplify(&mut self) -> usize {
        let mut i = self.c.len();
        while i > 1 {
            i -= 1;
            if self.c[i] == G(0) {
                self.c.pop();
            } else {
                return i;
            }
        }
        0
    }

    //  ************************************************************
    pub fn truncate(&mut self, degree: usize) -> usize {
        self.c.truncate(degree + 1);
        self.simplify()
    }

    //  ************************************************************
    pub fn reverse(&mut self) {
        self.simplify();
        let n = self.c.len();
        for i in 0..n / 2 {
            self.c.swap(i, n - i - 1);
        }
    }

    //  ************************************************************
    pub fn mul_add(&mut self, factor: G, poly: &Poly) {
        debug_assert!(self.c.len() == poly.c.len());
        #[allow(clippy::needless_range_loop)]
        for i in 0..self.c.len() {
            self.c[i] += factor * poly[i];
        }
    }

    //  ************************************************************
    pub fn eval(&self, x: G) -> G {
        let n = self.degree();
        let mut v = self.c[n];
        for jj in 1..=n {
            let j = n - jj;
            v = self.c[j] + v * x;
        }
        v
    }

    //  ************************************************************
    pub fn find_roots(&self) -> Vec<G> {
        let mut roots = Vec::with_capacity(self.degree());
        let n = self.degree();
        for rr in 0..=255 {
            let r = G(rr);
            let v = self.eval(r);
            if v == G(0) {
                roots.push(r);
                if roots.len() >= n {
                    return roots;
                }
            }
        }
        roots
    }

    //  ************************************************************
    pub fn derivative(&self) -> Self {
        let n = self.c.len() - 1;
        let mut der = Poly::new(n);
        for i in 0..n {
            let mut p = self.c[i + 1];
            for _ in 1..=i {
                p += self.c[i + 1];
            }
            der.c[i] = p;
        }
        der
    }


    //  ************************************************************
    pub fn generator(nbytes: usize) -> Self {
        trace!("Poly::generator begin; n={}", nbytes);

        // multiply (x + a^n) for n = 1 to nbytes

        let mut genpoly = Poly::new(nbytes);
        if nbytes == 0 {
            return genpoly;
        }
        genpoly[0u8] = G(0).exp();
        genpoly[1u8] = G(0).exp();
        let mut tp = Poly::new(nbytes);
        tp[1u8] = G(0).exp();
        for i in 1..nbytes {
            tp[0u8] = G((i % 256) as u8).exp(); // set up x+a^n
            genpoly = &genpoly * &tp;
        }
        trace!("Poly::generator done; n={}, genpoly={:?} ~ {}", nbytes, genpoly, genpoly);
        genpoly
    }

    //  ************************************************************
    pub fn coef(&self) -> Vec<u8> {
        self.c.iter().map(|g| g.log().into()).collect()
    }
}

//  ************************************************************
impl fmt::Display for Poly {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.coef().fmt(f)
    }
}

//  ************************************************************
impl From<Vec<G>> for Poly {
    fn from(c: Vec<G>) -> Poly {
        Poly { c }
    }
}

//  ************************************************************
impl Index<u8> for Poly {
    type Output = G;

    fn index(&self, idx: u8) -> &G {
        let i = idx as usize;
        if i >= self.c.len() {
            return &G(0);
        }
        &self.c[i]
    }
}

//  ************************************************************
impl IndexMut<u8> for Poly {
    fn index_mut(&mut self, idx: u8) -> &mut G {
        &mut self.c[idx as usize]
    }
}

//  ************************************************************
impl Index<usize> for Poly {
    type Output = G;

    fn index(&self, idx: usize) -> &G {
        &self.c[idx]
    }
}

//  ************************************************************
impl IndexMut<usize> for Poly {
    fn index_mut(&mut self, idx: usize) -> &mut G {
        &mut self.c[idx]
    }
}


//  ************************************************************
///  polynomial multiplication
//  ************************************************************

#[allow(clippy::suspicious_arithmetic_impl)]
impl<'a> Mul for &'a Poly {
    type Output = Poly;

    fn mul(self, other: &Poly) -> Poly {
        let m = self.c.len();
        let n = other.c.len();
        trace!("Poly::Mul::mul begin; n={} m={}", n, m);
        insane!("Poly::Mul::mul self={:?}", self.c);
        insane!("Poly::Mul::mul other={:?}", other.c);
        let mut dst = Poly::new(n + m + 1);
        for i in 0..m {
            for j in 0..n {
                insane!("Poly::Mul::mul>1 i={} j={}: {:?} + {:?}", i, j, self.c[i], other.c[j]);
                insane!("Poly::Mul::mul>2 i={} j={}: {:?} + {:?}", i, j, dst[i + j], self.c[i] * other.c[j]);
                dst[i + j] += self.c[i] * other.c[j];
                insane!("Poly::Mul::mul>3 dst={:?}", dst[i + j]);
            }
        }
        insane!("Poly::Mul::mul dst={:?}", dst.c);
        dst.simplify();
        insane!("Poly::Mul::mul dst={:?}", dst.c);
        trace!("Poly::Mul::mul dst={}", dst);
        dst
    }
}


//  ************************************************************
/// polynomial addition
//  ************************************************************

impl<'a> AddAssign<&'a Poly> for Poly {
    fn add_assign(&mut self, other: &Poly) {
        for i in 0..self.c.len() {
            self.c[i] += other.c[i];
        }
    }
}


//  ************************************************************
/// polynomial subtraction
//  ************************************************************

impl<'a> SubAssign<&'a Poly> for Poly {
    fn sub_assign(&mut self, other: &Poly) {
        for i in 0..self.c.len() {
            self.c[i] -= other.c[i];
        }
    }
}


//  ************************************************************
/// multiply scalar with polynomial
//  ************************************************************

impl MulAssign<G> for Poly {
    fn mul_assign(&mut self, factor: G) {
        for i in 0..self.c.len() {
            self.c[i] *= factor;
        }
    }
}

//  ************************************************************
/// polynomial scaling
//  ************************************************************

impl ShlAssign<usize> for Poly {
    #[allow(clippy::suspicious_op_assign_impl)]
    fn shl_assign(&mut self, shift: usize) {
        let n = self.c.len();
        if shift >= n {
            warn!("Poly:ShlAssign: ALL BITS SHIFTED OUT {:?} <<= {}", self.c, shift);
            for i in 0..n {
                self.c[i] = G(0);
            }
            return;
        }

        let mut sp = 0;
        for i in n - shift..n {
            if self.c[i - 1] != G(0) {
                sp += 1;
            }
        }
        if sp > 0 {
            warn!("Poly:ShlAssign: VALUES SHIFTED OUT {}: {:?} << {}", sp, self.c, shift);
        }
        for i in 1..n + 1 - shift {
            self.c[n - i] = self.c[n - i - shift];
        }
        for i in 0..shift {
            self.c[i] = G(0);
        }
    }
}


//  ************************************************************
#[cfg(test)]
//  ************************************************************

mod poly {
    use super::*;

    #[test]
    fn generator_02() {
        let g = Poly::generator(2);
        let p = g.coef();
        let mut q: Vec<u8> = vec![0, 25, 1];
        q.reverse();
        assert!(p == q, "invalid generator 02; got {:?}; expected {:?}", p, q);
    }

    #[test]
    fn generator_03() {
        let g = Poly::generator(3);
        let p = g.coef();
        let mut q: Vec<u8> = vec![0, 198, 199, 3];
        q.reverse();
        assert!(p == q, "invalid generator 03; got {:?}; expected {:?}", p, q);
    }

    #[test]
    fn generator_68() {
        let g = Poly::generator(68);
        let p = g.coef();
        let mut q: Vec<u8> = vec![
            0, 247, 159, 223, 33, 224, 93, 77, 70, 90, 160, 32, 254, 43, 150, 84, 101, 190, 205, 133, 52, 60, 202, 165, 220, 203,
            151, 93, 84, 15, 84, 253, 173, 160, 89, 227, 52, 199, 97, 95, 231, 52, 177, 41, 125, 137, 241, 166, 225, 118, 2, 54,
            32, 82, 215, 175, 198, 43, 238, 235, 27, 101, 184, 127, 3, 5, 8, 163, 238,
        ];
        q.reverse();
        assert!(p == q, "invalid generator 68; got {:?}; expected {:?}", p, q);
    }
}


//  ************************************************************
/// Element in Galois Field
//  ************************************************************

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct G(u8);

//  ************************************************************
impl G {
    pub fn exp(self) -> Self {
        GF285_EXP[self.0 as usize]
    }
    pub fn log(self) -> Self {
        GF285_LOG[self.0 as usize]
    }
    pub fn log_inv(self) -> Self {
        // log of the inverse
        // the inverse is GF285_EXP[255u8 - GF285_LOG[self.0 as usize]]
        // so to get the log of the inverse simply remove the GF285_EXP term
        G(255u8 - GF285_LOG[self.0 as usize].0)
    }
}

//  ************************************************************
impl From<G> for u8 {
    fn from(g: G) -> u8 {
        g.0
    }
}

//  ************************************************************
impl From<G> for usize {
    fn from(g: G) -> usize {
        g.0 as usize
    }
}

//  ************************************************************
impl Mul for G {
    type Output = G;

    fn mul(self, other: G) -> G {
        // Galois field multiplication
        if self.0 == 0 || other.0 == 0 {
            return G(0);
        }
        let i = u16::from(GF285_LOG[self.0 as usize].0);
        let j = u16::from(GF285_LOG[other.0 as usize].0);
        GF285_EXP[((i + j) % 255) as usize]
    }
}


//  ************************************************************
impl MulAssign for G {
    fn mul_assign(&mut self, other: G) {
        // Galois field multiplication
        if self.0 == 0 {
            return;
        }
        if other.0 == 0 {
            self.0 = 0
        } else {
            let i = u16::from(GF285_LOG[self.0 as usize].0);
            let j = u16::from(GF285_LOG[other.0 as usize].0);
            let e = GF285_EXP[((i + j) % 255) as usize];
            self.0 = e.0
        }
    }
}

//  ************************************************************
impl Div for G {
    type Output = G;

    fn div(self, other: G) -> G {
        // Galois field division
        if self.0 == 0 {
            return G(0);
        }
        if other.0 == 0 {
            panic!("cannot divide by zero in galois field");
        }
        let i = u16::from(GF285_LOG[self.0 as usize].0);
        let j = u16::from(GF285_LOG[other.0 as usize].0);
        GF285_EXP[((255 + i - j) % 255) as usize]
    }
}

//  ************************************************************
impl Not for G {
    type Output = G;
    fn not(self) -> G {
        // Galois Field inverse
        let i: u8 = 255u8 - GF285_LOG[self.0 as usize].0;
        GF285_EXP[i as usize]
    }
}


//  ************************************************************
impl AddAssign for G {
    fn add_assign(&mut self, other: G) {
        *self = *self + other;
    }
}

//  ************************************************************

#[allow(clippy::suspicious_arithmetic_impl)]
impl Add for G {
    type Output = G;
    fn add(self, other: G) -> G {
        G(self.0 ^ other.0)
    }
}


//  ************************************************************
impl SubAssign for G {
    fn sub_assign(&mut self, other: G) {
        *self = *self - other;
    }
}

//  ************************************************************

#[allow(clippy::suspicious_arithmetic_impl)]
impl Sub for G {
    type Output = G;
    fn sub(self, other: G) -> G {
        G(self.0 ^ other.0)
    }
}


//  ************************************************************
#[cfg(test)]
//  ************************************************************

mod galois {
    use super::*;

    #[test]
    fn exp_all() {
        let mut f = [false; 256];
        for i in 0..=255 {
            let e = GF285_EXP[i];
            let j: usize = e.into();
            if f[j] {
                panic!("galois exponent {:?} found more than once", e);
            }
            f[j] = true;
        }
    }

    #[test]
    fn log_all() {
        let mut f = [false; 256];
        for i in 0..=255 {
            let e = GF285_LOG[i];
            let j: usize = e.into();
            if f[j] {
                panic!("galois logarithm {:?} found more than once", e);
            }
            f[j] = true;
        }
    }

    #[test]
    fn log_exp_inverse() {
        for i in 0..=255u8 {
            let log_i = GF285_LOG[i as usize];
            let j: usize = log_i.into();
            let exp_log_i = GF285_EXP[j];
            assert!(exp_log_i == G(i), "exp and log not inverse; i={:?} exp(log(i))={:?}", G(i), exp_log_i);
        }
    }

    #[test]
    fn add_commutative() {
        for i in 0..=255 {
            let a = G(i);
            for j in 0..=255 {
                let b = G(j);
                assert!(a + b == b + a, "addition not commutative; a={:?} b={:?} a+b={:?} b+a={:?}", a, b, a + b, b + a);
            }
        }
    }

    #[test]
    fn add_associative() {
        for i in 0..=255 {
            let a = G(i);
            for j in 0..=255 {
                let b = G(j);
                for k in 0..=255 {
                    let c = G(k);
                    assert!(
                        a + (b + c) == (a + b) + c,
                        "addition not associative; a={:?} b={:?} c={:?} a+(b+c)={:?} (a+b)+c={:?}",
                        a,
                        b,
                        c,
                        a + (b + c),
                        (a + b) + c
                    );
                }
            }
        }
    }

    #[test]
    fn mul_commutative() {
        for i in 0..=255 {
            let a = G(i);
            for j in 0..=255 {
                let b = G(j);
                assert!(a * b == b * a, "multiplication not commutative; a={:?} b={:?} a*b={:?} b*a={:?}", a, b, a * b, b * a);
            }
        }
    }

    #[test]
    fn mul_associative() {
        for i in 0..=255 {
            let a = G(i);
            for j in 0..=255 {
                let b = G(j);
                for k in 0..=255 {
                    let c = G(k);
                    assert!(
                        a * (b * c) == (a * b) * c,
                        "multiplication not associative; a={:?} b={:?} c={:?} a*(b*c)={:?} (a*b)*c={:?}",
                        a,
                        b,
                        c,
                        a * (b * c),
                        (a * b) * c
                    );
                }
            }
        }
    }

    #[test]
    fn mul_add_distributive() {
        for i in 0..=255 {
            let a = G(i);
            for j in 0..=255 {
                let b = G(j);
                for k in 0..=255 {
                    let c = G(k);
                    assert!(
                        a * (b + c) == (a * b) + (a * c),
                        "multiplication not distributive over addition; a={:?} b={:?} c={:?} a*(b+c)={:?} (a*b)+(a*c)={:?}",
                        a,
                        b,
                        c,
                        a * (b + c),
                        (a * b) + (a * c)
                    );
                }
            }
        }
    }

    #[test]
    fn div_mul_inverse() {
        for i in 0..=255 {
            let a = G(i);
            for j in 1..=255 {
                let b = G(j);
                let c = a / b;
                let bc = b * c;
                assert!(
                    a == b * c,
                    "division is not inverse of multiplication: a={:?} b={:?} a/b={:?} b*(a/b)={:?}!={:?}",
                    a,
                    b,
                    c,
                    bc,
                    a
                );
            }
        }
    }


    #[test]
    fn not_self_inverse() {
        for i in 0..=255 {
            let inv = !i;
            let rev = !inv;
            assert!(i == rev, "not is not self inverse; !!{:?}={:?}", i, rev);
        }
    }
}


//  ************************************************************
/// Pre-calculated exponentials for Galois Field 285
//  ************************************************************

const GF285_EXP: [G; 256] = [
    G(0x01u8), // 0x00
    G(0x02u8), // 0x01
    G(0x04u8), // 0x02
    G(0x08u8), // 0x03
    G(0x10u8), // 0x04
    G(0x20u8), // 0x05
    G(0x40u8), // 0x06
    G(0x80u8), // 0x07
    G(0x1du8), // 0x08
    G(0x3au8), // 0x09
    G(0x74u8), // 0x0a
    G(0xe8u8), // 0x0b
    G(0xcdu8), // 0x0c
    G(0x87u8), // 0x0d
    G(0x13u8), // 0x0e
    G(0x26u8), // 0x0f
    G(0x4cu8), // 0x10
    G(0x98u8), // 0x11
    G(0x2du8), // 0x12
    G(0x5au8), // 0x13
    G(0xb4u8), // 0x14
    G(0x75u8), // 0x15
    G(0xeau8), // 0x16
    G(0xc9u8), // 0x17
    G(0x8fu8), // 0x18
    G(0x03u8), // 0x19
    G(0x06u8), // 0x1a
    G(0x0cu8), // 0x1b
    G(0x18u8), // 0x1c
    G(0x30u8), // 0x1d
    G(0x60u8), // 0x1e
    G(0xc0u8), // 0x1f
    G(0x9du8), // 0x20
    G(0x27u8), // 0x21
    G(0x4eu8), // 0x22
    G(0x9cu8), // 0x23
    G(0x25u8), // 0x24
    G(0x4au8), // 0x25
    G(0x94u8), // 0x26
    G(0x35u8), // 0x27
    G(0x6au8), // 0x28
    G(0xd4u8), // 0x29
    G(0xb5u8), // 0x2a
    G(0x77u8), // 0x2b
    G(0xeeu8), // 0x2c
    G(0xc1u8), // 0x2d
    G(0x9fu8), // 0x2e
    G(0x23u8), // 0x2f
    G(0x46u8), // 0x30
    G(0x8cu8), // 0x31
    G(0x05u8), // 0x32
    G(0x0au8), // 0x33
    G(0x14u8), // 0x34
    G(0x28u8), // 0x35
    G(0x50u8), // 0x36
    G(0xa0u8), // 0x37
    G(0x5du8), // 0x38
    G(0xbau8), // 0x39
    G(0x69u8), // 0x3a
    G(0xd2u8), // 0x3b
    G(0xb9u8), // 0x3c
    G(0x6fu8), // 0x3d
    G(0xdeu8), // 0x3e
    G(0xa1u8), // 0x3f
    G(0x5fu8), // 0x40
    G(0xbeu8), // 0x41
    G(0x61u8), // 0x42
    G(0xc2u8), // 0x43
    G(0x99u8), // 0x44
    G(0x2fu8), // 0x45
    G(0x5eu8), // 0x46
    G(0xbcu8), // 0x47
    G(0x65u8), // 0x48
    G(0xcau8), // 0x49
    G(0x89u8), // 0x4a
    G(0x0fu8), // 0x4b
    G(0x1eu8), // 0x4c
    G(0x3cu8), // 0x4d
    G(0x78u8), // 0x4e
    G(0xf0u8), // 0x4f
    G(0xfdu8), // 0x50
    G(0xe7u8), // 0x51
    G(0xd3u8), // 0x52
    G(0xbbu8), // 0x53
    G(0x6bu8), // 0x54
    G(0xd6u8), // 0x55
    G(0xb1u8), // 0x56
    G(0x7fu8), // 0x57
    G(0xfeu8), // 0x58
    G(0xe1u8), // 0x59
    G(0xdfu8), // 0x5a
    G(0xa3u8), // 0x5b
    G(0x5bu8), // 0x5c
    G(0xb6u8), // 0x5d
    G(0x71u8), // 0x5e
    G(0xe2u8), // 0x5f
    G(0xd9u8), // 0x60
    G(0xafu8), // 0x61
    G(0x43u8), // 0x62
    G(0x86u8), // 0x63
    G(0x11u8), // 0x64
    G(0x22u8), // 0x65
    G(0x44u8), // 0x66
    G(0x88u8), // 0x67
    G(0x0du8), // 0x68
    G(0x1au8), // 0x69
    G(0x34u8), // 0x6a
    G(0x68u8), // 0x6b
    G(0xd0u8), // 0x6c
    G(0xbdu8), // 0x6d
    G(0x67u8), // 0x6e
    G(0xceu8), // 0x6f
    G(0x81u8), // 0x70
    G(0x1fu8), // 0x71
    G(0x3eu8), // 0x72
    G(0x7cu8), // 0x73
    G(0xf8u8), // 0x74
    G(0xedu8), // 0x75
    G(0xc7u8), // 0x76
    G(0x93u8), // 0x77
    G(0x3bu8), // 0x78
    G(0x76u8), // 0x79
    G(0xecu8), // 0x7a
    G(0xc5u8), // 0x7b
    G(0x97u8), // 0x7c
    G(0x33u8), // 0x7d
    G(0x66u8), // 0x7e
    G(0xccu8), // 0x7f
    G(0x85u8), // 0x80
    G(0x17u8), // 0x81
    G(0x2eu8), // 0x82
    G(0x5cu8), // 0x83
    G(0xb8u8), // 0x84
    G(0x6du8), // 0x85
    G(0xdau8), // 0x86
    G(0xa9u8), // 0x87
    G(0x4fu8), // 0x88
    G(0x9eu8), // 0x89
    G(0x21u8), // 0x8a
    G(0x42u8), // 0x8b
    G(0x84u8), // 0x8c
    G(0x15u8), // 0x8d
    G(0x2au8), // 0x8e
    G(0x54u8), // 0x8f
    G(0xa8u8), // 0x90
    G(0x4du8), // 0x91
    G(0x9au8), // 0x92
    G(0x29u8), // 0x93
    G(0x52u8), // 0x94
    G(0xa4u8), // 0x95
    G(0x55u8), // 0x96
    G(0xaau8), // 0x97
    G(0x49u8), // 0x98
    G(0x92u8), // 0x99
    G(0x39u8), // 0x9a
    G(0x72u8), // 0x9b
    G(0xe4u8), // 0x9c
    G(0xd5u8), // 0x9d
    G(0xb7u8), // 0x9e
    G(0x73u8), // 0x9f
    G(0xe6u8), // 0xa0
    G(0xd1u8), // 0xa1
    G(0xbfu8), // 0xa2
    G(0x63u8), // 0xa3
    G(0xc6u8), // 0xa4
    G(0x91u8), // 0xa5
    G(0x3fu8), // 0xa6
    G(0x7eu8), // 0xa7
    G(0xfcu8), // 0xa8
    G(0xe5u8), // 0xa9
    G(0xd7u8), // 0xaa
    G(0xb3u8), // 0xab
    G(0x7bu8), // 0xac
    G(0xf6u8), // 0xad
    G(0xf1u8), // 0xae
    G(0xffu8), // 0xaf
    G(0xe3u8), // 0xb0
    G(0xdbu8), // 0xb1
    G(0xabu8), // 0xb2
    G(0x4bu8), // 0xb3
    G(0x96u8), // 0xb4
    G(0x31u8), // 0xb5
    G(0x62u8), // 0xb6
    G(0xc4u8), // 0xb7
    G(0x95u8), // 0xb8
    G(0x37u8), // 0xb9
    G(0x6eu8), // 0xba
    G(0xdcu8), // 0xbb
    G(0xa5u8), // 0xbc
    G(0x57u8), // 0xbd
    G(0xaeu8), // 0xbe
    G(0x41u8), // 0xbf
    G(0x82u8), // 0xc0
    G(0x19u8), // 0xc1
    G(0x32u8), // 0xc2
    G(0x64u8), // 0xc3
    G(0xc8u8), // 0xc4
    G(0x8du8), // 0xc5
    G(0x07u8), // 0xc6
    G(0x0eu8), // 0xc7
    G(0x1cu8), // 0xc8
    G(0x38u8), // 0xc9
    G(0x70u8), // 0xca
    G(0xe0u8), // 0xcb
    G(0xddu8), // 0xcc
    G(0xa7u8), // 0xcd
    G(0x53u8), // 0xce
    G(0xa6u8), // 0xcf
    G(0x51u8), // 0xd0
    G(0xa2u8), // 0xd1
    G(0x59u8), // 0xd2
    G(0xb2u8), // 0xd3
    G(0x79u8), // 0xd4
    G(0xf2u8), // 0xd5
    G(0xf9u8), // 0xd6
    G(0xefu8), // 0xd7
    G(0xc3u8), // 0xd8
    G(0x9bu8), // 0xd9
    G(0x2bu8), // 0xda
    G(0x56u8), // 0xdb
    G(0xacu8), // 0xdc
    G(0x45u8), // 0xdd
    G(0x8au8), // 0xde
    G(0x09u8), // 0xdf
    G(0x12u8), // 0xe0
    G(0x24u8), // 0xe1
    G(0x48u8), // 0xe2
    G(0x90u8), // 0xe3
    G(0x3du8), // 0xe4
    G(0x7au8), // 0xe5
    G(0xf4u8), // 0xe6
    G(0xf5u8), // 0xe7
    G(0xf7u8), // 0xe8
    G(0xf3u8), // 0xe9
    G(0xfbu8), // 0xea
    G(0xebu8), // 0xeb
    G(0xcbu8), // 0xec
    G(0x8bu8), // 0xed
    G(0x0bu8), // 0xee
    G(0x16u8), // 0xef
    G(0x2cu8), // 0xf0
    G(0x58u8), // 0xf1
    G(0xb0u8), // 0xf2
    G(0x7du8), // 0xf3
    G(0xfau8), // 0xf4
    G(0xe9u8), // 0xf5
    G(0xcfu8), // 0xf6
    G(0x83u8), // 0xf7
    G(0x1bu8), // 0xf8
    G(0x36u8), // 0xf9
    G(0x6cu8), // 0xfa
    G(0xd8u8), // 0xfb
    G(0xadu8), // 0xfc
    G(0x47u8), // 0xfd
    G(0x8eu8), // 0xfe
    G(0x00u8), // 0xff
];


//  ************************************************************
/// Pre-calculated logarithms for Galois Field 285
//  ************************************************************

const GF285_LOG: [G; 256] = [
    G(0xffu8), // 0x00
    G(0x00u8), // 0x01
    G(0x01u8), // 0x02
    G(0x19u8), // 0x03
    G(0x02u8), // 0x04
    G(0x32u8), // 0x05
    G(0x1au8), // 0x06
    G(0xc6u8), // 0x07
    G(0x03u8), // 0x08
    G(0xdfu8), // 0x09
    G(0x33u8), // 0x0a
    G(0xeeu8), // 0x0b
    G(0x1bu8), // 0x0c
    G(0x68u8), // 0x0d
    G(0xc7u8), // 0x0e
    G(0x4bu8), // 0x0f
    G(0x04u8), // 0x10
    G(0x64u8), // 0x11
    G(0xe0u8), // 0x12
    G(0x0eu8), // 0x13
    G(0x34u8), // 0x14
    G(0x8du8), // 0x15
    G(0xefu8), // 0x16
    G(0x81u8), // 0x17
    G(0x1cu8), // 0x18
    G(0xc1u8), // 0x19
    G(0x69u8), // 0x1a
    G(0xf8u8), // 0x1b
    G(0xc8u8), // 0x1c
    G(0x08u8), // 0x1d
    G(0x4cu8), // 0x1e
    G(0x71u8), // 0x1f
    G(0x05u8), // 0x20
    G(0x8au8), // 0x21
    G(0x65u8), // 0x22
    G(0x2fu8), // 0x23
    G(0xe1u8), // 0x24
    G(0x24u8), // 0x25
    G(0x0fu8), // 0x26
    G(0x21u8), // 0x27
    G(0x35u8), // 0x28
    G(0x93u8), // 0x29
    G(0x8eu8), // 0x2a
    G(0xdau8), // 0x2b
    G(0xf0u8), // 0x2c
    G(0x12u8), // 0x2d
    G(0x82u8), // 0x2e
    G(0x45u8), // 0x2f
    G(0x1du8), // 0x30
    G(0xb5u8), // 0x31
    G(0xc2u8), // 0x32
    G(0x7du8), // 0x33
    G(0x6au8), // 0x34
    G(0x27u8), // 0x35
    G(0xf9u8), // 0x36
    G(0xb9u8), // 0x37
    G(0xc9u8), // 0x38
    G(0x9au8), // 0x39
    G(0x09u8), // 0x3a
    G(0x78u8), // 0x3b
    G(0x4du8), // 0x3c
    G(0xe4u8), // 0x3d
    G(0x72u8), // 0x3e
    G(0xa6u8), // 0x3f
    G(0x06u8), // 0x40
    G(0xbfu8), // 0x41
    G(0x8bu8), // 0x42
    G(0x62u8), // 0x43
    G(0x66u8), // 0x44
    G(0xddu8), // 0x45
    G(0x30u8), // 0x46
    G(0xfdu8), // 0x47
    G(0xe2u8), // 0x48
    G(0x98u8), // 0x49
    G(0x25u8), // 0x4a
    G(0xb3u8), // 0x4b
    G(0x10u8), // 0x4c
    G(0x91u8), // 0x4d
    G(0x22u8), // 0x4e
    G(0x88u8), // 0x4f
    G(0x36u8), // 0x50
    G(0xd0u8), // 0x51
    G(0x94u8), // 0x52
    G(0xceu8), // 0x53
    G(0x8fu8), // 0x54
    G(0x96u8), // 0x55
    G(0xdbu8), // 0x56
    G(0xbdu8), // 0x57
    G(0xf1u8), // 0x58
    G(0xd2u8), // 0x59
    G(0x13u8), // 0x5a
    G(0x5cu8), // 0x5b
    G(0x83u8), // 0x5c
    G(0x38u8), // 0x5d
    G(0x46u8), // 0x5e
    G(0x40u8), // 0x5f
    G(0x1eu8), // 0x60
    G(0x42u8), // 0x61
    G(0xb6u8), // 0x62
    G(0xa3u8), // 0x63
    G(0xc3u8), // 0x64
    G(0x48u8), // 0x65
    G(0x7eu8), // 0x66
    G(0x6eu8), // 0x67
    G(0x6bu8), // 0x68
    G(0x3au8), // 0x69
    G(0x28u8), // 0x6a
    G(0x54u8), // 0x6b
    G(0xfau8), // 0x6c
    G(0x85u8), // 0x6d
    G(0xbau8), // 0x6e
    G(0x3du8), // 0x6f
    G(0xcau8), // 0x70
    G(0x5eu8), // 0x71
    G(0x9bu8), // 0x72
    G(0x9fu8), // 0x73
    G(0x0au8), // 0x74
    G(0x15u8), // 0x75
    G(0x79u8), // 0x76
    G(0x2bu8), // 0x77
    G(0x4eu8), // 0x78
    G(0xd4u8), // 0x79
    G(0xe5u8), // 0x7a
    G(0xacu8), // 0x7b
    G(0x73u8), // 0x7c
    G(0xf3u8), // 0x7d
    G(0xa7u8), // 0x7e
    G(0x57u8), // 0x7f
    G(0x07u8), // 0x80
    G(0x70u8), // 0x81
    G(0xc0u8), // 0x82
    G(0xf7u8), // 0x83
    G(0x8cu8), // 0x84
    G(0x80u8), // 0x85
    G(0x63u8), // 0x86
    G(0x0du8), // 0x87
    G(0x67u8), // 0x88
    G(0x4au8), // 0x89
    G(0xdeu8), // 0x8a
    G(0xedu8), // 0x8b
    G(0x31u8), // 0x8c
    G(0xc5u8), // 0x8d
    G(0xfeu8), // 0x8e
    G(0x18u8), // 0x8f
    G(0xe3u8), // 0x90
    G(0xa5u8), // 0x91
    G(0x99u8), // 0x92
    G(0x77u8), // 0x93
    G(0x26u8), // 0x94
    G(0xb8u8), // 0x95
    G(0xb4u8), // 0x96
    G(0x7cu8), // 0x97
    G(0x11u8), // 0x98
    G(0x44u8), // 0x99
    G(0x92u8), // 0x9a
    G(0xd9u8), // 0x9b
    G(0x23u8), // 0x9c
    G(0x20u8), // 0x9d
    G(0x89u8), // 0x9e
    G(0x2eu8), // 0x9f
    G(0x37u8), // 0xa0
    G(0x3fu8), // 0xa1
    G(0xd1u8), // 0xa2
    G(0x5bu8), // 0xa3
    G(0x95u8), // 0xa4
    G(0xbcu8), // 0xa5
    G(0xcfu8), // 0xa6
    G(0xcdu8), // 0xa7
    G(0x90u8), // 0xa8
    G(0x87u8), // 0xa9
    G(0x97u8), // 0xaa
    G(0xb2u8), // 0xab
    G(0xdcu8), // 0xac
    G(0xfcu8), // 0xad
    G(0xbeu8), // 0xae
    G(0x61u8), // 0xaf
    G(0xf2u8), // 0xb0
    G(0x56u8), // 0xb1
    G(0xd3u8), // 0xb2
    G(0xabu8), // 0xb3
    G(0x14u8), // 0xb4
    G(0x2au8), // 0xb5
    G(0x5du8), // 0xb6
    G(0x9eu8), // 0xb7
    G(0x84u8), // 0xb8
    G(0x3cu8), // 0xb9
    G(0x39u8), // 0xba
    G(0x53u8), // 0xbb
    G(0x47u8), // 0xbc
    G(0x6du8), // 0xbd
    G(0x41u8), // 0xbe
    G(0xa2u8), // 0xbf
    G(0x1fu8), // 0xc0
    G(0x2du8), // 0xc1
    G(0x43u8), // 0xc2
    G(0xd8u8), // 0xc3
    G(0xb7u8), // 0xc4
    G(0x7bu8), // 0xc5
    G(0xa4u8), // 0xc6
    G(0x76u8), // 0xc7
    G(0xc4u8), // 0xc8
    G(0x17u8), // 0xc9
    G(0x49u8), // 0xca
    G(0xecu8), // 0xcb
    G(0x7fu8), // 0xcc
    G(0x0cu8), // 0xcd
    G(0x6fu8), // 0xce
    G(0xf6u8), // 0xcf
    G(0x6cu8), // 0xd0
    G(0xa1u8), // 0xd1
    G(0x3bu8), // 0xd2
    G(0x52u8), // 0xd3
    G(0x29u8), // 0xd4
    G(0x9du8), // 0xd5
    G(0x55u8), // 0xd6
    G(0xaau8), // 0xd7
    G(0xfbu8), // 0xd8
    G(0x60u8), // 0xd9
    G(0x86u8), // 0xda
    G(0xb1u8), // 0xdb
    G(0xbbu8), // 0xdc
    G(0xccu8), // 0xdd
    G(0x3eu8), // 0xde
    G(0x5au8), // 0xdf
    G(0xcbu8), // 0xe0
    G(0x59u8), // 0xe1
    G(0x5fu8), // 0xe2
    G(0xb0u8), // 0xe3
    G(0x9cu8), // 0xe4
    G(0xa9u8), // 0xe5
    G(0xa0u8), // 0xe6
    G(0x51u8), // 0xe7
    G(0x0bu8), // 0xe8
    G(0xf5u8), // 0xe9
    G(0x16u8), // 0xea
    G(0xebu8), // 0xeb
    G(0x7au8), // 0xec
    G(0x75u8), // 0xed
    G(0x2cu8), // 0xee
    G(0xd7u8), // 0xef
    G(0x4fu8), // 0xf0
    G(0xaeu8), // 0xf1
    G(0xd5u8), // 0xf2
    G(0xe9u8), // 0xf3
    G(0xe6u8), // 0xf4
    G(0xe7u8), // 0xf5
    G(0xadu8), // 0xf6
    G(0xe8u8), // 0xf7
    G(0x74u8), // 0xf8
    G(0xd6u8), // 0xf9
    G(0xf4u8), // 0xfa
    G(0xeau8), // 0xfb
    G(0xa8u8), // 0xfc
    G(0x50u8), // 0xfd
    G(0x58u8), // 0xfe
    G(0xafu8), // 0xff
];
