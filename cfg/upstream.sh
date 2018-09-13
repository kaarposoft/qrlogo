# ============================================================
# Use specifig nightly toolchain to ensure reproducability
# ============================================================
RUST_TOOLCHAIN="nightly-2018-09-13"
printf "\n>>> >>> RUST_TOOLCHAIN: %s\n" "${RUST_TOOLCHAIN}"

# ============================================================
# Use sepecific version of wasm-bindgen
# TODO: When wasm-bindgen stabilizes, put those definitions into Cargo.toml
# This can only be done sensibly when the wasm-bindgen/crates js-sys etc.
#     have been split off into separate packages.
# ============================================================
WASM_BINDGEN="0.2.19"
printf ">>> >>> WASM_BINDGEN: %s\n" "${WASM_BINDGEN}"

# ============================================================
# If you modify the above to upgrade to a newer version of rust or wasm-bindgen
# you may also want to remove Cargo.lock
# to pick up newer crate dependencies
# ============================================================
