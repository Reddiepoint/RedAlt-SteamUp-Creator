// ==UserScript==
// @name        RedAlt SteamDB Changelist Grabber
// @namespace   Violentmonkey Scripts
// @match       *://steamdb.info/app/*
// @run-at      document-idle
// @require     https://ajax.googleapis.com/ajax/libs/jquery/3.7.1/jquery.min.js
// @grant       GM_xmlhttpRequest
// @version     0.1
// @author      Reddiepoint
// @description
// ==/UserScript==

const url = document.querySelector("#js-builds > tr:nth-child(1) > td:nth-child(4) > a").href;
console.log(url);

getChangeLog(url);

function getChangeLog(url) {

}
