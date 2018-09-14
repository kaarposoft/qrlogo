console.log("begin loading qrlogo_wasm");
console.time("qrlogo_wasm_load");
qrlogo_wasm('./qrlogo_wasm_bg.wasm')
	.then(() => {
		console.log("qrlogo_wasm_load: done loading wasm");
		qrlogo_wasm_loaded_resolve(true);
		console.timeEnd("qrlogo_wasm_load")
	})
	.catch( e => {
		console.timeEnd("qrlogo_wasm_load")
		console.error("qrlogo_wasm_load: Failed to load wasm");
		console.error(e);
		alert("Failed to load QR Logo Web Assembler module");
	});


