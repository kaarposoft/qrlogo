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
/*jslint plusplus: true */
/*global alert: true */


/* ************************************************************ */
/* JavaScript STRICT mode
*/

"use strict";


/* ************************************************************
 * ON LOAD
 * ************************************************************
 */

function onTestLoad() {
	document.getElementById('nojs').style.display = "none";
	if (Modernizr.canvas) { document.getElementById('noHTML5canvas').style.display = "none"; }

	var pictures = new Pictures();
	pictures.init();
	pictures.show();
}


/* ************************************************************
 * PICTURES
 * ************************************************************
 */

function Pictures() {
}

Pictures.prototype = {

    /* ************************************************************ */
    init: function() {

        var pictures = new Array(5);
        var i, pic;
        for (i = 0; i < pictures.length; i++) pictures[i] = [];

        var texts = [];
        texts[0] = "One Two Three Four Five";
        texts[1] = texts[0] + "\nSix Seven Eight Nine Ten";
        texts[2] = texts[1] + "\nEleven Twelve Thirteen Fourteen Fifteen";
        texts[3] = texts[2] + "\nSixteen Seventeen Eighteen Nineteen Twenty";

        for (i = 0; i < 4; i++) {
            pic = {
                text: texts[i],
                version: 3 + i,
                mode: 4,
                ec: 'L',
                ppm: 6 - i / 2
            };
            pictures[0].push(pic);
            pic = {
                text: texts[i],
                version: 5 + i,
                mode: 4,
                ec: 'M',
                ppm: 5 - i / 2
            };
            pictures[1].push(pic);
            pic = {
                text: texts[i],
                version: 7 + i,
                mode: 4,
                ec: 'Q',
                ppm: 5 - i / 2
            };
            pictures[2].push(pic);
            pic = {
                text: texts[i],
                version: 8 + i,
                mode: 4,
                ec: 'H',
                ppm: 5 - i / 2
            };
            pictures[3].push(pic);
            pic = {
                text: texts[i].replace(new RegExp("\n", "g"), "").toUpperCase(),
                version: 4 + i,
                mode: 2,
                ec: 'M',
                ppm: 5 - i / 2
            };
            pictures[4].push(pic);
        }
        this.pictures = pictures;
    },

    /* ************************************************************ */
    show: function() {
        var div = document.getElementById('div_test_pictures');
        var pictures = this.pictures;
        var qr = new QRCodeDecode();
        var r, c;
        var table, tr, td, pic, canvas, t;
        for (r = 0; r < pictures.length; r++) {
            table = document.createElement('table');
            table.className = "table_borders";

            tr = document.createElement('tr');
            for (c = 0; c < pictures[r].length; c++) {
                pic = pictures[r][c];
                td = document.createElement('td');
                canvas = document.createElement('canvas');
                t = pic.text;
                if (pic.mode == 2) t = pic.text.toUpperCase();
                qr.encodeToCanvas(pic.mode, t, pic.version, qr.ERROR_CORRECTION_LEVEL[pic.ec], pic.ppm, canvas);
                td.appendChild(canvas);
                tr.appendChild(td);
            }
            table.appendChild(tr);

            tr = document.createElement('tr');
            for (c = 0; c < pictures[r].length; c++) {
                pic = pictures[r][c];
                td = document.createElement('td');
                td.innerHTML = htmlEscape(pic.text);
                tr.appendChild(td);
            }
            table.appendChild(tr);

            tr = document.createElement('tr');
            for (c = 0; c < pictures[r].length; c++) {
                pic = pictures[r][c];
                td = document.createElement('td');
                td.innerHTML = "version=" + pic.version + "<br>mode=" + pic.mode + "<br>ec=" + pic.ec;
                tr.appendChild(td);
            }
            table.appendChild(tr);

            div.appendChild(table);
        }
    }
};

