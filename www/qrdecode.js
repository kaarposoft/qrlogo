/*
       QR-Logo: http://qrlogo.kaarposoft.dk

       Copyright (C) 2011 Henrik Kaare Poulsen

       Licensed under the Apache License, Version 2.0 (the "License");
       you may not use this file except in compliance with the License.
       You may obtain a copy of the License at

	 http://www.apache.org/licenses/LICENSE-2.0

       Unless required by applicable law or agreed to in writing, software
       distributed under the License is distributed on an "AS IS" BASIS,
       WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
       See the License for the specific language governing permissions and
       limitations under the License.
*/


/* ************************************************************ */
/* Settings for http://www.jslint.com/
** Note that we do NOT adhere to all jslint proposals
*/

/*jslint
	white,
	single,
	this,
	for,
	long,
	browser,
*/
/*global
	alert,
	QRCodeDecode,
	Logger,
	canvas_loader,
	Modernizr,
*/


/* ************************************************************ */
/* JavaScript STRICT mode
*/

"use strict";


/* ************************************************************ */
function qrdecode_onload() {
	if (Modernizr.canvas && Modernizr.filereader) { document.getElementById("noHTML5canvas").style.display = "none"; }
	var qparms = getQueryParams(document.location.search);
	var url = qparms.url;
	if (url) {
		setTimeout(function () {
			canvas_url_loader(url, document.getElementById("qrlogo_canvas"), on_logo_loaded);
		}, 0);
	}
}


/* ************************************************************ */
function on_logo_loaded() {
	document.getElementById("div_decoded").style.display = "none";
	if (qrlogo_wasm_loaded) {
		console.log("waiting for qrlogo_wasm_loaded before enabeling ondecode button");
		qrlogo_wasm_loaded.then( () => {
			console.log("qrlogo_wasm_loaded so enabeling ondecode button");
			document.getElementById("ondecode_button").disabled = false;
		});
	} else {
		console.error("qrlogo_wasm_loaded not set");
	}

}



/* ************************************************************ */
function on_logo_file_upload(evt) {

	canvas_loader(evt, document.getElementById("qrlogo_canvas"), on_logo_loaded);
}


/* ************************************************************ */
function qrdecode_ondecode() {

	console.info("qrdecode_ondecode: Decoding QR Code");
	console.time("qrdecode_ondecode");

	var canvas = document.getElementById("qrlogo_canvas");
	var ctx = canvas.getContext("2d");
	var image_data = ctx.getImageData(0, 0, canvas.width, canvas.height);
	var decoding_result = qrlogo_wasm.decode_from_image_data(image_data, true);

	console.info("decoding_result:");
	console.log(decoding_result);
	console.timeEnd("qrdecode_ondecode");

	if (typeof decoding_result.err !== 'undefined') {
		console.error(decoding_result.err);
		alert(decoding_result.err);
	} else {
		document.getElementById("qrlogo_text").value = decoding_result.data;
		document.getElementById("div_decoded").style.display = "block";
	}
}
