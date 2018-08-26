/* ************************************************************ 

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


/* ************************************************************
 * GLOBAL
 * ************************************************************
 */

var global_qrlogo;


/* ************************************************************
 * QRLOGO OUTPUT
 * ************************************************************
 */

/**  @class
 *   Holds information about one logo to be displayed
 */

function QRLogoOutput() {
	this.init();
}

QRLogoOutput.prototype = {

	/* ************************************************************ */

	grades: [
		'unreadable',	// 0
		'poor',		// 1	
		'fair',		// 2
		'good',		// 3
		'excellent'	// 4
	],

	/*   ************************************************************ */
	/**  Initialize
	 */

	init: function () {

		this.element = document.createElement('div'); /** whatever */
		this.element.className = "logo_output clearfix";

		this.canvas = document.createElement('canvas');
		this.ctx = this.canvas.getContext('2d');
		this.element.appendChild(this.canvas);

		this.logo_grades = document.createElement('div');
		this.logo_grades.className = "logo_grades";

		var div = document.createElement('div');
		this.download = document.createElement('a');
		this.download.innerHTML = "Download Logo";
		this.download.setAttribute("href", "#");
		div.appendChild(this.download);
		this.logo_grades.appendChild(div);

		this.result = document.createElement('div');
		this.logo_grades.appendChild(this.result);

		this.functional_grade = document.createElement('div');
		this.logo_grades.appendChild(this.functional_grade);

		this.error_grade = document.createElement('div');
		this.logo_grades.appendChild(this.error_grade);

		this.element.appendChild(this.logo_grades);
	},


	/* ************************************************************ */
	copyCanvas: function (canvas, w, h) {
		this.canvas.width = w;
		this.canvas.height = h;
		this.ctx.drawImage(canvas, 0, 0);
	},


	/* ************************************************************ */
	showGrades: function (qr, ppm, ok, result) {
		this.download.canvas = this.canvas;
		this.download.onclick = function () {
			document.location.href = this.canvas.toDataURL(); //.replace("image/png", "image/octet-stream");
			return false;
		};

		var grade = 4;
		var fg_text = "";
		var eg_text = "";

		// FUNCTIONAL GRADE
		var fg = qr.functional_grade;
		if (!fg) {
			grade = 0;
			fg_text = "n/a";
		} else {
			if (fg < grade) { grade = fg; }
			fg_text = fg.toString() + ": " + this.grades[fg];
		}
		fg_text = "Functional grade:<br>" + fg_text;

		if (grade > 0) {

			// ERROR GRADE
			var eg = qr.error_grade;
			if (!eg) {
				grade = 0;
				eg_text = "n/a";
			} else {
				if (eg < grade) { grade = eg; }
				eg_text = eg.toString() + ": " + this.grades[eg];
			}
			eg_text = "Error correction grade:<br>" + eg_text;

		}

		// OVERALL GRADE
		if (!ok) { grade = 0; }

		var og_text = "Overall grade:<br>" + grade + ": " + this.grades[grade] + "<br><br>" + result;

		this.qr = qr;
		this.result.innerHTML = og_text;
		this.functional_grade.innerHTML = fg_text;
		this.error_grade.innerHTML = eg_text;
		this.grade = grade;

		this.element.setAttribute('grade', grade.toString());
		this.element.setAttribute('ppm', ppm.toString());

		return grade;
	}
};


/* ************************************************************
 * QRLOGO
 * ************************************************************
 */

/** @class
 *  Main class for generating QR-Logo's
 */

function QRLogo() {
}

QRLogo.prototype = {

	/* ************************************************************ */
	init: function () {
		this.logo_canvas = document.getElementById('qrlogo_canvas');
		this.text = document.getElementById("qrlogo_text");
		this.crlf = document.getElementById("qrlogo_crlf");
		this.debug_checkbox = document.getElementById('qrlogo_debug_checkbox');
		this.bg_color = document.getElementById('qrlogo_bg_color');
		this.module_color = document.getElementById('qrlogo_module_color');
		this.start_button = document.getElementById('qrlogo_start');
		this.stop_button = document.getElementById('qrlogo_stop');
		this.span_logo_i = document.getElementById('span_logo_i');
		this.span_logo_n = document.getElementById('span_logo_n');
		this.div_current = document.getElementById('div_current');
		this.div_best = document.getElementById('div_best');
		this.div_debug = document.getElementById('div_debug');
		this.div_debug_output = document.getElementById('div_debug_output');
		this.div_debug_detailed = document.getElementById('div_debug_detailed');
		this.div_debug_detailed_output = document.getElementById('div_debug_detailed_output');
		this.div_current_qrlogo = document.getElementById('div_current_qrlogo');
		this.div_best_qrlogos = [];
		this.div_best_qrlogos.push(document.getElementById('div_best_qrlogos_1'));
		this.div_best_qrlogos.push(document.getElementById('div_best_qrlogos_2'));
		this.div_best_qrlogos.push(document.getElementById('div_best_qrlogos_3'));
		this.div_best_qrlogos.push(document.getElementById('div_best_qrlogos_4'));
		this.debug = false;
		this.debug_detailed = false;
		this.start_button.disabled = true;
		this.stop_button.disabled = true;

	},

	/* ************************************************************ */
	show: function () {
		this.shouldStop = true;
		this.enableButtons(true);
	},

	/* ************************************************************ */
	enableButtons: function (f) {
		this.start_button.disabled = !f;
		this.stop_button.disabled = f;
	},

	/* ************************************************************ */
	onStart: function () {
		this.enableButtons(false);
		this.shouldStop = false;
		this.div_current.style.display = "block";
		this.div_best.style.display = "block";
		this.debug = this.debug_checkbox.checked;
		if (this.debug) { this.div_debug.style.display = "block"; }
		this.generateLogo();
	},

	/* ************************************************************ */
	onStop: function () {
		this.enableButtons(true);
		this.shouldStop = true;
	},

	/* ************************************************************ */
	onComplete: function () {
		this.div_current.style.display = "none";
		this.onStop();
		if (this.debug && this.debug_detailed) {
		    this.div_debug_detailed.style.display = "block";
		}
	},

	/* ************************************************************ */
	debugOutput: function (s) {
		if (!this.debug) { return; }
		var span = document.createElement('span');
		var ss = "<b>" + s + "</b><br>";
		span.innerHTML = ss;
		this.div_debug_output.appendChild(span);

		if (this.debug_detailed) {
		    this.logger.debug(ss);
		}
	},

	/* ************************************************************ */
	colorsFromImage: function () {

                var ld = QRColor.canvas_light_dark(this.logo_canvas);

		if (ld.light_brightness-ld.dark_brightness < 10) { // not much contrast
			if ((ld.light_brightness+ld.dark_brightness) / 2 < 128) {
				this.module_color.jscolor.fromRGB(ld.dark_rgb[0], ld.dark_rgb[1], ld.dark_rgb[2]);
			} else {
				this.bg_color.jscolor.fromRGB(ld.light_rgb[0], ld.light_rgb[1], ld.light_rgb[2]);
			}
		} else {
			this.module_color.jscolor.fromRGB(ld.dark_rgb[0], ld.dark_rgb[1], ld.dark_rgb[2]);
			this.bg_color.jscolor.fromRGB(ld.light_rgb[0], ld.light_rgb[1], ld.light_rgb[2]);
		}		

	},

	/* ************************************************************ */
	generateLogo: function () {

		var error_correction_level = 2;	// Level H - about 30%
		var mode = 4;			        // 8bit encoding

		var lw = this.logo_canvas.width;
		var lh = this.logo_canvas.height;
		var logo_min = Math.min(lw, lh);
		var logo_max = Math.max(lw, lh);

		var qr = new QRCodeDecode();

		if (this.debug && this.debug_detailed) {
			this.logger = new Logger('div_debug_detailed_output');
			this.logger.init();
			qr.logger = this.logger;
		}
		var text = this.text.value;
	        var crlf = this.crlf.options[this.crlf.selectedIndex].value;
	        text = text_crlf_mode(text, crlf, 4);
		this.txt = text;

		this.version = qr.getVersionFromLength(error_correction_level, mode, text.length);
		if (this.version > 37) { this.max_version = 40; } else { this.max_version = this.version+2; }
		var n_modules = qr.nModulesFromVersion(this.version);

		// Largest logo; aka smallest pixpermodule for QR
		var pixpermodule_min = Math.max(1, Math.ceil(logo_max/(n_modules)));

		// Smallest logo; aka largest pixpermodule for QR
		var pixpermodule_max = Math.max(Math.ceil(21-this.version/2.0), Math.ceil(2.2*logo_max/(n_modules)));

		var n_pixpermodule = (pixpermodule_max-pixpermodule_min);

		this.logo_n = 0;

		this.qr_ppm_array = [];
		this.qr_canvas_array = [];
		this.xy_arr = [];

		var ppm;
		this.debugOutput("logo w=" + lw + " h=" + lh);
		this.debugOutput("version=" + this.version + " #modules=" + n_modules);
		this.debugOutput("ppm=(" + pixpermodule_min + ".." + pixpermodule_max + ")");

		for (ppm=pixpermodule_min; ppm<=pixpermodule_max; ppm++) {

			var canvas = document.createElement('canvas');
			canvas.qrlogo_pixpermodule = ppm;
			qr.encodeToCanvas(mode, text, this.version, error_correction_level, ppm, canvas, this.bg_color.jscolor.rgb, this.module_color.jscolor.rgb);
			this.qr_ppm_array.push(ppm);
			this.qr_canvas_array.push(canvas);

			var n_x = Math.floor(Math.log(Math.pow(n_modules*ppm-lw+1,4))/Math.pow(n_pixpermodule, 1/3))+1;
			var n_y = Math.floor(Math.log(Math.pow(n_modules*ppm-lh+1,4))/Math.pow(n_pixpermodule, 1/3))+1;
			var a = this.generateXY(4*ppm , (4+n_modules)*ppm-lw, (4+n_modules)*ppm-lh, n_x, n_y);
			this.xy_arr.push(a);
			this.logo_n += a.x.length;

			this.debugOutput("");
			this.debugOutput("ppm=" + ppm + " #x=" + n_x + " #y=" + n_y);
			this.debugOutput("x=" + a.x);
			this.debugOutput("y=" + a.y);
		}

		this.debugOutput("");

		this.span_logo_n.innerHTML=this.logo_n;

		var g;
		for (g = 1; g <= 4; g++) {
			while (this.div_best_qrlogos[g-1].hasChildNodes()) {
				this.div_best_qrlogos[g-1].removeChild(this.div_best_qrlogos[g-1].firstChild);
			}
		}
		this.current_logo_output = new QRLogoOutput();
		while (this.div_current_qrlogo.hasChildNodes()) {
			this.div_current_qrlogo.removeChild(this.div_current_qrlogo.firstChild);
		}
		this.div_current_qrlogo.appendChild(this.current_logo_output.element);

		this.logo_i = 0;

		this.current_idx = 0;
		this.current_xy_idx = 0;
		this.best_grade = 0;

		this.current_logo_output.canvas.width = this.qr_canvas_array[this.qr_canvas_array.length-1].width;
		this.current_logo_output.canvas.height = this.qr_canvas_array[this.qr_canvas_array.length-1].height;

		setTimeout(function () { global_qrlogo.generateNext(global_qrlogo); }, 0);
	},

	/* ************************************************************ */
	generateXY: function (min, max_x, max_y, nx, ny) {


		var mx = Math.pow(2,Math.ceil(Math.log(nx)/Math.log(2)));
		var xa = [mx-1];
		var sx;
		for (sx = mx; sx >= 2; sx /= 2) {
			var iix;
			for (iix = sx/2; iix < mx; iix += sx) {
				xa.push(mx-iix-1);
			}
		}

		var my = Math.pow(2,Math.ceil(Math.log(ny)/Math.log(2)));
		var ya = [my-1];
		var sy;
		for (sy = my; sy >= 2; sy /= 2) {
			var iiy;
			for (iiy = sy/2; iiy < my; iiy += sy) {
				ya.push(my-iiy-1);
			}
		}

		var xval = [];
		var yval = [];

		var n = Math.max(nx, ny);

		var dx = (max_x-min)/(mx);
		var dy = (max_y-min)/(my);

		var i;
		for (i = 0; i < nx+ny; i++) {
			var j;
			for (j = Math.min(0, n-i); j <= Math.min(i, n); j++) {
				var ix = i-j;
				var iy = j;
				if ( (ix < nx) && (iy < ny) ) {
					var x = min + dx*xa[ix];
					var y = min + dy*ya[iy];
					x = Math.round(x*100)/100;
					y = Math.round(y*100)/100;
					xval.push(x);
					yval.push(y);
				}
			}
		}

		return { x: xval, y: yval };
	},


	/* ************************************************************ */
	generateNext: function () {

		this.logo_i++;
		this.span_logo_i.innerHTML = this.logo_i;

		this.generateThis();

		if (this.current_idx < this.qr_canvas_array.length-1) {
			this.current_idx++;
		} else {
			this.current_idx=0;
			this.current_xy_idx++;
			while (true) {
				if (this.current_xy_idx < this.xy_arr[this.current_idx].x.length) { break; }
				this.current_idx++;
				if (this.current_idx >= this.qr_canvas_array.length) {
					this.onComplete();
					return;
				}
			}
		}

		if (!this.shouldStop) {
			setTimeout(function () { global_qrlogo.generateNext(global_qrlogo); }, 0);
		}
	},

	/* ************************************************************ */
	generateThis: function () {

		var x = this.xy_arr[this.current_idx].x[this.current_xy_idx];
		var y = this.xy_arr[this.current_idx].y[this.current_xy_idx];

		var source_canvas = this.qr_canvas_array[this.current_idx];
		var dest_canvas = this.current_logo_output.canvas;
		var dest_ctx = this.current_logo_output.ctx;
		var bg_rgb = this.bg_color.jscolor.rgb;
		dest_ctx.fillStyle = "rgb(" + Math.round(bg_rgb[0]) + "," + Math.round(bg_rgb[1]) + "," + Math.round(bg_rgb[2]) +")";
		dest_ctx.fillRect (0, 0, dest_canvas.width, dest_canvas.height);

		dest_ctx.drawImage(source_canvas, 0, 0);

		// The HTML5 standard does not seem to say anything about "rounding" the dx and dy parameters of drawImage.
		// However, Chrome seems to convert to integer, whereas Firefox seems to use the fractional value.
		// To ensure same results in all browsers, we round the dx and dy values.
		dest_ctx.drawImage(this.logo_canvas, Math.round(x), Math.round(y));

		var imagedata = dest_ctx.getImageData(0, 0, dest_canvas.width, dest_canvas.height);

		var qr = new QRCodeDecode();

		if (this.debug_detailed) {
			qr.logger = this.logger;
		}

		var result = "";
		var ok = false;
		var ppm;
		try {
			ppm = source_canvas.qrlogo_pixpermodule;
			var decoded = qr.decodeImageDataInsideBordersWithMaxVersion(imagedata, dest_canvas.width, dest_canvas.height,
				4*ppm, source_canvas.width-4*ppm-1, 4*ppm, source_canvas.height-4*ppm-1, this.max_version);
			ok = (decoded === this.txt);
			if (!ok) { result = "Wrong text decode"; }
		} catch (err) {
			result = err.toString();
			ok = false;
		}

		var grade = this.current_logo_output.showGrades(qr, ppm, ok, result);

		this.debugOutput("grade=" + grade +
			" ppm=" + ppm + " x=" + x + " y=" + y +
			" ok=" + ok + " fun_grade=" + qr.functional_grade + " ec_grade=" + qr.error_grade + " &nbsp; result=[" + result + "]");

		this.showThisResult(grade, ppm, dest_canvas, source_canvas.width, source_canvas.height, qr, ok, result);
	},

	/* ************************************************************ */
	showThisResult: function (grade, ppm, canvas, w, h, qr, ok, result) {
		if (grade > 0) {
			var logo_output;
			if (!this.div_best_qrlogos[grade-1].hasChildNodes()) {
				this.best_grade=grade;
				logo_output = new QRLogoOutput();
				logo_output.copyCanvas(canvas, w, h);
				logo_output.showGrades(qr, ppm, ok, result);
				this.div_best_qrlogos[grade-1].appendChild(logo_output.element);
				return;
			}

			var e = this.div_best_qrlogos[grade-1].firstChild;
			while (e) {
				var e_ppm= e.getAttribute('ppm');
				if ( ppm <= e_ppm ) {
					this.best_grade=grade;
					logo_output = new QRLogoOutput();
					logo_output.copyCanvas(canvas, w, h);
					logo_output.showGrades(qr, ppm, ok, result);
					this.div_best_qrlogos[grade-1].insertBefore(logo_output.element, e);
					return;
				}
				e = e.nextSibling;
			}

			this.best_grade = grade;
			logo_output = new QRLogoOutput();
			logo_output.copyCanvas(canvas, w, h);
			logo_output.showGrades(qr, ppm, ok, result);
			this.div_best_qrlogos[grade-1].appendChild(logo_output.element);
		}
	}
};


/* ************************************************************
 * GLOBALS
 * ************************************************************
 */

/* ************************************************************ */
function qrlogo_onload() {

	document.getElementById('nojs').style.display = "none";
	if (Modernizr.canvas && Modernizr.filereader) {
		document.getElementById('noHTML5canvas').style.display = "none";

		global_qrlogo = new QRLogo();
		global_qrlogo.init();
	}
}


/* ************************************************************ */
function qrlogo_onstart() {
	global_qrlogo.onStart();
}


/* ************************************************************ */
function qrlogo_onstop() {
	global_qrlogo.onStop();
}


/* ************************************************************ */
function on_logo_file_upload(evt) {
	var f = function () {
		global_qrlogo.colorsFromImage();
		global_qrlogo.show(); };
	canvas_loader(evt, document.getElementById('qrlogo_canvas'), f);
}

