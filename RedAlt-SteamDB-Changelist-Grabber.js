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
// @version     0.6.0
// @author      Reddiepoint
// @description
// @updateURL   https://github.com/Reddiepoint/RedAlt-Steam-Update-Creator/raw/main/steamdb_changelist_grabber.user.js
// @downloadURL https://github.com/Reddiepoint/RedAlt-Steam-Update-Creator/raw/main/steamdb_changelist_grabber.user.js
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
    GM_setValue("changesObject", null);
    GM_setValue("readyToDownload", false);
    GM_setValue("depotID", null);
    GM_setValue("manifestID", null);
    GM_setValue("gettingChangelogs", false);
    // Your code here
    console.log("Reset!");
    // Add your custom functionality here
}

console.log(GM_getValue("changesObject"));

if (GM_getValue("gettingChangelogs", false) && window.location.href.includes("steamdb.info/patchnotes/")) {
    (function () {
        const depotID = GM_getValue("depotID", null);
        const depotElement = document.querySelector(`a[href*="/depot/${depotID}/"]`);
        if (!depotElement) {
            window.close();
        }
        const manifestID = depotElement.href.split("M:")[1];
        GM_setValue("manifestID", manifestID);
        const observer = new MutationObserver(async (mutations, observer) => {
            const parentSibling = depotElement.parentElement.nextElementSibling;
            const li = parentSibling.querySelector("li.versions");
            if (parentSibling && li) {
                const versions = parentSibling.children;
                // Retrieve the existing changelogObject
                const existingChangelogObject = JSON.parse(GM_getValue("changesObject"));

                for (let i = 0; i < versions.length; i++) {
                    const version = versions[i];
                    if (version.className === "diff-added") {
                        const filePath = version.querySelector("ins").textContent;
                        if (!existingChangelogObject.added.includes(filePath)) {
                            if (existingChangelogObject.removed.includes(filePath)) {
                                existingChangelogObject.removed = existingChangelogObject.removed.filter(item => item.toString() !== filePath);
                            }
                            existingChangelogObject.added.push(filePath);
                        }
                    } else if (version.className === "diff-removed") {
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
                window.close();
                observer.disconnect();
            }
        });

        observer.observe(document, {childList: true, subtree: true});
    })();
}

if (GM_getValue("readyToDownload", false)) {
    const filename = GM_getValue("depotID") + "_changes.json";
    const changes = JSON.parse(GM_getValue("changesObject"));
    changes.manifest = GM_getValue("manifestID");

    const element = document.createElement("a");
    element.setAttribute("href", "data:text/plain;charset=utf-8," + encodeURIComponent(JSON.stringify(changes)));
    element.setAttribute("download", filename);

    element.style.display = "none";
    document.body.appendChild(element);

    element.click();

    document.body.removeChild(element);

    GM_setValue("readyToDownload", false)
}

if (window.location.href.includes("steamdb.info/app/")) {
    // Add modal CSS
    const css = `
    .modal {
        display: none;
        position: fixed;
        z-index: 2;
        left: 0;
        top: 0;
        width: 100%;
        height: 100%;
        overflow: auto;
        background-color: rgba(0,0,0,0.8); /* Darker background for the overlay */
    }
    
    .modal-content {
        background-color: #333; /* Dark background for the modal */
        color: #ddd; /* Light text colour for readability */
        margin: 15% auto;
        padding: 20px;
        border: 1px solid #444; /* Slightly lighter border colour */
        width: 80%;
    }
    
    .close {
        color: #aaa; /* Lighter colour for the close button */
        float: right;
        font-size: 28px;
        font-weight: bold;
    }
    
    .close:hover,
    .close:focus {
        color: white; /* Even lighter colour on hover/focus for contrast */
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
    // Create modal content HTML
    const modalHTML = `
    <div id="changelogModal" class="modal" style="display: none;">
        <div class="modal-content">
            <span class="close">&times;</span>
            <form id="buildForm">
                <label for="depotID">Depot:</label>
                <input type="text" id="depotID" name="depotID">
                <br>
                <label for="buildID1">Get changes from: </label>
                <input list="buildID1List" id="buildID1" name="buildID1">
                <datalist id="buildID1List">
                    ${buildIDs.map((id) => `<option value="${id}"></option>`).join("")}
                </datalist>
                <label for="buildID2"> to </label>
                <input list="buildID2List" id="buildID2" name="buildID2">
                <datalist id="buildID2List">
                    ${buildIDs.map((id) => `<option value="${id}"></option>`).join("")}
                </datalist>
                <br>
                <button type="button" id="getChangesBtn">Get changes</button>
            </form>
        </div>
    </div>`;

    // Append modal to body
    document.body.insertAdjacentHTML("beforeend", modalHTML);

    // Add button
    const refElement = document.querySelector("#main > div.container > div:nth-child(5) > a");
    const newDiv = document.createElement("div");
    const button = document.createElement("button");
    button.textContent = "Get changes";
    button.id = "modalButton";
    newDiv.style.marginTop = "10px"; // Add some spacing
    if (refElement) {
        refElement.parentNode.insertBefore(newDiv, refElement.nextSibling);
        newDiv.appendChild(refElement);
        newDiv.appendChild(document.createElement("br"));
        newDiv.appendChild(button);
    }

    // Modal interaction
    const modal = document.getElementById("changelogModal");
    const span = document.getElementsByClassName("close")[0];

    button.onclick = function () {
        modal.style.display = "block";
    };

    span.onclick = function () {
        modal.style.display = "none";
    };

    window.onclick = function (event) {
        if (event.target === modal) {
            modal.style.display = "none";
        }
    };

    document.getElementById("getChangesBtn").addEventListener("click", getChanges);
}


function getChanges() {
    const getChangesBtn = document.querySelector("#getChangesBtn");

    let title = document.querySelector("#main > div.patchnotes-header > div > div:nth-child(1) > h1 > a");
    const appName = title.textContent;
    const appID = title.getAttribute("data-appid")

    let buildID1 = document.getElementById("buildID1").value;
    let buildID2 = document.getElementById("buildID2").value;

    if (buildID1 >= buildID2) {
        // Switch ID1 and ID2
        document.getElementById("buildID1").value = buildID2;
        document.getElementById("buildID2").value = buildID1;
        const temp = buildID1;
        buildID1 = buildID2;
        buildID2 = temp;

    }

    const depotID = document.getElementById("depotID").value;
    const builds = getBuildIDs().reverse();
    // Get slice of builds from buildID1 + 1 to buildID2
    let intermediaryBuilds = builds.slice(builds.indexOf(buildID1) + 1, builds.indexOf(buildID2) + 1);

    // Build and Depot Check
    if (!depotID) {
        getChangesBtn.insertAdjacentHTML('afterend', "<p>Please specify a depot ID.</p>");
        setTimeout(() => {
            getChangesBtn.nextElementSibling.remove();
        }, 2000);
        return;
    } else if (!buildID1 || !buildID2) {
        getChangesBtn.insertAdjacentHTML('afterend', "<p>Invalid Build ID.</p>");

        setTimeout(() => {
            getChangesBtn.nextElementSibling.remove();
        }, 2000);
        return;
    }


    GM_setValue("depotID", depotID);
    GM_setValue("readyToDownload", false);
    const changesObject = {
        name: appName,
        app: appID,
        depot: depotID,
        initial_build: buildID1,
        final_build: buildID2,
        added: [],
        removed: [],
        modified: []
    }
    GM_setValue("changesObject", JSON.stringify(changesObject));


    getChangesBtn.insertAdjacentHTML('afterend', '<p>Getting changes... This page will refresh automatically to download the changes.</p>');

    // Get changelog for each build
    for (let i = 0; i < intermediaryBuilds.length; i++) {
        const repeat = setInterval(() => {
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
            location.reload();
            clearInterval(repeat);
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

function getChangelog(depotID, buildID) {
    const url = document.querySelector(`a[href*="/patchnotes/${buildID}"]`).href;
    const tab = GM_openInTab(url, {
        active: true
    });
    tab.onclose = () => {
        GM_setValue("gettingChangelogs", false)
    }
}
