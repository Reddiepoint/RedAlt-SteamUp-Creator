// ==UserScript==
// @name        RedAlt SteamDB Changelist Grabber
// @namespace   Violentmonkey Scripts
// @match       *://steamdb.info/app/*
// @match       *://steamdb.info/patchnotes/*
// @run-at      document-idle
// @grant       GM_setValue
// @grant       GM_getValue
// @grant       GM_openInTab
// @grant       window.close
// @version     0.2.1
// @author      Reddiepoint
// @description
// ==/UserScript==


// Create the button
const button = document.createElement("button");
button.textContent = "Reset"; // Set the text content of the button
button.id = "myButton"; // Set the ID of the button

// Add event listener to the button
button.addEventListener("click", myFunction);

// Add the button as the first child of the body
document.body.insertBefore(button, document.body.lastElementChild);

// Function to run when the button is clicked
function myFunction() {
    GM_setValue("changesObject", '{ "added": [], "removed": [], "modified": [] }');
    GM_setValue("readyToDownload", false);
    GM_setValue("depotID", null);
    GM_setValue("gettingChangelogs", false);
    // Your code here
    console.log("Reset!");
    // Add your custom functionality here
}

console.log(GM_getValue("changesObject"));
console.log(GM_getValue("depotID"));
if (GM_getValue("gettingChangelogs", false) && window.location.href.includes("steamdb.info/patchnotes/")) {
    (function () {
        const depotID = GM_getValue("depotID", null);
        const depots = document.querySelector(`a[href*="/depot/${depotID}/"]`);
        if (!depots) {
            window.close();
        }

        const observer = new MutationObserver((mutations, observer) => {
            const parentSibling = depots.parentElement.nextElementSibling;
            const li = parentSibling.querySelector('li.versions');
            if (parentSibling && li) {
                const versions = parentSibling.children;
                // Retrieve the existing changelogObject
                const existingChangelogObject = JSON.parse(GM_getValue("changesObject", '{ "added": [], "removed": [], "modified": [] }'));

                for (let i = 0; i < versions.length; i++) {
                    const version = versions[i];
                    if (version.className === "diff-added") {
                        console.log("added");
                        const filePath = version.querySelector("ins").textContent;
                        if (!existingChangelogObject.added.includes(filePath)) {
                            console.log("not included")
                            if (existingChangelogObject.removed.includes(filePath)) {
                                existingChangelogObject.removed = existingChangelogObject.removed.filter(item => item.toString() !== filePath);
                            }
                            existingChangelogObject.added.push(filePath);
                            console.log("added now")
                        }
                    } else if (version.className === "diff-removed") {
                        console.log("removed");
                        const filePath = version.querySelector("del").textContent;
                        if (!existingChangelogObject.removed.includes(filePath)) {
                            if (existingChangelogObject.added.includes(filePath)) {
                                existingChangelogObject.added = existingChangelogObject.added.filter(item => item.toString() !== filePath);
                            }
                            if (existingChangelogObject.modified.includes(filePath)) {
                                existingChangelogObject.modified = existingChangelogObject.modified.filter(item => item.toString() !== filePath);
                            }
                            existingChangelogObject.removed.push(filePath);
                        }

                    } else if (version.className === "diff-modified") {
                        console.log("modified");
                        const filePath = version.querySelector("i").textContent;
                        if (!existingChangelogObject.added.includes(filePath) && !existingChangelogObject.modified.includes(filePath)) {
                            if (existingChangelogObject.removed.includes(filePath)) {
                                existingChangelogObject.removed = existingChangelogObject.removed.filter(item => item !== filePath);
                            }
                            existingChangelogObject.modified.push(filePath);
                        }
                    }
                }

                GM_setValue("changesObject", JSON.stringify(existingChangelogObject));

                console.log(GM_getValue("changesObject"));
                window.close();
                observer.disconnect();
            }
        });

        observer.observe(document, {childList: true, subtree: true});
    })();
}

if (GM_getValue("readyToDownload", false)) {
    const filename = "test.txt";
    const text = GM_getValue("changesObject");
    const element = document.createElement('a');
    element.setAttribute('href', 'data:text/plain;charset=utf-8,' + encodeURIComponent(text));
    element.setAttribute('download', filename);

    element.style.display = 'none';
    document.body.appendChild(element);

    element.click();

    document.body.removeChild(element);

    GM_setValue("readyToDownload", false)
}

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
                <label for="depotID">Depot:</label>
                <input type="text" id="depotID" name="depotID">
                <br>
                <label for="buildID1">Build ID 1:</label>
                <input list="buildID1List" id="buildID1" name="buildID1">
                <datalist id="buildID1List">
                    ${buildIDs.map((id) => `<option value="${id}"></option>`).join('')}
                </datalist>
                <br>
                <label for="buildID2">Build ID 2:</label>
                <input list="buildID2List" id="buildID2" name="buildID2">
                <datalist id="buildID2List">
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
    const button = document.createElement("button");
    button.textContent = "Open Modal";
    button.id = "myBtn";
    button.style.marginTop = "10px"; // Add some spacing

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

    console.log(JSON.parse(GM_getValue("changesObject", '{ "added": [], "removed": [], "modified": [] }')));
})();


function getDiff() {
    const buildID1 = document.getElementById("buildID1").value;
    const buildID2 = document.getElementById("buildID2").value;
    const depotID = document.getElementById("depotID").value;
    const builds = getBuildIDs().reverse();
    // Get slice of builds from buildID1 + 1 to buildID2
    let intermediaryBuilds = builds.slice(builds.indexOf(buildID1) + 1, builds.indexOf(buildID2) + 1);
    console.log(intermediaryBuilds);

    GM_setValue("depotID", depotID);
    GM_setValue("readyToDownload", false);
    GM_setValue("changesObject", '{ "added": [], "removed": [], "modified": [] }');
    // Get changelog for each build
    for (let i = 0; i < intermediaryBuilds.length; i++) {
        console.log(typeof intermediaryBuilds[i]);
        const repeat = setInterval(() => {
            console.log(GM_getValue("gettingChangelogs", false));
            if (!GM_getValue("gettingChangelogs", false)) {
                clearInterval(repeat);
                GM_setValue("gettingChangelogs", true);
                getChangelog(depotID, intermediaryBuilds[i]);
            }
        }, 1000); // Adjust the interval duration as needed
    }
    const repeat = setInterval(() => {
        if (!GM_getValue("gettingChangelogs", false)) {
            GM_setValue("readyToDownload", true);
            clearInterval(repeat);
            console.log("Downloading changes");
            location.reload();
        }
    }, 1000); // Adjust the interval duration as needed


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


function getChangelog(depotID, buildID) {
    console.log(depotID);
    console.log(buildID);
    const url = document.querySelector(`a[href*="/patchnotes/${buildID}"]`).href;
    console.log(url);
    const tab = GM_openInTab(url, {
        active: false
    });
    tab.onclose = () => {
        GM_setValue("gettingChangelogs", false)
    }
}

