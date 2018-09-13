console.log("begin loading qrlogo_wasm");
console.time("qrlogo_wasm_load");
qrlogo_wasm('./qrlogo_wasm_bg.wasm')
	.then(() => {
		console.log("qrlogo_wasm_load: done loading wasm");
		console.timeEnd("qrlogo_wasm_load")
	})
	.catch( e => {
		console.timeEnd("qrlogo_wasm_load")
		console.warn("qrlogo_wasm_load: Failed to load wasm");
		console.error(e);
		alert("Failed to load QR Log Web Assembler module");
	});


