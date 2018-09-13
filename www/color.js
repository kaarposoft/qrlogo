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
/*jslint white: true */
/*global alert: true */

"use strict";

var QRColor = {
	canvas_light_dark: function(canvas) {

		var ctx = canvas.getContext("2d");
		var img = ctx.getImageData(0, 0, canvas.width, canvas.height).data;

		var light_brightness = 0;
		var dark_brightness = 255;
		var light_pos = -1;
		var dark_pos = -1;

		var p;
		for (p = 0; p < img.length; p += 4) {
			if (img[p+3] > 127) {
				var gray = img[0] + img[p+1] + img[p+2];
				if ( gray>6 && gray<3*255-6) {
					var b = 0.30 * img[p] + 0.59 * img[p+1] + 0.11 * img[p+2];
					if (b < dark_brightness) { dark_brightness = b; dark_pos = p; }
					if (b > light_brightness) { light_brightness = b; light_pos = p; }
				}
			}
		}

		var light_rgb = [ 255, 255, 255 ];
		if (light_pos >= 0 ) {
			light_rgb = img.slice(light_pos, light_pos+4);
		}
		var dark_rgb = [ 0, 0, 0 ];
		if (dark_pos >= 0) {
			dark_rgb = img.slice(dark_pos, dark_pos+4);
		}

		return {
			light_brightness: light_brightness,
			light_pos: light_pos,
			light_rgb: light_rgb,
			dark_brightness: dark_brightness,
			dark_pos: dark_pos,
			dark_rgb: dark_rgb
		}
	},

	/**
	 * Converts an RGB color value to HSV. Conversion formula
	 * adapted from http://en.wikipedia.org/wiki/HSV_color_space.
	 * Assumes r, g, and b are contained in the set [0, 255] and
	 * returns h, s, and v in the set [0, 1].
	 *
	 * @param   Number  r       The red color value
	 * @param   Number  g       The green color value
	 * @param   Number  b       The blue color value
	 * @return  Array           The HSV representation
	 */
	rgb2hsv: function rgb(r, g, b) {
	    r = r/255, g = g/255, b = b/255;
	    var max = Math.max(r, g, b), min = Math.min(r, g, b);
	    var h, s, v = max;

	    var d = max - min;
	    s = max == 0 ? 0 : d / max;

	    if(max == min){
		h = 0; // achromatic
	    }else{
		switch(max){
		    case r: h = (g - b) / d + (g < b ? 6 : 0); break;
		    case g: h = (b - r) / d + 2; break;
		    case b: h = (r - g) / d + 4; break;
		}
		h /= 6;
	    }

	    return [h, s, v];
	},

	/**
	 * Converts an HSV color value to RGB. Conversion formula
	 * adapted from http://en.wikipedia.org/wiki/HSV_color_space.
	 * Assumes h, s, and v are contained in the set [0, 1] and
	 * returns r, g, and b in the set [0, 255].
	 *
	 * @param   Number  h       The hue
	 * @param   Number  s       The saturation
	 * @param   Number  v       The value
	 * @return  Array           The RGB representation
	 */
	hsv2rgb: function hsv(h, s, v){
	    var r, g, b;

	    var i = Math.floor(h * 6);
	    var f = h * 6 - i;
	    var p = v * (1 - s);
	    var q = v * (1 - f * s);
	    var t = v * (1 - (1 - f) * s);

	    switch(i % 6) {
		case 0: r = v, g = t, b = p; break;
		case 1: r = q, g = v, b = p; break;
		case 2: r = p, g = v, b = t; break;
		case 3: r = p, g = q, b = v; break;
		case 4: r = t, g = p, b = v; break;
		case 5: r = v, g = p, b = q; break;
	    }

	    return [r * 255, g * 255, b * 255];
	},

	/**
	 * Converts an RGB color value to HSL. Conversion formula
	 * adapted from http://en.wikipedia.org/wiki/HSL_color_space.
	 * Assumes r, g, and b are contained in the set [0, 255] and
	 * returns h, s, and l in the set [0, 1].
	 *
	 * @param   Number  r       The red color value
	 * @param   Number  g       The green color value
	 * @param   Number  b       The blue color value
	 * @return  Array           The HSL representation
	 */
	rgb2hsl: function(r, g, b) {
	    r /= 255, g /= 255, b /= 255;
	    var max = Math.max(r, g, b), min = Math.min(r, g, b);
	    var h, s, l = (max + min) / 2;

	    if(max == min){
		h = s = 0; // achromatic
	    }else{
		var d = max - min;
		s = l > 0.5 ? d / (2 - max - min) : d / (max + min);
		switch(max){
		    case r: h = (g - b) / d + (g < b ? 6 : 0); break;
		    case g: h = (b - r) / d + 2; break;
		    case b: h = (r - g) / d + 4; break;
		}
		h /= 6;
	    }

	    return [h, s, l];
	},

	/**
	 * Converts an HSL color value to RGB. Conversion formula
	 * adapted from http://en.wikipedia.org/wiki/HSL_color_space.
	 * Assumes h, s, and l are contained in the set [0, 1] and
	 * returns r, g, and b in the set [0, 255].
	 *
	 * @param   Number  h       The hue
	 * @param   Number  s       The saturation
	 * @param   Number  l       The lightness
	 * @return  Array           The RGB representation
	 */
	hsl2rgb: function (h, s, l) {
	    var r, g, b;

	    if(s == 0){
		r = g = b = l; // achromatic
	    }else{
		function hue2rgb(p, q, t){
		    if(t < 0) t += 1;
		    if(t > 1) t -= 1;
		    if(t < 1/6) return p + (q - p) * 6 * t;
		    if(t < 1/2) return q;
		    if(t < 2/3) return p + (q - p) * (2/3 - t) * 6;
		    return p;
		}

		var q = l < 0.5 ? l * (1 + s) : l + s - l * s;
		var p = 2 * l - q;
		r = hue2rgb(p, q, h + 1/3);
		g = hue2rgb(p, q, h);
		b = hue2rgb(p, q, h - 1/3);
	    }

	    return [r * 255, g * 255, b * 255];
	}
}
