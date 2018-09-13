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
*/

/* ************************************************************ */
/* JavaScript STRICT mode
*/
"use strict";


/*  ************************************************************ */
/** Always bark on error
 */
window.onerror = function (msg, url, num) {
    alert("Error in QR-Logo:\n\n" + msg + "\n\n" + url + " (line " + num + ")");
    return false;
};


/*  ************************************************************ */
/** Escape text to be output as verbatim html
 *  @param text The text to be escaped
 */

function htmlEscape(text) {
	return text.toString().replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(new RegExp("\n", "g"),"<br>");
	}


function text_crlf_mode(text, crlf, mode) {
	var regex = /(\r\n|\n|\r)/gm;
	if (mode === 2) { text = text.toUpperCase(); }
	if (crlf == "lf") { return text.replace(regex, "\n"); }
	if (crlf == "crlf") { return text.replace(regex, "\r\n"); }
	if (crlf == "cr") { return text.replace(regex, "\r"); }
	return text;
}


/*  ************************************************************ */
/** Create a logger
 *
 *  @param element_id id of the div into which log messages will be written
 *
 *  @class
 *  Class to handle logging into a html div
 */

function Logger(element_id) {
	this.output = document.getElementById(element_id);
}


/*  ************************************************************ */
Logger.prototype = {

    /** Remove all current content of the log div */
	init: function() {
		while (this.output.hasChildNodes()) {
			this.output.removeChild(this.output.firstChild);
		}
	},

	/** Print a debug message. */
	debug: function(s) {
		var span = document.createElement('span');
		span.innerHTML=s+"<br>";
		this.output.insertBefore(span, null);
	}

};


/*  ************************************************************ */
/** Load url into a canvas
 *
 *  @param url      The URL to load
 *  @param canvas   The canvas to load into
 *  @param func     Function to be called when load is complete
 */

function canvas_url_loader(url, canvas, func) {
	var ctx = canvas.getContext('2d');

	var img = new Image();
	img.onload = function() {
		canvas.width = img.width;
		canvas.height = img.height;
		ctx.drawImage(img,0,0);
		func();
	};
	img.onerror = function(err) {
		alert("Unable to load specified URL into canvas");
	}
	img.src = url;
}



/*  ************************************************************ */
/** Load contents of a file into a canvas
 *
 *  @param evt      Event from a input type="file"
 *  @param canvas   The canvas to load into
 *  @param func     Function to be called when load is complete
 */

function canvas_loader(evt, canvas, func) {

	// Note: This will not work on local file with Chrome browser...

	var ctx = canvas.getContext('2d');

	var files = evt.target.files; // FileList object
	var theFile = files[0];

	var reader = new FileReader();

	reader.onload = ( function(e) {
		canvas_url_loader(e.target.result, canvas, func);
	});

	// Read in the image file as a data URL.
	reader.readAsDataURL(theFile);
}

/*  ************************************************************ */
/*  Decode URL query parameters
    https://stackoverflow.com/questions/979975/how-to-get-the-value-from-the-get-parameters#1099670
*/

function getQueryParams(qs) {
    qs = qs.split('+').join(' ');

    var params = {},
	tokens,
	re = /[?&]?([^=]+)=([^&]*)/g;

    while (tokens = re.exec(qs)) {
	params[decodeURIComponent(tokens[1])] = decodeURIComponent(tokens[2]);
    }

    return params;
}

//var query = getQueryParams(document.location.search);
//alert(query.foo);

