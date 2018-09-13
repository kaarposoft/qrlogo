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
/*jslint continue: true */
/*global alert: true */
/*global Modernizr: true */
/*global htmlEscape: true */
/*global QRCodeDecode: true */


/* ************************************************************ */
/* JavaScript STRICT mode
*/

"use strict";

/* ************************************************************
 * GLOBALS
 * ************************************************************
 */
var global_tests;


/* ************************************************************
 * TESTS
 * ************************************************************
 */

function Tests() {
}

Tests.prototype = {

	/* ************************************************************ */
	init: function() {
		this.groups = [];
		this.start_button = document.getElementById('button_test_start');
		this.stop_button = document.getElementById('button_test_stop');
		this.div_test_overview = document.getElementById('div_test_overview');
		this.div_test_results = document.getElementById('div_test_results');

		var use_visible_canvas = false;
		if (use_visible_canvas) {
			this.canvas = document.getElementById('test_canvas');
		} else {
			this.canvas = document.createElement('canvas');
		}
		this.ctx = this.canvas.getContext('2d');
	},

	/* ************************************************************ */
	ready: function() {
		this.start_button.disabled = false;
		this.stop_button.disabled = true;
	},

	/* ************************************************************ */
	onTestStart: function() {
		this.start_button.disabled = true;
		this.stop_button.disabled = false;
		this.shouldStop = false;
		this.runTests();
	},

	/* ************************************************************ */
	onTestStop: function() {
		this.shouldStop = true;
		this.start_button.disabled = false;
		this.stop_button.disabled = true;
	},

	/* ************************************************************ */
	addGroup: function(group) {
		this.groups.push(group);
	},

	/* ************************************************************ */
	showTests: function() {

		var table, tr, th, td, g, group, t, test;

		table = document.createElement('table');
		table.className="table_bold_borders";

		tr = document.createElement('tr');
		th = document.createElement('th');
		th.innerHTML="Test group";
		tr.appendChild(th);
		th = document.createElement('th');
		th.innerHTML="# Total";
		tr.appendChild(th);
		th = document.createElement('th');
		th.innerHTML="# OK";
		tr.appendChild(th);
		th = document.createElement('th');
		th.innerHTML="# Fail";
		tr.appendChild(th);
		table.appendChild(tr);

		for (g = 0; g < this.groups.length; g++) {

			group = this.groups[g];
			tr = document.createElement('tr');
			td = document.createElement('td');
			td.innerHTML = group.title;
			tr.appendChild(td);

			td = document.createElement('td');
			group.n_total_element = td;
			tr.appendChild(td);

			td = document.createElement('td');
			group.n_success_element = td;
			tr.appendChild(td);

			td = document.createElement('td');
			group.n_fail_element = td;
			tr.appendChild(td);

			group.showCount();

			table.appendChild(tr);
		}
		this.div_test_overview.appendChild(table);

		for (g = 0; g < this.groups.length; g++) {

			group = this.groups[g];

			table = document.createElement('table');
			table.className = "table_bold_borders";

			tr = document.createElement('tr');

			th = document.createElement('th');
			th.setAttribute('colspan', '2');
			th.innerHTML = group.title;
			tr.appendChild(th);
			table.appendChild(tr);

			for (t = 0; t < group.tests.length; t++) {
				test = group.tests[t];

				tr = document.createElement('tr');

				td = document.createElement('td');
				//td.innerHTML = "AAA " + this.groups[g].title;
				td.innerHTML = test.title;
				tr.appendChild(td);
				td = document.createElement('td');
				tr.appendChild(td);

				test.status_element = td;

			table.appendChild(tr);
			}
			this.div_test_results.appendChild(table);
		}
	},

	/* ************************************************************ */

	current_group: 0,
	current_test: 0,

	runTests: function() {

		var g, group, t, test;

		for (g = 0; g < this.groups.length; g++) {
			group = this.groups[g];
			group.n_success=0;
			group.n_fail=0;
			group.showCount();
			for (t = 0; t < group.tests.length; t++) {
				test = group.tests[t];
				test.status = "";
				test.status_element.setAttribute('class', '');
				test.status_element.innerHTML = test.status;
			}
		}
		this.current_group = 0;
		this.current_test = 0;
		setTimeout(function () { global_tests.groups[global_tests.current_group].tests[global_tests.current_test].execute(global_tests); }, 0);
	},

	/* ************************************************************ */
	oneTestComplete: function(test, success) {

		test.status_element.innerHTML=test.status;

		if (success) {
			test.group.n_success++;
			test.status_element.setAttribute('class', 'testOK');
		} else {
			test.group.n_fail++;
			test.status_element.setAttribute('class', 'testFAIL');
		}

		test.group.showCount();

		if (this.shouldStop) { return; }

		if (this.current_test<this.groups[this.current_group].tests.length-1) {
			this.current_test++;
		} else {
			if (this.current_group<this.groups.length-1) {
				this.current_test = 0;
				this.current_group++;
			} else {
				this.onTestStop();
				return;
			}
		}
		setTimeout(function () { global_tests.groups[global_tests.current_group].tests[global_tests.current_test].execute(global_tests); }, 0);
	}
}; // Tests


/* ************************************************************
 * TEST GROUP
 * ************************************************************
 */

function TestGroup(title) {
	this.title = title;
	this.tests = [];
	this.n_success = 0;
	this.n_fail = 0;
}

TestGroup.prototype = {

	appendTest: function (t) {
		t.group = this;
		this.tests.push(t);
	},

	showCount: function() {
		this.n_total_element.innerHTML = this.tests.length.toString();
		this.n_success_element.innerHTML = this.n_success.toString();
		this.n_fail_element.innerHTML = this.n_fail.toString();

		if (this.n_success === this.tests.length) {
			this.n_success_element.setAttribute('class', 'testgroupOK');
		} else {
			this.n_success_element.setAttribute('class', '');
		}

		if (this.n_fail > 0) {
			this.n_fail_element.setAttribute('class', 'testgroupFAIL');
		} else {
			this.n_fail_element.setAttribute('class', '');
		}
	}
};


/* ************************************************************
 * URL TEST
 * ************************************************************
 */

function URLTest(title, url, text) {
	this.title = title;
	this.url = url;
	this.text = text;
}

URLTest.prototype = {

	validate: function(image) {
		var canvas = global_tests.canvas;
		var ctx = global_tests.ctx;
		canvas.width=image.width;
		canvas.height=image.height;
		ctx.drawImage(image,0,0); 
		var imagedata = ctx.getImageData(0, 0, canvas.width, canvas.height);

		var qr = new QRCodeDecode();
		var decoded = qr.decodeImageData(imagedata, canvas.width, canvas.height);

		if (decoded === this.text) {
			this.status = "OK";
			this.success = true;
		} else {
			this.status = "decoded wrong text";
			this.success = false;
		}
	},

	execute: function (tests) {
		this.success = false;
		this.status="executing";
		var image = new Image();
		image.test = this;
		image.onload = function() {
			try {
				this.test.validate(image);
			} catch (err) {
				this.test.status = "exception: " + htmlEscape(err);
			}
			tests.oneTestComplete(this.test, this.test.success);
		};
		image.onerror = function() {
			this.test.status="download error";
			tests.oneTestComplete(this.test, false);
		};
		image.ontimeout = function() {
			this.test.status="timeout error";
			tests.oneTestComplete(this.test, false);
		};

		image.src=this.url;
	}
};

/* ************************************************************
 * INLINE CANVAS TEST
 * ************************************************************
 */

function InlineCanvasTest(title, text, mode, version, errorcorrection) {
	this.title = title;
	if (mode === 2) { this.text = text.toUpperCase(); } else { this.text=text; }
	this.mode = mode;
	this.version = version;
	this.errorcorrection = errorcorrection;
}

InlineCanvasTest.prototype = {

	encodedecode: function() {

		var canvas = global_tests.canvas;
		var ctx = global_tests.ctx;

		var qr = new QRCodeDecode();
		qr.encodeToCanvas(this.mode, this.text, this.version, this.errorcorrection, 1, canvas);

		var imagedata = ctx.getImageData(0, 0, canvas.width, canvas.height);

		qr = new QRCodeDecode();
		var decoded = qr.decodeImageData(imagedata, canvas.width, canvas.height);

		if (decoded === this.text) {
			this.status="OK";
			this.success = true;
		} else {
			this.status="decoded wrong text";
			this.success = false;
		}
	},

	execute: function (tests) {
		this.success = false;
		this.status="executing";
		try {
			this.encodedecode();
		} catch (err) {
			this.status="exception: " + htmlEscape(err);
		}
		tests.oneTestComplete(this, this.success);
	}
};


/* ************************************************************
 * INLINE PIXMAP TEST
 * ************************************************************
 */

function InlinePixarrayTest(title, text, mode, version, errorcorrection) {
	this.title = title;
	if (mode === 2) { this.text = text.toUpperCase(); } else { this.text=text; }
	this.mode = mode;
	this.version = version;
	this.errorcorrection = errorcorrection;
}

InlinePixarrayTest.prototype = {

	encodedecode: function() {

		var qr = new QRCodeDecode();
		var pix = qr.encodeToPixarray(this.mode, this.text, this.version, this.errorcorrection);
		qr = new QRCodeDecode();
		var decoded = qr.decodePixarray(pix);

		if (decoded === this.text) {
			this.status="OK";
			this.success = true;
		} else {
			this.status="decoded wrong text";
			this.success = false;
		}
	},

	execute: function (tests) {
		this.success = false;
		this.status="executing";
		try {
			this.encodedecode();
		} catch (err) {
			this.status="exception: " + htmlEscape(err);
		}
		tests.oneTestComplete(this, this.success);
	}
};


/* ************************************************************
 * INLINE ERROR CORRECTION TEST
 * ************************************************************
 */

function InlineErrorcorrectionTest(title, text, mode, version, errorcorrection, dist, magnitude, should_fail) {
	this.title = title;
	if (mode === 2) { this.text = text.toUpperCase(); } else { this.text=text; }
	this.mode = mode;
	this.version = version;
	this.errorcorrection = errorcorrection;
	this.dist = dist;
	this.magnitude = magnitude;
	this.should_fail = should_fail;
}


InlineErrorcorrectionTest.prototype = {

	encodedecode: function() {

		var qr = new QRCodeDecode();
		var pix = qr.encodeToPixarray(this.mode, this.text, this.version, this.errorcorrection);

		var n_modules = qr.nModulesFromVersion(this.version);

		var m, i, j;
		if (this.dist === "SE") {
			for (m = 0; m < this.magnitude; m++) {
				for (i = 0; i < m; i++) {
					for (j = 0; j < m; j++) {
						pix.arr[4+n_modules-i-1][4+n_modules-j-1] = false;
					}
				}
			}
		} else if (this.dist === "WEDGE") {
			for (i = n_modules-this.magnitude; i < n_modules; i++) {
				for (j = 9; j < 18; j++) {
					pix.arr[4+i][4+j]=false;
					pix.arr[4+j][4+i]=false;
				}
			}
		} else {
			alert("Unknown distortion: " + this.dist);
		}

		qr = new QRCodeDecode();

		var decoded;
		if (this.should_fail) {
			try {
				decoded = qr.decodePixarray(pix);
				this.status = "Expected failure, but tested OK?";
				this.success = false;
			} catch (e) {
				this.status = "Failed as expected";
				this.success = true;
			}
		} else {
			decoded = qr.decodePixarray(pix);
			if (decoded === this.text) {
				this.status="OK ec=" + qr.n_block_ec_words + " egrade=" + qr.error_grade + " err= " + qr.errors.join(",");
				this.success = true;
			} else {
				this.status="decoded wrong text ec=" + qr.n_block_ec_words +
					" err= " + qr.errors.join(","); 
				this.success = false;
			}
		}
	},

	execute: function (tests) {
		this.success = false;
		this.status="executing";
		try {
			this.encodedecode();
		} catch (err) {
			this.status="exception: " + htmlEscape(err);
		}
		tests.oneTestComplete(this, this.success);

	}
};


/* ************************************************************
 * ALL TESTS
 * ************************************************************
 */

function addAllTests(tests) {

	function predefinedTestGroup() {
		var tg = new TestGroup("Decode predefined codes");

		// http://www.moongate.ro/en/products/qr_code-vcard/
		var jd="BEGIN:VCARD\n\
VERSION:2.1\n\
N:Doe;John;;Mr\n\
FN:John Doe\n\
ADR;HOME:;;Memory Lane 42;Memphis;TN;38111\n\
TEL;HOME:007 911\n\
URL:http://kaarpo.dk\n\
END:VCARD\n";

		tg.appendTest(new URLTest("John Doe VCARD", "../testpics/vcard_john_doe.png", jd));

		var hamlet = "To be, or not to be: that is the question:\n\
Whether 'tis nobler in the mind to suffer\n\
The slings and arrows of outrageous fortune,\n\
Or to take arms against a sea of troubles,\n\
And by opposing end them? To die: to sleep;\n\
No more; and by a sleep to say we end\n\
The heart-ache and the thousand natural shocks\n\
That flesh is heir to, 'tis a consummation\n\
Devoutly to be wish'd. To die, to sleep;\n\
To sleep: perchance to dream: ay, there's the rub;\n\
For in that sleep of death what dreams may come\n\
When we have shuffled off this mortal coil,\n\
Must give us pause: there's the respect\n\
That makes calamity of so long life;\n\
For who would bear the whips and scorns of time,\n\
The oppressor's wrong, the proud man's contumely,\n\
The pangs of despised love, the law's delay,\n\
The insolence of office and the spurns\n\
That patient merit of the unworthy takes,\n\
When he himself might his quietus make\n\
With a bare bodkin? who would fardels bear,\n\
To grunt and sweat under a weary life,\n\
But that the dread of something after death,\n\
The undiscover'd country from whose bourn\n\
No traveller returns, puzzles the will\n\
And makes us rather bear those ills we have\n\
Than fly to others that we know not of?\n\
Thus conscience does make cowards of us all;\n\
And thus the native hue of resolution\n\
Is sicklied o'er with the pale cast of thought,\n\
And enterprises of great pith and moment\n\
With this regard their currents turn awry,";

		tg.appendTest(new URLTest("Hamlet", "../testpics/hamlet.png", hamlet));

		// http://hackaday.com/2011/08/11/how-to-put-your-logo-in-a-qr-code/
		tg.appendTest(new URLTest("Firefox", "../testpics/ffox.png", "http://www.mozilla.com/firefox"));
		tg.appendTest(new URLTest("Hackaday", "../testpics/hackaday.png", "http://www.hackaday.com"));

		var ecc = "Lecture by Dr Tim Kindberg \r\n\
6pm Wednesday 10 December MC001\r\n\
\r\n\
http://www.champignon.net/TimKindberg/";

		tg.appendTest(new URLTest("qr-code-error-correction", "../testpics/qr-code-error-correction.gif", ecc));

		tg.appendTest(new URLTest("qr_logo_01", "../testpics/qr_logo_01.png", "http://qrlogo.kaarposoft.dk"));
		tg.appendTest(new URLTest("qr_kaarpo", "../testpics/qr_kaarpo.png", "http://www.kaarpo.dk"));
		tg.appendTest(new URLTest("qr_kaarpo45", "../testpics/qr_kaarpo45.png", "http://www.kaarpo.dk"));
		tg.appendTest(new URLTest("qr_firefox", "../testpics/qr_firefox.png", "http://www.mozilla.com/firefox/"));

		return tg;
	}

	function inlineCanvasTestGroup() {
		var tg = new TestGroup("Inline canvas encode+decode");
		var tbase = "OgHerKommerJeg$/:123";
		var text = tbase;
		var nbase = "0112223333444445555556666666777777778888888889999999999";
		var n = nbase;
		var ec = 0;
		var version;
		for (version = 2; version <= 40; version +=3 ) {
			tg.appendTest(new InlineCanvasTest("8bit len="+text.length+" version="+version+" ec="+ec, text, 4, version, ec));
			tg.appendTest(new InlineCanvasTest("anum len="+text.length+" version="+version+" ec="+ec, text, 2, version, ec));
			tg.appendTest(new InlineCanvasTest("num len="+n.length+" version="+version+" ec="+ec, n, 1, version, ec));
			n += nbase;
			text = "Sporvogn" + text + tbase;
			ec = (ec+1)%4;
		}
		return tg;
	}

	function inlinePixarrayTestGroup() {
		var tg = new TestGroup("Inline pixarray encode+decode");
		var tbase = "\
Lorem ipsum dolor sit amet/ consectetur adipiscing elit.\
Vestibulum sit amet odio aliquet/ mollis erat et/ lobortis nibh.\
Nullam in orci et libero luctus posuere.\
Fusce dignissim hendrerit odio/ ut interdum lacus condimentum at.\
Quisque tempor ultricies sapien/ consectetur scelerisque velit.\
Class aptent taciti sociosqu ad litora torquent per conubia nostra/ per inceptos himenaeos.\
Mauris a tellus diam. Donec nec nisl nisl.\
Integer non lacus posuere/ aliquet diam ultrices/ tristique sem.\
Nulla scelerisque libero nec ligula malesuada/ sit amet pellentesque orci pellentesque.\
Maecenas eget lorem consectetur nulla sagittis tempor.\
Proin mattis sapien mauris/ vel sollicitudin enim mattis ac.\
Cras eleifend lorem et finibus hendrerit.\
Donec in magna dui.\
Cras luctus vitae velit vel gravida.\
Vestibulum convallis eros nec turpis malesuada lacinia.\
In a justo ullamcorper/ pulvinar dolor molestie/ aliquam ante.\
Maecenas lacinia viverra sapien id molestie.\
Vivamus et tempor massa/ eget pulvinar dui.\
Donec egestas sapien sit amet elit consectetur/ ut mattis purus elementum.\
Proin aliquet sollicitudin tempor.\
Nullam finibus est nec tortor ullamcorper tincidunt.\
Integer commodo ac ex vitae convallis.\
Fusce malesuada a orci ac efficitur.\
Nam interdum suscipit urna sit amet pellentesque volutpat.\
";
		var errorcorrection;
		for (errorcorrection = 0; errorcorrection <= 3; errorcorrection++) {
		        //var nbase = "2030507011013017";
		        var nbase = "235711131719";
                        var n = nbase;
                        var version;
			for (version = 1; version <= 40; version++ ) {
				var text = tbase.slice(version, version*version*3.0/4.0+5+2*version);
				tg.appendTest(new InlinePixarrayTest("8bit len="+text.length+" version="+version+" ec="+errorcorrection, text, 4, version, errorcorrection));
				tg.appendTest(new InlinePixarrayTest("anum len="+text.length+" version="+version+" ec="+errorcorrection, text, 2, version, errorcorrection));
				tg.appendTest(new InlinePixarrayTest("num len="+n.length+" version="+version+" ec="+errorcorrection, n, 1, version, errorcorrection));
                                n += nbase + version + version + version + version;
			}
		}
		return tg;
	}


	function inlineErrorcorrectionTestGroup() {

		var tg = new TestGroup("Inline Error correction test");

		var text = "";
		var i;

		for (i = 32; i <= 126; i++) { text += String.fromCharCode(i); }
		for (i = 0; i < 5; i++) { text += text; }
		// we should now have a text with length greater than 2.953 which is max capacity for 8 bit mode

		var qr = new QRCodeDecode();

		var mode = qr.MODE['EightBit'];
		var version, ec, magnitude;

		for (version = 1; version <= 40; version++) {
			for (ec = 0; ec < 4; ec++) {
				var len = qr.getDataCapacity(version, ec, mode);
				var t = text.slice(0, len);

				for (magnitude = version-1; magnitude < Math.floor(1.2*version); magnitude += 4) {
					tg.appendTest(new InlineErrorcorrectionTest(
						"SE-OK len="+t.length+" version="+version+" ec="+ec+" mag="+magnitude,
						 t, mode, version, ec, "SE", magnitude, false));
				}

				var lim;
				if (ec>1) { lim = 3*version-1; }
				else { lim = version; }
				if (version < 9) { lim = Math.floor(lim/2); }
				var lim0 = lim-2;
				if (lim0 < 1) { lim0=1; }
				for (magnitude = lim0; magnitude <= lim; magnitude += 1) {
					tg.appendTest(new InlineErrorcorrectionTest(
						"WEDGE-OK len="+t.length+" version="+version+" ec="+ec+" mag="+magnitude,
						 t, mode, version, ec, "WEDGE", magnitude, false));
				}


				if (version<6) { continue; }
				if (version>9) { continue; }
				lim = 17+4*version - 7;

				for (magnitude = lim; magnitude <= lim+4; magnitude++) {
					tg.appendTest(new InlineErrorcorrectionTest(
						"WEDGE-FAIL len="+t.length+" version="+version+" ec="+ec+" mag="+magnitude,
						 t, mode, version, ec, "WEDGE", magnitude, true));
				}
			}
		}
		return tg;
	}

	tests.addGroup(predefinedTestGroup());
	tests.addGroup(inlineCanvasTestGroup());
	tests.addGroup(inlineErrorcorrectionTestGroup());
	tests.addGroup(inlinePixarrayTestGroup());
}


/* ************************************************************
 * ON LOAD
 * ************************************************************
 */

function onTestLoad() {
	document.getElementById('nojs').style.display = "none";
	if (Modernizr.canvas) {
		document.getElementById('noHTML5canvas').style.display = "none";

		global_tests = new Tests();
		global_tests.init();
		addAllTests(global_tests);
		global_tests.showTests();
		global_tests.ready();
	}
}
