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
//! Pseudo random number generator
//  ************************************************************

use super::qr;
use super::Mode;

//  ************************************************************
/// Very simple XORSHIFT pseudo random number generator
///
/// # References
///
/// - <https://en.wikipedia.org/wiki/Xorshift>
/// - <http://www.jstatsoft.org/v08/i14/paper>
///
/// The `Rng` is only used for generating testdata
//  ************************************************************

pub struct Rng {
    state: u32,
}

impl Rng {
    pub fn new(seed: u32) -> Self {
        Rng { state: seed }
    }
    pub fn get_u32(&mut self) -> u32 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 17;
        self.state ^= self.state << 5;
        self.state - 1
    }
    pub fn get_u8(&mut self) -> u8 {
        self.get_u32() as u8
    }
    pub fn get_u8_clamped(&mut self, min: u8, max: u8) -> u8 {
        min + (self.get_u32() as u8) % (max - min)
    }
    pub fn get_u8_vec(&mut self, len: usize) -> Vec<u8> {
        let mut v = Vec::with_capacity(len);
        for _ in 0..len {
            v.push(self.get_u8());
        }
        v
    }
    pub fn get_usize_clamped(&mut self, min: usize, max: usize) -> usize {
        if min == max {
            min
        } else {
            min + (self.get_u32() as usize) % (max - min)
        }
    }
    pub fn get_usize_unique_clamped_vec(&mut self, len: usize, min: usize, max: usize) -> Vec<usize> {
        assert!(max >= min, "get_usize_unique_clamped_vec: max={} must not be less than min={}", max, min);
        let n = max - min;
        assert!(n > len, "get_usize_unique_clamped_vec: unable to generate {} unique values in range only {} long", len, n);
        if len < n / 2 {
            // If a few values are requested in a large range, generate values one by one
            let mut v = Vec::with_capacity(len);
            for _ in 0..len {
                loop {
                    let u = self.get_usize_clamped(min, max);
                    if !v.contains(&u) {
                        v.push(u);
                        break;
                    }
                }
            }
            v
        } else {
            // If many values are requested in a small range, generate the whole range and remove
            // values one by one
            let mut v = Vec::with_capacity(n);
            for i in min..=max {
                v.push(i)
            }
            while v.len() > len {
                let l = v.len();
                v.remove(self.get_usize_clamped(0, l - 1));
            }
            v
        }
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
