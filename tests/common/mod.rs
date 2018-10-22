use qrlogo_wasm::qrdecode::DecodingResult;

pub fn print_decoding_result(res: &DecodingResult) {
    let mut s: Vec<u8> = Vec::new();
    //res.write(&mut io::stdout());
    res.write(&mut s);
    println!("{}", ">".repeat(60));
    print!("{}", String::from_utf8_lossy(&s));
    println!("{}", "<".repeat(60));
}
