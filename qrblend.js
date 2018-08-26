/*
       QR-Logo: http://qrlogo.kaarposoft.dk

       Copyright (C) 2018 Henrik Kaare Poulsen

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
*/

/*jslint browser: true */
/*jslint vars: true */
/*global alert: true */
/*global QRCodeDecode: true, Modernizr: true, Logger: true */


/* ************************************************************ */
/* JavaScript STRICT mode
*/

"use strict";


/* ************************************************************ */
function qrblend_onload() {
	document.getElementById("nojs").style.display = "none";
	if (Modernizr.canvas) { document.getElementById("noHTML5canvas").style.display = "none"; }
}


/* ************************************************************ */
function on_logo_file_upload(evt) {

	var f = function () {
		document.getElementById("blend_button").disabled = false;
		document.getElementById("div_encoded").style.display = "none";
		document.getElementById("div_debug").style.display = "none";
		var bg_color = document.getElementById("qrlogo_bg_color");
		var module_color = document.getElementById("qrlogo_module_color");
		var ld = QRColor.canvas_light_dark(canvas);
		//console.log(ld);
		var bg_hsl = QRColor.rgb2hsl(ld.light_rgb[0], ld.light_rgb[1] ,ld.light_rgb[2]);
		var module_hsl = QRColor.rgb2hsl(ld.dark_rgb[0], ld.dark_rgb[1] ,ld.dark_rgb[2]);
		bg_hsl[0] = bg_hsl[0]+0.5; if (bg_hsl[0]>1.0) { bg_hsl[0] -= 1.0; }
		module_hsl[0] = module_hsl[0]+0.5; if (module_hsl[0]>1.0) { module_hsl[0] -= 1.0; }
		if (bg_hsl[2] < 0.8) { bg_hsl[2] = 0.8; }
		if (module_hsl[2] > 0.2) { module_hsl[2] = 0.2; }
		//console.log(bg_hsl);
		//console.log(module_hsl);
		var bg_rgb = QRColor.hsl2rgb(bg_hsl[0], bg_hsl[1], bg_hsl[2]);
		var module_rgb = QRColor.hsl2rgb(module_hsl[0], module_hsl[1], module_hsl[2]);
		module_color.jscolor.fromRGB(module_rgb[0], module_rgb[1], module_rgb[2]);
		bg_color.jscolor.fromRGB(bg_rgb[0], bg_rgb[1], bg_rgb[2]);
	};

	var canvas = document.getElementById("qrlogo_input_canvas");
	canvas_loader(evt, canvas, f);

}


/* ************************************************************ */
function qrblend_verify() {

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
function qrblend_onblend() {

	var qr = new QRCodeDecode();

	var canvas = document.getElementById("qrlogo_canvas");
	var ctx = canvas.getContext("2d");
	var bg_color = document.getElementById("qrlogo_bg_color").jscolor.rgb;
	var module_color = document.getElementById("qrlogo_module_color").jscolor.rgb;

	var shade_factor = parseFloat(document.getElementById("qrlogo_shadefactor").value);
//alert("SF " + document.getElementById("qrlogo_shadefactor").value + " -> " + shade_factor);
	var tint_factor = parseFloat(document.getElementById("qrlogo_tintfactor").value);

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

	var canvas_arr = canvas.getContext("2d").getImageData(0, 0, canvas.width, canvas.height).data;

	var input = document.getElementById("qrlogo_input_canvas");
	var input_arr = input.getContext("2d").getImageData(0, 0, input.width, input.height).data;

	var dx = Math.round((canvas.width-input.width)/2);
	if (dx<0) { dx = 0 };
	var dy = Math.round((canvas.height-input.height)/2);
	if (dy<0) { dy = 0 };
	var nx = canvas.width - dx;
	if (nx>input.width) { nx=input.width; }
	var ny = canvas.height - dy;
	if (ny>input.height) { ny=input.height; }
	var mx = canvas.width;
	var my = canvas.height;

	var overlay_arr = new Uint8ClampedArray(mx*my*4);
	for (var x=0; x<nx; x++)
	    for (var y=0; y<ny; y++) {

		var canvas_idx = 4*(dx+x+mx*(dy+y));
		var r0 = canvas_arr[canvas_idx+0];
		var g0 = canvas_arr[canvas_idx+1];
		var b0 = canvas_arr[canvas_idx+2];
		var a0 = canvas_arr[canvas_idx+3];
		var v0 = 0.30 * r0 + 0.59 * g0 + 0.11 * b0;

		var input_idx = 4*(x+input.width*y);
		var r = input_arr[input_idx+0];
		var g = input_arr[input_idx+1];
		var b = input_arr[input_idx+2];
		var a = input_arr[input_idx+3];
		//var a = 100;
		//if (v0 > 127) { a = a/3; }
		if (v0 > 127) { 
		    // QR background
		    r = r + (255-r)*tint_factor;
		    g = g + (255-g)*tint_factor;
		    b = b + (255-b)*tint_factor;
		} else  {
		    // QR marker
		    r = r * (1-shade_factor);
		    g = g * (1-shade_factor);
		    b = b * (1-shade_factor);
		}

		overlay_arr[canvas_idx+0] = r;
		overlay_arr[canvas_idx+1] = g;
		overlay_arr[canvas_idx+2] = b;
		overlay_arr[canvas_idx+3] = a;
	    }
	var overlay_img_data = new ImageData(overlay_arr, mx, my);

	var overlay = document.createElement('canvas');
	overlay.width = mx;
	overlay.height = my;

	var overlay_ctx = overlay.getContext("2d");
	overlay_ctx.putImageData(overlay_img_data, 0, 0);

	var pattern = ctx.createPattern(overlay, "no-repeat");
	ctx.fillStyle = pattern;
	ctx.fillRect(0, 0, canvas.width, canvas.height);

	document.getElementById("qrlogo_version").innerHTML = version.toString();
	document.getElementById("div_encoded").style.display = "block";

	setTimeout(qrblend_verify, 0);
}


/* ************************************************************ */
function qrblend_download() {
	document.location.href = document.getElementById("qrlogo_canvas").toDataURL().replace("image/png", "image/octet-stream");
	return false;
}
