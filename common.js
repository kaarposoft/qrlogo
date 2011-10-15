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
*/

/*jslint browser: true */
/*jslint vars: true */
/*jslint white: true */
/*global alert: true */
/*global FileReader: true */
/*global QRCodeDecode: true, Logger: true, canvas_loader: true */

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
		var img = new Image();
		img.onload = function() {  
			canvas.width = img.width;
			canvas.height = img.height;
			ctx.drawImage(img,0,0);
			func();
		};
		img.src = e.target.result; 
	});

	// Read in the image file as a data URL.
	reader.readAsDataURL(theFile);
}
