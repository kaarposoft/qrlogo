/* ************************************************************

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
//! Logging utilities (logging to stdout or browser console)
//  ************************************************************

//!
//! The following `LOG_LEVEL`s are defined:
//!
//! | LOG_LEVEL | Meaning |
//! | --------- | ------- |
//! | 0         | Log only errors and warnings |
//! | 1         | Log level *log* |
//! | 2         | Log level *debug* |
//! | 3         | Log level *trace* |
//! | 4         | Log level *insane* |
//!
//! The different functions will emit a log message depending on
//! `LOG_LEVEL` (which can be set with `set_loglevel`
//! and `MAX_LOG_LEVEL` which is defined for different configurations
//! (i.e. less logging for release than for debug)

use std::sync::atomic::{AtomicUsize, Ordering};


//  ************************************************************
/// Global logging level
//  ************************************************************

pub static mut LOG_LEVEL: AtomicUsize = AtomicUsize::new(2);


//  ************************************************************
/// Static maximal logging level (defined by cargo configuration)
//  ************************************************************

#[cfg(debug_assertions)]
pub const MAX_LOG_LEVEL: usize = 5;

#[cfg(not(debug_assertions))]
pub const MAX_LOG_LEVEL: usize = 2;


//  ************************************************************
/// Set global logging level (`LOG_LEVEL`)
//  ************************************************************

pub fn set_loglevel(lvl: usize) {
    unsafe { LOG_LEVEL.store(lvl, Ordering::Relaxed) };
}


//  ************************************************************
/// Return true if `lvl` should be logged according to `LOG_LEVEL`
//  ************************************************************

pub fn shall_log(lvl: usize) -> bool {
    let max = unsafe { LOG_LEVEL.load(Ordering::Relaxed) };
    lvl <= max
}


/* ============================================================
 * Logging in browser
 * ============================================================
 */

#[cfg(all(target_arch = "wasm32", not(target_os = "emscripten")))]
#[macro_export]
macro_rules! error {
    ($($t:tt)*) => ({
        use web_sys_fallback::console;
        console::error_with_str(&format!($($t)*))
    })
}

#[cfg(all(target_arch = "wasm32", not(target_os = "emscripten")))]
#[macro_export]
macro_rules! warn {
    ($($t:tt)*) => ({
        use web_sys_fallback::console;
        console::warn_with_str(&format!($($t)*))
    })
}

#[cfg(all(target_arch = "wasm32", not(target_os = "emscripten")))]
#[macro_export]
macro_rules! log {
    ($($t:tt)*) => (if (logging::MAX_LOG_LEVEL>0) && logging::shall_log(1) {
        use web_sys_fallback::console;
        console::log_with_str(&format!($($t)*))
    })
}

#[cfg(all(target_arch = "wasm32", not(target_os = "emscripten")))]
#[macro_export]
macro_rules! debug {
    ($($t:tt)*) => (if (logging::MAX_LOG_LEVEL>1) && logging::shall_log(2) {
        use web_sys_fallback::console;
        console::log_with_str(&format!($($t)*))
    })
}

#[cfg(all(target_arch = "wasm32", not(target_os = "emscripten")))]
#[macro_export]
macro_rules! trace {
    ($($t:tt)*) => (if (logging::MAX_LOG_LEVEL>2) && logging::shall_log(3) {
        use web_sys_fallback::console;
        console::log_with_str(&format!($($t)*))
    })
}

#[cfg(all(target_arch = "wasm32", not(target_os = "emscripten")))]
#[macro_export]
macro_rules! insane {
    ($($t:tt)*) => (if (logging::MAX_LOG_LEVEL>3) && logging::shall_log(4) {
        use web_sys_fallback::console;
        console::log_with_str(&format!($($t)*))
    })
}


/* ============================================================
 * Logging locally
 * ============================================================
 */

/// Unconditionally log an error
#[cfg(not(all(target_arch = "wasm32", not(target_os = "emscripten"))))]
#[macro_export]
macro_rules! error {
    ($($t:tt)*) => (println!($($t)*))
}

/// Unconditionally log a warning
#[cfg(not(all(target_arch = "wasm32", not(target_os = "emscripten"))))]
#[macro_export]
macro_rules! warn {
    ($($t:tt)*) => (println!($($t)*))
}

/// Log if `LOG_LEVEL` >= 1
#[cfg(not(all(target_arch = "wasm32", not(target_os = "emscripten"))))]
#[macro_export]
macro_rules! log {
    ($($t:tt)*) => (if (logging::MAX_LOG_LEVEL>0) && logging::shall_log(1) {
        println!($($t)*)})
}

/// Log if `LOG_LEVEL` >= 2
#[cfg(not(all(target_arch = "wasm32", not(target_os = "emscripten"))))]
#[macro_export]
macro_rules! debug {
    ($($t:tt)*) => (if (logging::MAX_LOG_LEVEL>1) && logging::shall_log(2) {
        println!($($t)*)})
}

/// Log if `LOG_LEVEL` >=3
#[cfg(not(all(target_arch = "wasm32", not(target_os = "emscripten"))))]
#[macro_export]
macro_rules! trace {
    ($($t:tt)*) => (if (logging::MAX_LOG_LEVEL>2) && logging::shall_log(3) {
        println!($($t)*)})
}

/// Log if `LOG_LEVEL` >= 4
#[cfg(not(all(target_arch = "wasm32", not(target_os = "emscripten"))))]
#[macro_export]
macro_rules! insane {
    ($($t:tt)*) => (if (logging::MAX_LOG_LEVEL>3) && logging::shall_log(4) {
        println!($($t)*)})
}
