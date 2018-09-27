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
//! Common definitions for QR codes
//!
//! # References
//!
//! * [Wikipedia on QR codes](https://en.wikipedia.org/wiki/QR_code)
//! * [ISO 18004:2015](https://www.iso.org/standard/62021.html)
//! * [ISO 18004:2005](http://www.arscreatio.com/repositorio/images/n_23/SC031-N-1915-18004Text.pdf)
//  ************************************************************

#![allow(clippy::collapsible_if)]

use super::logging;
use super::{ErrorCorrectionLevel, Mode};


//  ************************************************************

pub const VERSION_MIN: u8 = 1;
pub const VERSION_MAX: u8 = 40;

pub const QUIET_ZONE: usize = 4;

pub const MODULES_MIN: usize = 17 + 4 * (VERSION_MIN as usize);
pub const MODULES_MAX: usize = 17 + 4 * (VERSION_MAX as usize);


//  ************************************************************

pub fn min_max_module_pixels(img_width: usize, img_height: usize) -> (usize, usize) {
    let min_dim = usize::min(img_width, img_height);
    let max_dim = usize::max(img_width, img_height);
    // A version 40 QR Code covering half the image
    let cover = 2;
    let min = usize::max(1, max_dim / cover / MODULES_MAX);
    // A version 1 QR Code covering the whole image
    let max = usize::max(2, min_dim / MODULES_MIN);
    (min, max)
}


//  ************************************************************

#[rustfmt::skip]
pub fn version_from_length(len: usize, mode: Mode, ec: ErrorCorrectionLevel) -> Option<u8> {

    let version_ec_numeric: [ [u16; 40]; 4] = [
        [   // Numeric M
               34,   63,  101,  149,  202,  255,  293,  365,  432,  513,
              604,  691,  796,  871,  991, 1082, 1212, 1346, 1500, 1600,
             1708, 1872, 2059, 2188, 2395, 2544, 2701, 2857, 3035, 3289,
             3486, 3693, 3909, 4134, 4343, 4588, 4775, 5039, 5313, 5596,
        ],
        [   // Numeric L
               41,   77,  127,  187,  255,  322,  370,  461,  552,  652,
              772,  883, 1022, 1101, 1250, 1408, 1548, 1725, 1903, 2061,
             2232, 2409, 2620, 2812, 3057, 3283, 3517, 3669, 3909, 4158,
             4417, 4686, 4965, 5253, 5529, 5836, 6153, 6479, 6743, 7089,
        ],
        [   // Numeric H
               17,   34,   58,   82,  106,  139,  154,  202,  235,  288,
              331,  374,  427,  468,  530,  602,  674,  746,  813,  919,
              969, 1056, 1108, 1228, 1286, 1425, 1501, 1581, 1677, 1782,
             1897, 2022, 2157, 2301, 2361, 2524, 2625, 2735, 2927, 3057,
        ],
        [   // Numeric Q
               27,   48,   77,  111,  144,  178,  207,  259,  312,  364,
              427,  489,  580,  621,  703,  775,  876,  948, 1063, 1159,
             1224, 1358, 1468, 1588, 1718, 1804, 1933, 2085, 2181, 2358,
             2473, 2670, 2805, 2949, 3081, 3244, 3417, 3599, 3791, 3993,
        ],
    ];

    let version_ec_alpha_numeric: [ [u16; 40]; 4] = [
        [   // AlphaNumeric M
               20,   38,   61,   90,  122,  154,  178,  221,  262,  311,
              366,  419,  483,  528,  600,  656,  734,  816,  909,  970,
             1035, 1134, 1248, 1326, 1451, 1542, 1637, 1732, 1839, 1994,
             2113, 2238, 2369, 2506, 2632, 2780, 2894, 3054, 3220, 3391,
        ],
        [   // AlphaNumeric L
               25,   47,   77,  114,  154,  195,  224,  279,  335,  395,
              468,  535,  619,  667,  758,  854,  938, 1046, 1153, 1249,
             1352, 1460, 1588, 1704, 1853, 1990, 2132, 2223, 2369, 2520,
             2677, 2840, 3009, 3183, 3351, 3537, 3729, 3927, 4087, 4296,
        ],
        [   // AlphaNumeric H
               10,   20,   35,   50,   64,   84,   93,  122,  143,  174,
              200,  227,  259,  283,  321,  365,  408,  452,  493,  557,
              587,  640,  672,  744,  779,  864,  910,  958, 1016, 1080,
             1150, 1226, 1307, 1394, 1431, 1530, 1591, 1658, 1774, 1852,
        ],
        [   // AlphaNumeric Q
               16,   29,   47,   67,   87,  108,  125,  157,  189,  221,
              259,  296,  352,  376,  426,  470,  531,  574,  644,  702,
              742,  823,  890,  963, 1041, 1094, 1172, 1263, 1322, 1429,
             1499, 1618, 1700, 1787, 1867, 1966, 2071, 2181, 2298, 2420,
        ],
    ];

    let version_ec_eight_bit: [ [u16; 40]; 4] = [
        [   // EightBit M
               14,   26,   42,   62,   84,  106,  122,  152,  180,  213,
              251,  287,  331,  362,  412,  450,  504,  560,  624,  666,
              711,  779,  857,  911,  997, 1059, 1125, 1190, 1264, 1370,
             1452, 1538, 1628, 1722, 1809, 1911, 1989, 2099, 2213, 2331,
        ],
        [   // EightBit L
               17,   32,   53,   78,  106,  134,  154,  192,  230,  271,
              321,  367,  425,  458,  520,  586,  644,  718,  792,  858,
              929, 1003, 1091, 1171, 1273, 1367, 1465, 1528, 1628, 1732,
             1840, 1952, 2068, 2188, 2303, 2431, 2563, 2699, 2809, 2953,
        ],
        [   // EightBit H
                7,   14,   24,   34,   44,   58,   64,   84,   98,  119,
              137,  155,  177,  194,  220,  250,  280,  310,  338,  382,
              403,  439,  461,  511,  535,  593,  625,  658,  698,  742,
              790,  842,  898,  958,  983, 1051, 1093, 1139, 1219, 1273,
        ],
        [   // EightBit Q
               11,   20,   32,   46,   60,   74,   86,  108,  130,  151,
              177,  203,  241,  258,  292,  322,  364,  394,  442,  482,
              509,  565,  611,  661,  715,  751,  805,  868,  908,  982,
             1030, 1112, 1168, 1228, 1283, 1351, 1423, 1499, 1579, 1663,
        ],
    ];

    let vvv = match mode {
        Mode::Numeric => version_ec_numeric,
        Mode::AlphaNumeric => version_ec_alpha_numeric,
        Mode::EightBit => version_ec_eight_bit,
    };
    let vv = vvv[ec as usize];
    let v = match vv.binary_search(&(len as u16)) {
        Ok(v) => Some((v+1) as u8),
        Err(v) => {
            if v<40 {
                Some((v+1) as u8)
            } else {
                None
            }
        }
    };
    debug!("version_from_length length={} mode={:?} ec={:?} version={:?}", len, mode, ec, v);
    v
}


//  ************************************************************

pub fn data_capacity(version: u8, mode: Mode, ec: ErrorCorrectionLevel) -> u16 {
    let bytes = n_codewords(version) - n_ec_codewords(version, ec);
    let bits = 8 * bytes - 4 - n_count_bits(version, mode) as u16;
    match mode {
        Mode::EightBit => bits / 8,
        Mode::AlphaNumeric => {
            let cap = (bits / 11) * 2;
            if bits >= (cap / 2) * 11 + 6 {
                cap + 1
            } else {
                cap
            }
        }
        Mode::Numeric => {
            let cap = (bits / 10) * 3;
            if bits >= (cap / 3) * 10 + 7 {
                cap + 2
            } else if bits >= (cap / 3) * 10 + 4 {
                cap + 1
            } else {
                cap
            }
        }
    }
}


//  ************************************************************

pub fn n_modules_from_version(version: u8) -> usize {
    (17 + 4 * version) as usize
}


//  ************************************************************

pub fn n_count_bits(version: u8, mode: Mode) -> u16 {
    match mode {
        Mode::EightBit if version < 10 => 8,
        Mode::EightBit => 16,
        Mode::AlphaNumeric if version < 10 => 9,
        Mode::AlphaNumeric if version < 27 => 11,
        Mode::AlphaNumeric => 13,
        Mode::Numeric if version < 10 => 10,
        Mode::Numeric if version < 27 => 12,
        Mode::Numeric => 14,
    }
}


//  ************************************************************

pub fn mask(m: u8, j: usize, i: usize) -> bool {
    match m {
        0 => (i + j) % 2 == 0,
        1 => i % 2 == 0,
        2 => j % 3 == 0,
        3 => (i + j) % 3 == 0,
        4 => (i / 2 + j / 3) % 2 == 0,
        5 => (i * j) % 2 + (i * j) % 3 == 0,
        6 => ((i * j) % 2 + (i * j) % 3) % 2 == 0,
        7 => ((i + j) % 2 + (i * j) % 3) % 2 == 0,
        _ => false,
    }
}


//  ************************************************************

pub fn n_codewords(version: u8) -> u16 {
    [
        26, 44, 70, 100, 134, 172, 196, 242, 292, 346, 404, 466, 532, 581, 655, 733, 815, 901, 991, 1085, 1156, 1258, 1364, 1474,
        1588, 1706, 1828, 1921, 2051, 2185, 2323, 2465, 2611, 2761, 2876, 3034, 3196, 3362, 3532, 3706,
    ][version as usize - 1]
}


//  ************************************************************

pub fn n_remainder_bits(version: u8) -> usize {
    [0, 7, 7, 7, 7, 7, 0, 0, 0, 0, 0, 0, 0, 3, 3, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 4, 4, 3, 3, 3, 3, 3, 3, 3, 0, 0, 0, 0, 0, 0]
        [version as usize - 1]
}


//  ************************************************************

pub fn n_ec_codewords(version: u8, ec: ErrorCorrectionLevel) -> u16 {
    trace!("n_ec_codewords v={} e={:?} V={} E={}", version, ec, version as usize, ec as usize);
    [
        [10, 7, 17, 13],
        [16, 10, 28, 22],
        [26, 15, 44, 36],
        [36, 20, 64, 52],
        [48, 26, 88, 72],
        [64, 36, 112, 96],
        [72, 40, 130, 108],
        [88, 48, 156, 132],
        [110, 60, 192, 160],
        [130, 72, 224, 192],
        [150, 80, 264, 224],
        [176, 96, 308, 260],
        [198, 104, 352, 288],
        [216, 120, 384, 320],
        [240, 132, 432, 360],
        [280, 144, 480, 408],
        [308, 168, 532, 448],
        [338, 180, 588, 504],
        [364, 196, 650, 546],
        [416, 224, 700, 600],
        [442, 224, 750, 644],
        [476, 252, 816, 690],
        [504, 270, 900, 750],
        [560, 300, 960, 810],
        [588, 312, 1050, 870],
        [644, 336, 1110, 952],
        [700, 360, 1200, 1020],
        [728, 390, 1260, 1050],
        [784, 420, 1350, 1140],
        [812, 450, 1440, 1200],
        [868, 480, 1530, 1290],
        [924, 510, 1620, 1350],
        [980, 540, 1710, 1440],
        [1036, 570, 1800, 1530],
        [1064, 570, 1890, 1590],
        [1120, 600, 1980, 1680],
        [1204, 630, 2100, 1770],
        [1260, 660, 2220, 1860],
        [1316, 720, 2310, 1950],
        [1372, 750, 2430, 2040],
    ][version as usize - 1][ec as usize]
}


//  ************************************************************

pub fn version_info(version: u8) -> u32 {
    [
        0x07C94, 0x085BC, 0x09A99, 0x0A4D3, 0x0BBF6, 0x0C762, 0x0D847, 0x0E60D, 0x0F928, 0x10B78, 0x1145D, 0x12A17, 0x13532,
        0x149A6, 0x15683, 0x168C9, 0x177EC, 0x18EC4, 0x191E1, 0x1AFAB, 0x1B08E, 0x1CC1A, 0x1D33F, 0x1ED75, 0x1F250, 0x209D5,
        0x216F0, 0x228BA, 0x2379F, 0x24B0B, 0x2542E, 0x26A64, 0x27541, 0x28C69,
    ][(version - 7) as usize]
}


//  ************************************************************

pub fn alignment_patterns(version: u8) -> Vec<u8> {
    match version {
        2 => vec![6, 18],
        3 => vec![6, 22],
        4 => vec![6, 26],
        5 => vec![6, 30],
        6 => vec![6, 34],
        7 => vec![6, 22, 38],
        8 => vec![6, 24, 42],
        9 => vec![6, 26, 46],
        10 => vec![6, 28, 50],
        11 => vec![6, 30, 54],
        12 => vec![6, 32, 58],
        13 => vec![6, 34, 62],
        14 => vec![6, 26, 46, 66],
        15 => vec![6, 26, 48, 70],
        16 => vec![6, 26, 50, 74],
        17 => vec![6, 30, 54, 78],
        18 => vec![6, 30, 56, 82],
        19 => vec![6, 30, 58, 86],
        20 => vec![6, 34, 62, 90],
        21 => vec![6, 28, 50, 72, 94],
        22 => vec![6, 26, 50, 74, 98],
        23 => vec![6, 30, 54, 78, 102],
        24 => vec![6, 28, 54, 80, 106],
        25 => vec![6, 32, 58, 84, 110],
        26 => vec![6, 30, 58, 86, 114],
        27 => vec![6, 34, 62, 90, 118],
        28 => vec![6, 26, 50, 74, 98, 122],
        29 => vec![6, 30, 54, 78, 102, 126],
        30 => vec![6, 26, 52, 78, 104, 130],
        31 => vec![6, 30, 56, 82, 108, 134],
        32 => vec![6, 34, 60, 86, 112, 138],
        33 => vec![6, 30, 58, 86, 114, 142],
        34 => vec![6, 34, 62, 90, 118, 146],
        35 => vec![6, 30, 54, 78, 102, 126, 150],
        36 => vec![6, 24, 50, 76, 102, 128, 154],
        37 => vec![6, 28, 54, 80, 106, 132, 158],
        38 => vec![6, 32, 58, 84, 110, 136, 162],
        39 => vec![6, 26, 54, 82, 110, 138, 166],
        40 => vec![6, 30, 58, 86, 114, 142, 170],
        _ => Vec::with_capacity(0),
    }
}

//  ************************************************************

pub struct AlignmentPatternIterator {
    patterns: Vec<u8>,
    i: usize,
    j: usize,
}

impl AlignmentPatternIterator {
    pub fn new(version: u8) -> Self {
        AlignmentPatternIterator { patterns: alignment_patterns(version), i: 0, j: 0 }
    }
}

impl Iterator for AlignmentPatternIterator {
    type Item = (usize, usize);
    fn next(&mut self) -> Option<Self::Item> {
        let pats = &self.patterns;
        let n = pats.len();
        while self.i < n {
            while self.j < n {
                let ii = self.i;
                let jj = self.j;
                //log!("%%%%%%%%%%%% {} {}", ii, jj);
                self.j += 1;
                let good = if ii == 0 {
                    (jj > 0) && (jj < n - 1)
                } else if jj == 0 {
                    (ii > 0) && (ii < n - 1)
                } else {
                    true
                };
                if good {
                    return Some((pats[ii] as usize, pats[jj] as usize));
                }
            }
            self.i += 1;
            self.j = 0;
        }
        None
    }
}

//  ************************************************************

pub const N_VERSION_BITS: usize = 3 * 6;

//  ************************************************************

pub fn version_bit_pos(n: usize) -> (usize, usize) {
    [
        (0, 0),
        (1, 0),
        (2, 0),
        (0, 1),
        (1, 1),
        (2, 1),
        (0, 2),
        (1, 2),
        (2, 2),
        (0, 3),
        (1, 3),
        (2, 3),
        (0, 4),
        (1, 4),
        (2, 4),
        (0, 5),
        (1, 5),
        (2, 5),
    ][n]
}

//  ************************************************************

pub const N_FORMAT_BITS: usize = 15;

//  ************************************************************

pub fn format_bit_positions(n: usize, n_modules: usize) -> [(usize, usize); 2] {
    let n_modules = n_modules as i32;
    let ((x0, y0), (mut x1, mut y1)) = [
        ((8, 0), (-1, 8)),
        ((8, 1), (-2, 8)),
        ((8, 2), (-3, 8)),
        ((8, 3), (-4, 8)),
        ((8, 4), (-5, 8)),
        ((8, 5), (-6, 8)),
        ((8, 7), (-7, 8)),
        ((8, 8), (-8, 8)),
        ((7, 8), (8, -7)),
        ((5, 8), (8, -6)),
        ((4, 8), (8, -5)),
        ((3, 8), (8, -4)),
        ((2, 8), (8, -3)),
        ((1, 8), (8, -2)),
        ((0, 8), (8, -1)),
    ][n];
    if x1 < 0 {
        x1 += n_modules;
    }
    if y1 < 0 {
        y1 += n_modules;
    }
    [(x0, y0), (x1 as usize, y1 as usize)]
}
//  ************************************************************

pub fn format_bit_black_position(n_modules: usize) -> (usize, usize) {
    (8, n_modules - 8)
}
//  ************************************************************

pub fn format_info(mask: u8, ec: ErrorCorrectionLevel) -> u16 {
    [
        0x5412, 0x5125, 0x5E7C, 0x5B4B, 0x45F9, 0x40CE, 0x4F97, 0x4AA0, 0x77C4, 0x72F3, 0x7DAA, 0x789D, 0x662F, 0x6318, 0x6C41,
        0x6976, 0x1689, 0x13BE, 0x1CE7, 0x19D0, 0x0762, 0x0255, 0x0D0C, 0x083B, 0x355F, 0x3068, 0x3F31, 0x3A06, 0x24B4, 0x2183,
        0x2EDA, 0x2BED,
    ][(mask as usize) + 8 * (ec as usize)]
}


//  ************************************************************

pub fn alnum_to_ascii(alnum: u8) -> u8 {
    if alnum > 44 {
        panic!("Invalid alphanumeric");
    }
    [
        0x30, // 0
        0x31, // 1
        0x32, // 2
        0x33, // 3
        0x34, // 4
        0x35, // 5
        0x36, // 6
        0x37, // 7
        0x38, // 8
        0x39, // 9
        0x41, // A
        0x42, // B
        0x43, // C
        0x44, // D
        0x45, // E
        0x46, // F
        0x47, // G
        0x48, // H
        0x49, // I
        0x4A, // J
        0x4B, // K
        0x4C, // L
        0x4D, // M
        0x4E, // N
        0x4F, // O
        0x50, // P
        0x51, // Q
        0x52, // R
        0x53, // S
        0x54, // T
        0x55, // U
        0x56, // V
        0x57, // W
        0x58, // X
        0x59, // Y
        0x5A, // Z
        0x20, // space
        0x24, // $
        0x25, // %
        0x2A, // *
        0x2B, // +
        0x2D, // -
        0x2E, // .
        0x2F, // /
        0x30, // :
    ][alnum as usize]
}


//  ************************************************************

pub fn ascii_to_alnum(ascii: u8) -> u8 {
    let alnum = [
        255, // 0x00
        255, // 0x01
        255, // 0x02
        255, // 0x03
        255, // 0x04
        255, // 0x05
        255, // 0x06
        255, // 0x07
        255, // 0x08
        255, // 0x09
        255, // 0x0A
        255, // 0x0B
        255, // 0x0C
        255, // 0x0D
        255, // 0x0E
        255, // 0x0F
        255, // 0x10
        255, // 0x11
        255, // 0x12
        255, // 0x13
        255, // 0x14
        255, // 0x15
        255, // 0x16
        255, // 0x17
        255, // 0x18
        255, // 0x19
        255, // 0x1A
        255, // 0x1B
        255, // 0x1C
        255, // 0x1D
        255, // 0x1E
        255, // 0x1F
        36,  // 0x20  space
        255, // 0x21
        255, // 0x22
        255, // 0x23
        37,  // 0x24  $
        38,  // 0x25  %
        255, // 0x26
        255, // 0x27
        255, // 0x28
        255, // 0x29
        39,  // 0x2A  *
        40,  // 0x2B  +
        255, // 0x2C
        41,  // 0x2D  -
        42,  // 0x2E  .
        43,  // 0x2F  /
        0,   // 0x30  0
        1,   // 0x31  1
        2,   // 0x32  2
        3,   // 0x33  3
        4,   // 0x34  4
        5,   // 0x35  5
        6,   // 0x36  6
        7,   // 0x37  7
        8,   // 0x38  8
        9,   // 0x39  9
        44,  // 0x3A  :
        255, // 0x3B
        255, // 0x3C
        255, // 0x3D
        255, // 0x3E
        255, // 0x3F
        255, // 0x40
        10,  // 0x41  A
        11,  // 0x42  B
        12,  // 0x43  C
        13,  // 0x44  D
        14,  // 0x45  E
        15,  // 0x46  F
        16,  // 0x47  G
        17,  // 0x48  H
        18,  // 0x49  I
        19,  // 0x4A  J
        20,  // 0x4B  K
        21,  // 0x4C  L
        22,  // 0x4D  M
        23,  // 0x4E  N
        24,  // 0x4F  O
        25,  // 0x50  P
        26,  // 0x51  Q
        27,  // 0x52  R
        28,  // 0x53  S
        29,  // 0x54  T
        30,  // 0x55  U
        31,  // 0x56  V
        32,  // 0x57  W
        33,  // 0x58  X
        34,  // 0x59  Y
        35,  // 0x5A  Z
        255, // 0x5B
        255, // 0x5C
        255, // 0x5D
        255, // 0x5E
        255, // 0x5F
        255, // 0x60
        255, // 0x61
        255, // 0x62
        255, // 0x63
        255, // 0x64
        255, // 0x65
        255, // 0x66
        255, // 0x67
        255, // 0x68
        255, // 0x69
        255, // 0x6A
        255, // 0x6B
        255, // 0x6C
        255, // 0x6D
        255, // 0x6E
        255, // 0x6F
        255, // 0x70
        255, // 0x71
        255, // 0x72
        255, // 0x73
        255, // 0x74
        255, // 0x75
        255, // 0x76
        255, // 0x77
        255, // 0x78
        255, // 0x79
        255, // 0x7A
        255, // 0x7B
        255, // 0x7C
        255, // 0x7D
        255, // 0x7E
        255, // 0x7F
        255, // 0x80
        255, // 0x81
        255, // 0x82
        255, // 0x83
        255, // 0x84
        255, // 0x85
        255, // 0x86
        255, // 0x87
        255, // 0x88
        255, // 0x89
        255, // 0x8A
        255, // 0x8B
        255, // 0x8C
        255, // 0x8D
        255, // 0x8E
        255, // 0x8F
        255, // 0x90
        255, // 0x91
        255, // 0x92
        255, // 0x93
        255, // 0x94
        255, // 0x95
        255, // 0x96
        255, // 0x97
        255, // 0x98
        255, // 0x99
        255, // 0x9A
        255, // 0x9B
        255, // 0x9C
        255, // 0x9D
        255, // 0x9E
        255, // 0x9F
        255, // 0xA0
        255, // 0xA1
        255, // 0xA2
        255, // 0xA3
        255, // 0xA4
        255, // 0xA5
        255, // 0xA6
        255, // 0xA7
        255, // 0xA8
        255, // 0xA9
        255, // 0xAA
        255, // 0xAB
        255, // 0xAC
        255, // 0xAD
        255, // 0xAE
        255, // 0xAF
        255, // 0xB0
        255, // 0xB1
        255, // 0xB2
        255, // 0xB3
        255, // 0xB4
        255, // 0xB5
        255, // 0xB6
        255, // 0xB7
        255, // 0xB8
        255, // 0xB9
        255, // 0xBA
        255, // 0xBB
        255, // 0xBC
        255, // 0xBD
        255, // 0xBE
        255, // 0xBF
        255, // 0xC0
        255, // 0xC1
        255, // 0xC2
        255, // 0xC3
        255, // 0xC4
        255, // 0xC5
        255, // 0xC6
        255, // 0xC7
        255, // 0xC8
        255, // 0xC9
        255, // 0xCA
        255, // 0xCB
        255, // 0xCC
        255, // 0xCD
        255, // 0xCE
        255, // 0xCF
        255, // 0xD0
        255, // 0xD1
        255, // 0xD2
        255, // 0xD3
        255, // 0xD4
        255, // 0xD5
        255, // 0xD6
        255, // 0xD7
        255, // 0xD8
        255, // 0xD9
        255, // 0xDA
        255, // 0xDB
        255, // 0xDC
        255, // 0xDD
        255, // 0xDE
        255, // 0xDF
        255, // 0xE0
        255, // 0xE1
        255, // 0xE2
        255, // 0xE3
        255, // 0xE4
        255, // 0xE5
        255, // 0xE6
        255, // 0xE7
        255, // 0xE8
        255, // 0xE9
        255, // 0xEA
        255, // 0xEB
        255, // 0xEC
        255, // 0xED
        255, // 0xEE
        255, // 0xEF
        255, // 0xF0
        255, // 0xF1
        255, // 0xF2
        255, // 0xF3
        255, // 0xF4
        255, // 0xF5
        255, // 0xF6
        255, // 0xF7
        255, // 0xF8
        255, // 0xF9
        255, // 0xFA
        255, // 0xFB
        255, // 0xFC
        255, // 0xFD
        255, // 0xFE
        255, // 0xFF
    ][ascii as usize];
    if alnum > 44 {
        panic!("Invalid alphanumeric");
    };
    alnum
}


//  ************************************************************

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ECB {
    pub n: usize,
    pub c: usize,
    pub k: usize,
    pub r: usize,
}

impl ECB {
    fn new(e: [usize; 4]) -> Self {
        ECB { n: e[0], c: e[1], k: e[2], r: e[3] }
    }
}


//  ************************************************************

#[rustfmt::skip]
pub fn ec_blocks(version: u8, ec: ErrorCorrectionLevel) -> [ECB; 2] {
    let ec_blocks = [
        [[[1, 26, 16, 4], [0, 0, 0, 0]],   // 1-M
         [[1, 26, 19, 2], [0, 0, 0, 0]],   // 1-L
         [[1, 26, 9, 8], [0, 0, 0, 0]],    // 1-H
         [[1, 26, 13, 6], [0, 0, 0, 0]]],  // 1-Q
        [[[1, 44, 28, 8], [0, 0, 0, 0]],   // 2-M
         [[1, 44, 34, 4], [0, 0, 0, 0]],   // 2-L
         [[1, 44, 16, 14], [0, 0, 0, 0]],  // 2-H
         [[1, 44, 22, 11], [0, 0, 0, 0]]], // 2-Q
        [[[1, 70, 44, 13], [0, 0, 0, 0]],  // 3-M
         [[1, 70, 55, 7], [0, 0, 0, 0]],   // 3-L
         [[2, 35, 13, 11], [0, 0, 0, 0]],  // 3-H
         [[2, 35, 17, 9], [0, 0, 0, 0]]],  // 3-Q
        [[[2, 50, 32, 9], [0, 0, 0, 0]],   // 4-M
         [[1, 100, 80, 10], [0, 0, 0, 0]], // 4-L
         [[4, 25, 9, 8], [0, 0, 0, 0]],    // 4-H
         [[2, 50, 24, 13], [0, 0, 0, 0]]], // 4-Q
        [[[2, 67, 43, 12], [0, 0, 0, 0]],  // 5-M
         [[1, 134, 108, 13], [0, 0, 0, 0]],   // 5-L
         [[2, 33, 11, 11], [2, 34, 12, 11]],  // 5-H
         [[2, 33, 15, 9], [2, 34, 16, 9]]],   // 5-Q
        [[[4, 43, 27, 8], [0, 0, 0, 0]],      // 6-M
         [[2, 86, 68, 9], [0, 0, 0, 0]],      // 6-L
         [[4, 43, 15, 14], [0, 0, 0, 0]],     // 6-H
         [[4, 43, 19, 12], [0, 0, 0, 0]]],    // 6-Q
        [[[4, 49, 31, 9], [0, 0, 0, 0]],      // 7-M
         [[2, 98, 78, 10], [0, 0, 0, 0]],     // 7-L
         [[4, 39, 13, 13], [1, 40, 14, 13]],  // 7-H
         [[2, 32, 14, 9], [4, 33, 15, 9]]],   // 7-Q
        [[[2, 60, 38, 11], [2, 61, 39, 11]],  // 8-M
         [[2, 121, 97, 12], [0, 0, 0, 0]],    // 8-L
         [[4, 40, 14, 13], [2, 41, 15, 13]],  // 8-H
         [[4, 40, 18, 11], [2, 41, 19, 11]]], // 8-Q
        [[[3, 58, 36, 11], [2, 59, 37, 11]],  // 9-M
         [[2, 146, 116, 15], [0, 0, 0, 0]],   // 9-L
         [[4, 36, 12, 12], [4, 37, 13, 12]],  // 9-H
         [[4, 36, 16, 10], [4, 37, 17, 10]]], // 9-Q
        [[[4, 69, 43, 13], [1, 70, 44, 13]],  // 10-M
         [[2, 86, 68, 9], [2, 87, 69, 9]],    // 10-L
         [[6, 43, 15, 14], [2, 44, 16, 14]],  // 10-H
         [[6, 43, 19, 12], [2, 44, 20, 12]]], // 10-Q
        [[[1, 80, 50, 15], [4, 81, 51, 15]],  // 11-M
         [[4, 101, 81, 10], [0, 0, 0, 0]],    // 11-L
         [[3, 36, 12, 12], [8, 37, 13, 12]],  // 11-H
         [[4, 50, 22, 14], [4, 51, 23, 14]]], // 11-Q
        [[[6, 58, 36, 11], [2, 59, 37, 11]],  // 12-M
         [[2, 116, 92, 12], [2, 117, 93, 12]],   // 12-L
         [[7, 42, 14, 14], [4, 43, 15, 14]],     // 12-H
         [[4, 46, 20, 13], [6, 47, 21, 13]]],    // 12-Q
        [[[8, 59, 37, 11], [1, 60, 38, 11]],     // 13-M
         [[4, 133, 107, 13], [0, 0, 0, 0]],      // 13-L
         [[12, 33, 11, 11], [4, 34, 12, 11]],    // 13-H
         [[8, 44, 20, 12], [4, 45, 21, 12]]],    // 13-Q
        [[[4, 64, 40, 12], [5, 65, 41, 12]],     // 14-M
         [[3, 145, 115, 15], [1, 146, 116, 15]], // 14-L
         [[11, 36, 12, 12], [5, 37, 13, 12]],    // 14-H
         [[11, 36, 16, 10], [5, 37, 17, 10]]],   // 14-Q
        [[[5, 65, 41, 12], [5, 66, 42, 12]],     // 15-M
         [[5, 109, 87, 11], [1, 110, 88, 11]],   // 15-L
         [[11, 36, 12, 12], [7, 37, 13, 12]],    // 15-H
         [[5, 54, 24, 15], [7, 55, 25, 15]]],    // 15-Q
        [[[7, 73, 45, 14], [3, 74, 46, 14]],     // 16-M
         [[5, 122, 98, 12], [1, 123, 99, 12]],   // 16-L
         [[3, 45, 15, 15], [13, 46, 16, 15]],    // 16-H
         [[15, 43, 19, 12], [2, 44, 20, 12]]],   // 16-Q
        [[[10, 74, 46, 14], [1, 75, 47, 14]],    // 17-M
         [[1, 135, 107, 14], [5, 136, 108, 14]], // 17-L
         [[2, 42, 14, 14], [17, 43, 15, 14]],    // 17-H
         [[1, 50, 22, 14], [15, 51, 23, 14]]],   // 17-Q
        [[[9, 69, 43, 13], [4, 70, 44, 13]],     // 18-M
         [[5, 150, 120, 15], [1, 151, 121, 15]], // 18-L
         [[2, 42, 14, 14], [19, 43, 15, 14]],    // 18-H
         [[17, 50, 22, 14], [1, 51, 23, 14]]],   // 18-Q
        [[[3, 70, 44, 13], [11, 71, 45, 13]],    // 19-M
         [[3, 141, 113, 14], [4, 142, 114, 14]], // 19-L
         [[9, 39, 13, 13], [16, 40, 14, 13]],    // 19-H
         [[17, 47, 21, 13], [4, 48, 22, 13]]],   // 19-Q
        [[[3, 67, 41, 13], [13, 68, 42, 13]],    // 20-M
         [[3, 135, 107, 14], [5, 136, 108, 14]], // 20-L
         [[15, 43, 15, 14], [10, 44, 16, 14]],   // 20-H
         [[15, 54, 24, 15], [5, 55, 25, 15]]],   // 20-Q
        [[[17, 68, 42, 13], [0, 0, 0, 0]],       // 21-M
         [[4, 144, 116, 14], [4, 145, 117, 14]], // 21-L
         [[19, 46, 16, 15], [6, 47, 17, 15]],    // 21-H
         [[17, 50, 22, 14], [6, 51, 23, 14]]],   // 21-Q
        [[[17, 74, 46, 14], [0, 0, 0, 0]],       // 22-M
         [[2, 139, 111, 14], [7, 140, 112, 14]], // 22-L
         [[34, 37, 13, 12], [0, 0, 0, 0]],       // 22-H
         [[7, 54, 24, 15], [16, 55, 25, 15]]],   // 22-Q
        [[[4, 75, 47, 14], [14, 76, 48, 14]],    // 23-M
         [[4, 151, 121, 15], [5, 152, 122, 15]], // 23-L
         [[16, 45, 15, 15], [14, 46, 16, 15]],   // 23-H
         [[11, 54, 24, 15], [14, 55, 25, 15]]],  // 23-Q
        [[[6, 73, 45, 14], [14, 74, 46, 14]],    // 24-M
         [[6, 147, 117, 15], [4, 148, 118, 15]], // 24-L
         [[30, 46, 16, 15], [2, 47, 17, 15]],    // 24-H
         [[11, 54, 24, 15], [16, 55, 25, 15]]],  // 24-Q
        [[[8, 75, 47, 14], [13, 76, 48, 14]],    // 25-M
         [[8, 132, 106, 13], [4, 133, 107, 13]], // 25-L
         [[22, 45, 15, 15], [13, 46, 16, 15]],   // 25-H
         [[7, 54, 24, 15], [22, 55, 25, 15]]],   // 25-Q
        [[[19, 74, 46, 14], [4, 75, 47, 14]],    // 26-M
         [[10, 142, 114, 14], [2, 143, 115, 14]],   // 26-L
         [[33, 46, 16, 15], [4, 47, 17, 15]],       // 26-H
         [[28, 50, 22, 14], [6, 51, 23, 14]]],      // 26-Q
        [[[22, 73, 45, 14], [3, 74, 46, 14]],       // 27-M
         [[8, 152, 122, 15], [4, 153, 123, 15]],    // 27-L
         [[12, 45, 15, 15], [28, 46, 16, 15]],      // 27-H
         [[8, 53, 23, 15], [26, 54, 24, 15]]],      // 27-Q
        [[[3, 73, 45, 14], [23, 74, 46, 14]],       // 28-M
         [[3, 147, 117, 15], [10, 148, 118, 15]],   // 28-L
         [[11, 45, 15, 15], [31, 46, 16, 15]],      // 28-H
         [[4, 54, 24, 15], [31, 55, 25, 15]]],      // 28-Q
        [[[21, 73, 45, 14], [7, 74, 46, 14]],       // 29-M
         [[7, 146, 116, 15], [7, 147, 117, 15]],    // 29-L
         [[19, 45, 15, 15], [26, 46, 16, 15]],      // 29-H
         [[1, 53, 23, 15], [37, 54, 24, 15]]],      // 29-Q
        [[[19, 75, 47, 14], [10, 76, 48, 14]],      // 30-M
         [[5, 145, 115, 15], [10, 146, 116, 15]],   // 30-L
         [[23, 45, 15, 15], [25, 46, 16, 15]],      // 30-H
         [[15, 54, 24, 15], [25, 55, 25, 15]]],     // 30-Q
        [[[2, 74, 46, 14], [29, 75, 47, 14]],       // 31-M
         [[13, 145, 115, 15], [3, 146, 116, 15]],   // 31-L
         [[23, 45, 15, 15], [28, 46, 16, 15]],      // 31-H
         [[42, 54, 24, 15], [1, 55, 25, 15]]],      // 31-Q
        [[[10, 74, 46, 14], [23, 75, 47, 14]],      // 32-M
         [[17, 145, 115, 15], [0, 0, 0, 0]],        // 32-L
         [[19, 45, 15, 15], [35, 46, 16, 15]],      // 32-H
         [[10, 54, 24, 15], [35, 55, 25, 15]]],     // 32-Q
        [[[14, 74, 46, 14], [21, 75, 47, 14]],      // 33-M
         [[17, 145, 115, 15], [1, 146, 116, 15]],   // 33-L
         [[11, 45, 15, 15], [46, 46, 16, 15]],      // 33-H
         [[29, 54, 24, 15], [19, 55, 25, 15]]],     // 33-Q
        [[[14, 74, 46, 14], [23, 75, 47, 14]],      // 34-M
         [[13, 145, 115, 15], [6, 146, 116, 15]],   // 34-L
         [[59, 46, 16, 15], [1, 47, 17, 15]],       // 34-H
         [[44, 54, 24, 15], [7, 55, 25, 15]]],      // 34-Q
        [[[12, 75, 47, 14], [26, 76, 48, 14]],      // 35-M
         [[12, 151, 121, 15], [7, 152, 122, 15]],   // 35-L
         [[22, 45, 15, 15], [41, 46, 16, 15]],      // 35-H
         [[39, 54, 24, 15], [14, 55, 25, 15]]],     // 35-Q
        [[[6, 75, 47, 14], [34, 76, 48, 14]],       // 36-M
         [[6, 151, 121, 15], [14, 152, 122, 15]],   // 36-L
         [[2, 45, 15, 15], [64, 46, 16, 15]],       // 36-H
         [[46, 54, 24, 15], [10, 55, 25, 15]]],     // 36-Q
        [[[29, 74, 46, 14], [14, 75, 47, 14]],      // 37-M
         [[17, 152, 122, 15], [4, 153, 123, 15]],   // 37-L
         [[24, 45, 15, 15], [46, 46, 16, 15]],      // 37-H
         [[49, 54, 24, 15], [10, 55, 25, 15]]],     // 37-Q
        [[[13, 74, 46, 14], [32, 75, 47, 14]],      // 38-M
         [[4, 152, 122, 15], [18, 153, 123, 15]],   // 38-L
         [[42, 45, 15, 15], [32, 46, 16, 15]],      // 38-H
         [[48, 54, 24, 15], [14, 55, 25, 15]]],     // 38-Q
        [[[40, 75, 47, 14], [7, 76, 48, 14]],       // 39-M
         [[20, 147, 117, 15], [4, 148, 118, 15]],   // 39-L
         [[10, 45, 15, 15], [67, 46, 16, 15]],      // 39-H
         [[43, 54, 24, 15], [22, 55, 25, 15]]],     // 39-Q
        [[[18, 75, 47, 14], [31, 76, 48, 14]],      // 40-M
         [[19, 148, 118, 15], [6, 149, 119, 15]],   // 40-L
         [[20, 45, 15, 15], [61, 46, 16, 15]],      // 40-H
         [[34, 54, 24, 15], [34, 55, 25, 15]]],     // 40-Q
    ];
    let [b1, b2] = ec_blocks[(version-1) as usize][ec as usize];
    let e1 = ECB::new(b1);
    let e2 = ECB::new(b2);
    [ e1, e2 ]
}


//  ************************************************************
#[cfg(test)]
//  ************************************************************

mod qr {
    use super::*;

    pub fn version_from_length_calc(len: usize, mode: Mode, ec: ErrorCorrectionLevel) -> Option<u8> {
        let len = len as u16;
        let mut dc = 0;
        for v in VERSION_MIN..=VERSION_MAX {
            dc = data_capacity(v, mode, ec);
            insane!("version_from_length version={} mode={:?} ec={:?} data_capacity={}", v, mode, ec, dc);
            if dc >= len {
                return Some(v);
            }
        }
        None
    }

    #[test]
    fn test_version_from_length() {
        for i in 1..8000 {
            for m in [Mode::Numeric, Mode::AlphaNumeric, Mode::EightBit].iter() {
                for ec in
                    [ErrorCorrectionLevel::L, ErrorCorrectionLevel::M, ErrorCorrectionLevel::Q, ErrorCorrectionLevel::H].iter()
                {
                    let version_looked_up = version_from_length(i, *m, *ec);
                    let version_calculated = version_from_length_calc(i, *m, *ec);
                    assert!(
                        version_looked_up == version_calculated,
                        "WRONG VERSION FROM LENGTH version_looked_up={:?}, version_calculated={:?}",
                        version_looked_up,
                        version_calculated
                    )
                }
            }
        }
    }

    #[test]
    fn test_n_codewords() {
        for version in 1..=40 {
            for ec in [ErrorCorrectionLevel::L, ErrorCorrectionLevel::M, ErrorCorrectionLevel::Q, ErrorCorrectionLevel::H].iter() {
                let [ecb1, ecb2] = qr::ec_blocks(version, *ec);
                let expected_cw: usize = n_codewords(version) as usize;
                let got_cw: usize = ecb1.n * ecb1.c + ecb2.n * ecb2.c;
                assert!(
                    expected_cw == got_cw,
                    "INCONSISTENT NUMBER OF CODEWORDS: version={} ec={:?}; expected={} got={}",
                    version,
                    ec,
                    expected_cw,
                    got_cw
                );
            }
        }
    }

    #[test]
    fn test_n_ec_codewords() {
        for version in 1..=40 {
            for ec in [ErrorCorrectionLevel::L, ErrorCorrectionLevel::M, ErrorCorrectionLevel::Q, ErrorCorrectionLevel::H].iter() {
                let expected_ecw: usize = n_ec_codewords(version, *ec) as usize;
                let [ecb1, ecb2] = qr::ec_blocks(version, *ec);
                let n_dcw = ecb1.n * ecb1.k + ecb2.n * ecb2.k;
                let n_out_codewords = ecb1.n * ecb1.c + ecb2.n * ecb2.c;
                let got_ecw = n_out_codewords - n_dcw;
                assert!(
                    expected_ecw == got_ecw,
                    "INCONSISTENT NUMBER OF EC CODEWORDS: version={} ec={:?}; expected={} got={}",
                    version,
                    ec,
                    expected_ecw,
                    got_ecw
                );
            }
        }
    }

    #[test]
    fn test_ec_blocks() {
        for version in 1..=40 {
            for ec in [ErrorCorrectionLevel::L, ErrorCorrectionLevel::M, ErrorCorrectionLevel::Q, ErrorCorrectionLevel::H].iter() {
                let [ecb1, ecb2] = qr::ec_blocks(version, *ec);
                let [e1, e2] = [ecb1.c - ecb1.k, ecb2.c - ecb2.k];
                let n_dcw = ecb1.n * ecb1.k + ecb2.n * ecb2.k;
                let n_out_codewords = ecb1.n * ecb1.c + ecb2.n * ecb2.c;
                let n_ec_codewords = n_out_codewords - n_dcw;
                println!(
                    "{}, {}, {:?}, {}, n/a, {}, ({},{},{})",
                    version, n_out_codewords, ec, n_ec_codewords, ecb1.n, ecb1.c, ecb1.k, ecb1.r
                );
                if ecb2.n > 0 {
                    println!(
                        "{}, {}, {:?}, {}, n/a, {}, ({},{},{})",
                        version, n_out_codewords, ec, n_ec_codewords, ecb2.n, ecb2.c, ecb2.k, ecb2.r
                    );
                }
                if e2 > 1 {
                    assert!(
                        e1 == e2,
                        "INCONSISTENT NUMBER OF ERROR CORRECTION WORDS: version={} ec={:?} e1={} e2={}",
                        version,
                        ec,
                        e1,
                        e2
                    );
                }
                if ecb2.n > 0 {
                    assert!(
                        ecb2.c == ecb1.c + 1,
                        "INCONSISTENT C VALUES: version={} ec={:?} ecb1.c={} ecb2.c={}",
                        version,
                        ec,
                        ecb1.c,
                        ecb2.c
                    );
                    assert!(
                        ecb2.k == ecb1.k + 1,
                        "INCONSISTENT K VALUES: version={} ec={:?} ecb1.k={} ecb2.k={}",
                        version,
                        ec,
                        ecb1.k,
                        ecb2.k
                    );
                }
            }
        }
    }

    #[test]
    fn test_snake_data_iterator_len() {
        for version in 1..=40 {
            let expected_len = 8 * n_codewords(version) as usize + n_remainder_bits(version);
            let snake_len = SnakeDataIterator::new(version).count();
            assert!(
                snake_len == expected_len,
                "INCONSISTENT SNAKE DATA ITERATOR: version={} expected={}, got={}",
                version,
                expected_len,
                snake_len
            );
        }
    }

}

//  ************************************************************
pub struct SnakeDataIterator {
    version: u8,
    n_modules: usize,
    marks: Vec<bool>,
    first: bool,
    x: usize,
    y: usize,
    dx: usize,
    up: bool,
}

//  ************************************************************
impl SnakeDataIterator {
    pub fn new(version: u8) -> Self {
        let n_modules = n_modules_from_version(version);
        let marks = vec![false; n_modules * n_modules];
        let mut sdi =
            SnakeDataIterator { version, n_modules, marks, first: true, x: n_modules - 2, y: n_modules - 1, dx: 1, up: true };
        sdi.mark();
        sdi
    }
    fn get_mark(&self, x: usize, y: usize) -> bool {
        self.marks[x * self.n_modules + y]
    }
    fn mark_rect(&mut self, x0: usize, y0: usize, w: usize, h: usize) {
        for x in x0..x0 + w {
            for y in y0..y0 + h {
                self.marks[x * self.n_modules + y] = true;
            }
        }
    }
    fn mark(&mut self) {
        // Finder and Format
        let n8 = self.n_modules - 8;
        self.mark_rect(0, 0, 9, 9);
        self.mark_rect(n8, 0, 8, 9);
        self.mark_rect(0, n8, 9, 8);

        // Timing
        self.mark_rect(8, 6, n8 - 8, 1);
        self.mark_rect(6, 8, 1, n8 - 8);

        // Version
        if self.version >= 7 {
            let n11 = self.n_modules - 11;
            self.mark_rect(0, n11, 6, 3);
            self.mark_rect(n11, 0, 3, 6);
        }

        // Alignment
        for (x, y) in AlignmentPatternIterator::new(self.version) {
            let x = x - 2;
            let y = y - 2;
            self.mark_rect(x, y, 5, 5);
        }

        if false {
            self.log_mark();
        }
    }

    fn log_mark(&self) {
        for y in 0..self.n_modules {
            let mut s = String::with_capacity(self.n_modules);
            for x in 0..self.n_modules {
                if self.get_mark(x, y) {
                    s.push('@');
                } else {
                    s.push('.');
                }
                s.push(' ');
            }
            log!("log_mark[{:3}] {}", y, s);
        }
    }
}

//  ************************************************************
impl Iterator for SnakeDataIterator {
    type Item = (usize, usize);
    fn next(&mut self) -> Option<Self::Item> {
        if self.first {
            self.first = false;
            let (x, y) = (self.x + self.dx, self.y);
            trace!("SnakeDataIterator: first [{}, {}]", x, y);
            return Some((x, y));
        }
        loop {
            if self.dx == 1 {
                self.dx = 0;
            } else {
                self.dx = 1;
                let mut turn = false;
                if self.up {
                    if self.y == 0 {
                        turn = true;
                    } else {
                        self.y -= 1;
                    }
                } else {
                    if self.y >= self.n_modules - 1 {
                        turn = true;
                    } else {
                        self.y += 1;
                    }
                };
                if turn {
                    trace!("SnakeDataIterator: next turning");
                    if self.x < 2 {
                        trace!("SnakeDataIterator: next None");
                        return None;
                    }
                    self.up = !self.up;
                    self.x -= 2;
                    if self.x == 5 {
                        // Skip whole column with vertical alignment pattern;
                        // saves time and makes the other code proceed more cleanly
                        self.x -= 1;
                    }
                }
            }
            let (x, y) = (self.x + self.dx, self.y);
            if self.get_mark(x, y) {
                trace!("SnakeDataIterator: skipping [{}, {}]", x, y);
            } else {
                trace!("SnakeDataIterator: next [{}, {}]", x, y);
                return Some((x, y));
            }
        }
    }
}

//  ************************************************************
/// Sequence of bits stored in a byte vector
//  ************************************************************

pub struct BitSeq {
    data: Vec<u8>,
    idx: usize,
}

//  ************************************************************
impl BitSeq {
    //  ************************************************************
    pub fn new(n_bytes: usize) -> Self {
        BitSeq { data: vec![0; n_bytes], idx: 0 }
    }


    //  ************************************************************
    pub fn get_bits(&self, idx: usize, n_bits: usize) -> u16 {
        let len = self.data.len();
        let shift = 24 - (idx & 7) - n_bits;
        let mask = (1 << n_bits) - 1;
        let bidx = idx / 8;
        let mut res = 0u32;
        //let tmp = self.data[bidx] as u16;
        res += (self.data[bidx] as u32) << 16;
        //res += tmp << 16;
        //res += self.data[bidx] << 16;
        if len > bidx + 1 {
            res += (self.data[bidx + 1] as u32) << 8;
            if len > bidx + 2 {
                res += self.data[bidx + 2] as u32;
            }
        }
        ((res >> shift) & mask) as u16
    }

    //  ************************************************************
    pub fn set_bits(&mut self, bits: u16, idx: usize, n_bits: usize) {
        let len = self.data.len();
        insane!("BitSeq::set: data.len()={} bits={} idx={} n_bits={}", len, bits, idx, n_bits);
        let bidx = idx / 8;
        let shift = 24 - (idx & 7) - n_bits;
        let mut v = (u32::from(bits)) << shift;
        if len > bidx + 2 {
            self.data[bidx + 2] = (v & 0x00FF) as u8;
        }
        v >>= 8;
        if len > bidx + 1 {
            self.data[bidx + 1] = (v & 0x00FF) as u8;
        }
        v >>= 8;
        self.data[bidx] += (v & 0x00FF) as u8;
    }

    //  ************************************************************
    pub fn append_bits(&mut self, bits: u16, n_bits: usize) {
        let idx = self.idx;
        self.set_bits(bits, idx, n_bits);
        self.idx += n_bits;
    }

    //  ************************************************************
    pub fn set_u8(&mut self, byte: u8, byte_idx: usize) {
        self.data[byte_idx] = byte;
    }

    //  ************************************************************
    pub fn skip_bits(&mut self, n_bits: usize) -> usize {
        let i = self.idx;
        self.idx += n_bits;
        i
    }

    //  ************************************************************
    pub fn push_bit(&mut self, set: bool) {
        if set {
            let byte = self.idx / 8;
            let bit = self.idx % 8;
            let new = 1 << (7 - bit);
            let old = self.data[byte];
            let res = old | new;
            insane!("push_bit len={} byte={} bit={} new={} old={} res={}", self.data.len(), byte, bit, new, old, res);
            self.data[byte] = res;
        };
        self.idx += 1;
    }

    //  ************************************************************
    pub fn into_bytes(self) -> Vec<u8> {
        self.data
    }

    //  ************************************************************
    pub fn next_byte_idx(&self) -> usize {
        (self.idx - 1) / 8 + 1
    }
}


//  ************************************************************
impl From<Vec<u8>> for BitSeq {
    fn from(data: Vec<u8>) -> Self {
        BitSeq { data, idx: 0 }
    }
}


//  ************************************************************
impl From<BitSeq> for Vec<u8> {
    fn from(bs: BitSeq) -> Self {
        bs.data
    }
}

//  ************************************************************
impl<'a> IntoIterator for &'a BitSeq {
    type Item = bool;
    type IntoIter = BitSeqIterator<'a>;
    fn into_iter(self) -> Self::IntoIter {
        insane!("BitSeq::IntoIterator: {:?}", self.data);
        BitSeqIterator { bits: &self.data, byte_idx: 0, bit_mask: 1 << 7 }
    }
}


//  ************************************************************
/// Iterator over bits in a `BitSeq`
//  ************************************************************

pub struct BitSeqIterator<'a> {
    bits: &'a Vec<u8>,
    byte_idx: usize,
    bit_mask: u8,
}

//  ************************************************************
impl<'a> Iterator for BitSeqIterator<'a> {
    type Item = bool;
    fn next(&mut self) -> Option<bool> {
        if self.bit_mask == 0 {
            self.byte_idx += 1;
            if self.byte_idx >= self.bits.len() {
                return None;
            }
            self.bit_mask = 1 << 7;
        }
        let res = Some(self.bits[self.byte_idx] & self.bit_mask > 0);
        self.bit_mask >>= 1;
        res
    }
}
