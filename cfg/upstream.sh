# ============================================================
# Use specifig nightly toolchain to ensure reproducability
# ============================================================
RUST_TOOLCHAIN="nightly-2018-09-13"
printf "\n>>> >>> RUST_TOOLCHAIN: %s\n" "${RUST_TOOLCHAIN}"

# ============================================================
# Use sepecific version of wasm-bindgen-cli
# **** ****
# **** **** MAKE SURE THAT THE VERSION IS THE SAME AS IN Cargo.toml !!!
# **** ****
# ============================================================
WASM_BINDGEN="0.2.21"
printf ">>> >>> WASM_BINDGEN: %s\n" "${WASM_BINDGEN}"
WASM_BINDGEN_INSTALL="--version=${WASM_BINDGEN}"
WASM_BINDGEN_VERSION_STRING="wasm-bindgen-cli v${WASM_BINDGEN}"
