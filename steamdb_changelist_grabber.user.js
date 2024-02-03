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


(function () {
    // Add modal CSS
    const css = `
    .modal {
        display: none;
        position: fixed;
        z-index: 1;
        left: 0;
        top: 0;
        width: 100%;
        height: 100%;
        overflow: auto;
        background-color: rgba(0,0,0,0.8); /* Darker background for the overlay */
    }
    
    .modal-content {
        background-color: #333; /* Dark background for the modal */
        color: #ddd; /* Light text color for readability */
        margin: 15% auto;
        padding: 20px;
        border: 1px solid #444; /* Slightly lighter border color */
        width: 80%;
    }
    
    .close {
        color: #aaa; /* Lighter color for the close button */
        float: right;
        font-size: 28px;
        font-weight: bold;
    }
    
    .close:hover,
    .close:focus {
        color: white; /* Even lighter color on hover/focus for contrast */
        text-decoration: none;
        cursor: pointer;
    }
    
    input {
      width: 20%;
      padding: 12px 20px;
      margin: 8px 0;
      display: inline-block;
      border: 1px solid #ccc;
      border-radius: 4px;
      box-sizing: border-box;
    }
    `;

    const styleSheet = document.createElement("style");
    styleSheet.innerText = css;
    document.head.appendChild(styleSheet);

    const buildIDs = getBuildIDs();
    // Create modal HTML
    const modalHTML = `
    <div id="myModal" class="modal" style="display: none;">
        <div class="modal-content">
            <span class="close">&times;</span>
            <form id="buildForm">
                <label for="depot">Depot:</label>
                <input type="text" id="depot" name="depot">
                <br>
                <label for="buildId1">Build ID 1:</label>
                <input list="buildId1List" id="buildId1" name="buildId1">
                <datalist id="buildId1List">
                    ${buildIDs.map((id) => `<option value="${id}"></option>`).join('')}
                </datalist>
                <br>
                <label for="buildId2">Build ID 2:</label>
                <input list="buildId2List" id="buildId2" name="buildId2">
                <datalist id="buildId2List">
                    ${buildIDs.map((id) => `<option value="${id}"></option>`).join('')}
                </datalist>
                <br>
                <button type="button" id="getDiffBtn">Get diff</button>
            </form>
        </div>
    </div>`;

    // Append modal to body
    document.body.insertAdjacentHTML('beforeend', modalHTML);

    // Create the button
    const button = document.createElement('button');
    button.textContent = 'Open Modal';
    button.id = 'myBtn';
    button.style.marginTop = '10px'; // Add some spacing

    // Get the reference element and insert the button
    const refElement = document.querySelector("#main > div.container > div:nth-child(5) > a");
    if (refElement) {
        refElement.parentNode.insertBefore(button, refElement.nextSibling);
    }

    // Modal interaction script
    const modal = document.getElementById('myModal');
    const span = document.getElementsByClassName('close')[0];

    button.onclick = function () {
        modal.style.display = 'block';
    };

    span.onclick = function () {
        modal.style.display = 'none';
    };

    window.onclick = function (event) {
        if (event.target === modal) {
            modal.style.display = 'none';
        }
    };

    document.getElementById('getDiffBtn').addEventListener('click', getDiff);
})();


function getDiff() {
    const buildId1 = document.getElementById("buildId1").value;
    const buildId2 = document.getElementById("buildId2").value;
    const builds = getBuildIDs();
    // Get slice of builds from buildId1 to buildId2
    let intermediaryBuilds = builds.slice(builds.indexOf(buildId1), builds.indexOf(buildId2) + 1);
    console.log(intermediaryBuilds);

    // Get changelog for each build
    for (let i = 0; i < intermediaryBuilds.length; i++) {
        console.log(intermediaryBuilds[i]);
        const changelog = getChangelog(intermediaryBuilds[i]);
    }
}

function getBuildIDs() {
    const builds = [];
    const jsBuilds = document.querySelector("#js-builds");
    const trElements = jsBuilds.querySelectorAll("tr");
    trElements.forEach((tr) => {
        const version = tr.querySelector("td:last-child");
        if (version) {
            builds.push(version.textContent);
        }
    });
    return builds;
}

const url = document.querySelector("#js-builds > tr:nth-child(1) > td:nth-child(4) > a").href;
console.log(url);


function getChangelog(buildId) {
    // Find element with href including /patchnotes/buildId
    const url = document.querySelector(`a[href*="/patchnotes/${buildId}"]`).href;
    console.log(url);
}
