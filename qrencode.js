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
	QRCodeDecode,
	Logger,
	Modernizr,
*/


/* ************************************************************ */
/* JavaScript STRICT mode
*/

"use strict";


/* ************************************************************ */
function qrencode_onload() {
	document.getElementById("nojs").style.display = "none";
	if (Modernizr.canvas) { document.getElementById("noHTML5canvas").style.display = "none"; }
}


/* ************************************************************ */
function qrencode_verify() {

	var mode = parseInt(document.getElementById("qrlogo_mode").value, 10);
	var text = document.getElementById("qrlogo_text").value;
	if (mode === 2) { text = text.toUpperCase(); }

	var qr = new QRCodeDecode();

	var debug = document.getElementById("qrlogo_debug_checkbox").checked;
	var logger;
	if (debug) {
		logger = new Logger("div_debug_output");
		logger.debug("<br><br><b>Verification</b><br>");
		qr.logger = logger;
	}

	var canvas = document.getElementById("qrlogo_canvas");
	var ctx = canvas.getContext("2d");
	var imagedata = ctx.getImageData(0, 0, canvas.width, canvas.height);

	var decoded = qr.decodeImageData(imagedata, canvas.width, canvas.height);

	if (text !== decoded) { throw ("Decoded text does not match"); }
}


/* ************************************************************ */
function qrencode_onencode() {

	var qr = new QRCodeDecode();

	var canvas = document.getElementById("qrlogo_canvas");
	var bg_color = document.getElementById("qrlogo_bg_color").color.rgb;
	var module_color = document.getElementById("qrlogo_module_color").color.rgb;

	var mode = parseInt(document.getElementById("qrlogo_mode").value, 10);
	var error_correction_level = parseInt(document.getElementById("qrlogo_errorcorrection").value, 10);
	var text = document.getElementById("qrlogo_text").value;
	if (mode === 2) { text = text.toUpperCase(); }
	var pixpermodule = parseInt(document.getElementById("qrlogo_pixpermodule").value, 10);

	var version = qr.getVersionFromLength(error_correction_level, mode, text.length);

	var debug = document.getElementById("qrlogo_debug_checkbox").checked;
	var logger = null;
	if (debug) {
		logger = new Logger("div_debug_output");
		logger.init();
		qr.logger = logger;
		document.getElementById("div_debug").style.display = "block";
	}

	qr.encodeToCanvas(mode, text, version, error_correction_level, pixpermodule, canvas, bg_color, module_color);

	document.getElementById("qrlogo_version").innerHTML = version.toString();
	document.getElementById("div_encoded").style.display = "block";

	setTimeout(qrencode_verify, 0);
}


/* ************************************************************ */
function qrencode_download() {
	document.location.href = document.getElementById("qrlogo_canvas").toDataURL().replace("image/png", "image/octet-stream");
	return false;
}
