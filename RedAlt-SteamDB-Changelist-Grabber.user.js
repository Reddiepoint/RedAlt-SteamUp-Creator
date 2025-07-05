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
// @version     2.0.0
// @author      Reddiepoint
// @description Aggregates the changes for a specified depot between different builds.
// @updateURL   https://github.com/Reddiepoint/RedAlt-Steam-Update-Creator/raw/main/RedAlt-SteamDB-Changelist-Grabber.user.js
// @downloadURL https://github.com/Reddiepoint/RedAlt-Steam-Update-Creator/raw/main/RedAlt-SteamDB-Changelist-Grabber.user.js
// ==/UserScript==


//region Global Vars
const appName = document.querySelector("#main > div > div.header-wrapper > div > div.pagehead > div.pagehead-title > h1")
    ?.firstChild.textContent.trim() ?? "Unknown";

const appID = document.querySelector('table tr:first-child td:nth-child(2)')
    ?.textContent.trim() ?? "Unknown";
console.log(appName, " ", appID);


const baseURL = window.location.origin + window.location.pathname.replace(/\/+$/, '').replace(/\/[^\/]+$/, '');
console.log(baseURL);
//endregion

//region UI

// Add CSS
function addCSS() {
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
    
    #depotIDList {
        width: 700px;         /* wider dropdown */
        height: 300px;        /* taller dropdown (more visible items) */
        padding: 8px;         /* spacing inside the dropdown */
        box-sizing: border-box;
        overflow-x: auto;
        white-space: nowrap;
    }
    `;

    const styleSheet = document.createElement("style");
    styleSheet.innerText = css;
    document.head.appendChild(styleSheet);
}

addCSS();

// Setup UI
function createResetButton() {
    const button = document.createElement("button");
    button.textContent = "Reset RedAlt";
    button.id = "redaltReset";
    button.addEventListener("click", () => {
        localStorage.removeItem("changesObject");
        GM_setValue("gettingChangelogs", false);
        GM_setValue("gettingDepots", false)
        GM_setValue("buildIndex", 0);
        console.log("Reset!");
    });
    document.body.insertBefore(button, document.body.lastElementChild);
}

createResetButton();

function createModal() {
    const buildIDs = getBuildIDs();
    getDepots(); //Change block to none
    const waitForDepots = () => {
        const depotIDs = getChangesObject().depots;
        if (depotIDs.length === 0) {
            setTimeout(waitForDepots, 100);
        } else {
            console.log("Modal depots:", depotIDs);
            const modalHTML = `
    <div id="changesModal" class="modal" style="display: block;">
        <div class="modal-content">
            <span class="close">&times;</span>
            <form id="buildForm">
                <label for="depotIDList">Depot:</label>
                <select id="depotIDList" multiple>
                    ${depotIDs.map((depot) => `<option value="${depot.id}">${depot.id + ": " + depot.description}</option>`).join("")}
                </select>
                <br>
                <label for="buildID1">Get changes from: </label>
                <input list="buildID1List" id="buildID1" name="buildID1">
                <datalist id="buildID1List">
                    ${buildIDs.slice(1).map((id) => `<option value="${id}"></option>`).join("")}
                </datalist>
                <label for="buildID2"> to </label>
                <input list="buildID2List" id="buildID2" name="buildID2">
                <datalist id="buildID2List">
                    ${buildIDs.map((id) => `<option value="${id}"></option>`).join("")}
                </datalist>
                <br>
                <button type="button" id="getChangesButton">Get changes</button>
                <br>
                <label id="errorLabel" style="color: red"></label>
            </form>
        </div>
    </div>
    `

            document.body.insertAdjacentHTML("beforeend", modalHTML);
            document.querySelector("span.close").onclick = () => {
                document.querySelector("#changesModal").style.display = "none";
            };
            document.querySelector("#getChangesButton").addEventListener("click", getChanges);
        }
    }
    waitForDepots();
}

function createGetChangesButton() {
    const observer = new MutationObserver(async (mutations, observer) => {
        const buildsTitle = document.querySelector("#js-patchnotes > div:nth-child(5) > h2");
        if (!buildsTitle) {
            return;
        }
        observer.disconnect();
        const openModalButton = document.createElement("button");
        openModalButton.textContent = "Get Changes";
        openModalButton.id = "openModalButton";
        openModalButton.addEventListener("click", () => {
            document.querySelector("#redaltReset").click(); // Reset state
            if (!document.querySelector("#changesModal")) {
                createModal();
            }

            const modal = document.getElementById("changesModal");
            if (modal) {
                modal.style.display = "block";
            }
        });
        buildsTitle.insertAdjacentElement("afterend", openModalButton);
    });
    observer.observe(document, {childList: true, subtree: true});
}

createGetChangesButton();

//endregion

//region Helper Functions

/**
 * @typedef {Object} Depot
 * @property {string} id - The depot's unique identifier.
 * @property {string} description - Description of the depot.
 */

/**
 * @typedef {Object} Changelog
 * @property {string} depot_id - The depot ID this changelog refers to.
 * @property {string} manifest - Manifest ID.
 * @property {string[]} added - List of added files.
 * @property {string[]} removed - List of removed files.
 * @property {string[]} modified - List of modified files.
 */

/**
 * @typedef {Object} ChangesObject
 * @property {string} app_name - Name of app.
 * @property {string} app_id - ID of app.
 * @property {string} initial_build - ID of initial build.
 * @property {string} final_build - ID of final build.
 * @property {string[]} build_ids - Array of available Build IDs. Currently unused.
 * @property {Depot[]} depots - Array of Depots.
 * @property {Changelog[]} changelogs - Array of changelog objects.
 */

/**
 * Gets the `changesObject` from local storage
 * @returns {ChangesObject}
 */
function getChangesObject() {
    return JSON.parse(localStorage.getItem("changesObject")) || {
        app_name: appName,
        app_id: appID,
        initial_build: null,
        final_build: null,
        build_ids: [],
        depots: [],
        changelogs: []
    }
}

/**
 *
 * @param changesObject {ChangesObject}
 */
function setChangesObject(changesObject) {
    localStorage.setItem("changesObject", JSON.stringify(changesObject));
}

/**
 * Sets gettingDepots to true, opens the Depots tab and sets gettingDepots to false on completion.
 * When the Depots tab opens, runs {@link getDepotsJob} in the new tab.
 */
function getDepots() {
    const url = baseURL + "/depots/";
    GM_setValue("gettingDepots", true);
    const tab = GM_openInTab(url, {active: false});
    tab.onclose = () => {
        GM_setValue("gettingDepots", false);
    }
}

/**
 * Reads `changesObject` for builds, opens each build's patch notes and runs {@link getChangelogJob}
 */
function getChangelogs() {
    console.log("Getting changelogs...");
    const changesObject = getChangesObject();
    let buildIDs = getBuildIDs();
    const builds = buildIDs.reverse().slice(
        buildIDs.indexOf(changesObject.initial_build) + 1,
        buildIDs.indexOf(changesObject.final_build) + 1
    );
    changesObject.build_ids = builds;
    setChangesObject(changesObject);
    builds.forEach((buildID) => {
        const waitForTurn = () => {
            const buildIndex = GM_getValue("buildIndex", 0);
            console.log(builds[buildIndex], buildID)
            if (builds[buildIndex] !== buildID) {
                setTimeout(waitForTurn, 100);
            } else {
                getChangelog(buildID);
            }
        };
        waitForTurn();

    });
}


/**
 * Sets gettingChangelogs to true, opens the patchnotes for the BuildID and sets gettingChangelogs to false on
 * completion.
 * When the Depots tab opens, runs {@link getChangelogJob} in the new tab.
 * @param {string} buildID
 */
function getChangelog(buildID) {
    const url = window.location.origin + "/patchnotes/" + buildID;
    GM_setValue("gettingChangelogs", true);
    const tab = GM_openInTab(url, {active: false});
    tab.onclose = () => {
        const currentIndex = GM_getValue("buildIndex", 0);
        GM_setValue("buildIndex", currentIndex + 1);
        if (getChangesObject().build_ids.length === currentIndex + 1) {
            GM_setValue("gettingChangelogs", false);
            downloadChangesJob();
            console.log("Ready to download!");
            document.querySelector("#errorLabel").textContent = "Changes downloaded.";
        }
    };
}

/**
 * In the Patches tab (must be current tab), get the BuildIDs from the table
 * @returns {string[]} Array of BuildIDs
 */
function getBuildIDs() {
    const tableBody = document.querySelector("#js-builds");
    const rows = tableBody.querySelectorAll("tr");
    const buildIDs = [];
    rows.forEach((row) => {
        const buildID = row.querySelector("td:last-child").textContent;
        buildIDs.push(buildID);
    });
    const changesObject = getChangesObject();
    changesObject.build_ids = buildIDs;
    setChangesObject(changesObject);
    return buildIDs;
}

/**
 * Main function for getting changes
 */
function getChanges() {
    document.querySelector("#errorLabel").textContent = "";
    GM_setValue("buildIndex", 0);
    // Reset changesObject in the case of getting different apps
    const changesObject = getChangesObject();
    changesObject.app_name = appName;
    changesObject.app_id = appID;
    // Add builds
    const initial_build = document.querySelector("#buildID1").value;
    if (!initial_build) {
        document.querySelector("#errorLabel").textContent = "Please select an initial build.";
        return;
    }
    const final_build = document.querySelector("#buildID2").value;
    if (!final_build) {
        document.querySelector("#errorLabel").textContent = "Please select a final build.";
        return;
    }
    if (initial_build > final_build) {
        changesObject.initial_build = final_build;
        changesObject.final_build = initial_build;
    } else {
        changesObject.initial_build = initial_build;
        changesObject.final_build = final_build;
    }
    // Add selected depots
    const selectedDepots = Array.from(document.querySelector("#depotIDList").selectedOptions)
        .map(option => {
            return {
                depot_id: option.value,
                manifest: "",
                added: [],
                removed: [],
                modified: []
            }
        });
    if (selectedDepots.length === 0) {
        document.querySelector("#errorLabel").textContent = "Please select at least one depot.";
        return;
    }
    console.log(selectedDepots);
    changesObject.changelogs = selectedDepots;
    setChangesObject(changesObject);
    getChangelogs();
}

//endregion

//region Jobs

// JOb Switcher
/**
 * Chooses the function to execute based on GM values:
 * gettingDepots, gettingChangelogs
 */
function jobSwitcher() {
    switch (true) {
        case GM_getValue("gettingDepots", false) && window.location.href.includes("/depots/"):
            return getDepotsJob();
        case GM_getValue("gettingChangelogs", false) && window.location.href.includes("steamdb.info/patchnotes/"):
            return getChangelogJob();
    }
}

jobSwitcher()

/**
 * In the Depots tab (must be current tab), get the Depot IDs and Configurations and update the
 * {@link ChangesObject} in local storage.
 */
function getDepotsJob() {
    const changesObject = getChangesObject();
    const depots = document.querySelectorAll(".depot");
    changesObject.depots = Array.from(depots).map(depot => {
        const depotID = depot.querySelector("td:first-child").textContent;
        const configuration = depot.querySelector("td:nth-child(2)");
        const size = depot.querySelector("td:nth-child(3)");
        const configurationText = Array.from(configuration.querySelectorAll("span"))
            .map(span => span.textContent.trim())
            .join(", ");
        const sizeText = size.textContent.trim();

        const description = [configurationText, sizeText].filter(Boolean).join(", ");
        if (!description.includes("Shared Install")) {
            return {
                id: depotID, description: description
            };
        } else {
            return null;
        }
    }).filter(depot => depot !== null);
    setChangesObject(changesObject);
    window.close();
}

function getChangelogJob() {
    const changesObject = getChangesObject();
    changesObject.changelogs.map(changelog => console.log(changelog.depot_id));

    // Set up observer as the changelogs use lazy loading
    const observer = new MutationObserver(async (mutations, observer) => {
        const changelists = changesObject.changelogs.map(changelog => {
            // Return list of changes element or null
            return document.querySelector(`a[href*="${changelog.depot_id}"]`)
                ?.closest(".panel-history")
                // Check if changes have been fully loaded (files plus manifest)
                ?.querySelector("ul > li:nth-child(2)")
                ?.closest(".app-history") ?? null
        });
        // Check if all depot changes have been loaded
        if (!changelists.includes(null)) {
            observer.disconnect();
            console.log(changelists);

            /**
             * Removes the file path from added, removed or modified arrays if it exists.
             * @param array
             * @param filePath
             */
            const removeFile = (array, filePath) => {
                const index = array.indexOf(filePath);
                if (index !== -1) {
                    array.splice(index, 1);
                }
            };
            changelists.forEach((changelist) => {
                const manifest = changelist.querySelector("li.versions")
                    .querySelector(".history-link.ins");
                const manifestID = manifest.textContent.trim();
                const depotID = manifest.href.match(/\/depot\/(\d+)/)[1];
                const changelogIndex = changesObject.changelogs.findIndex(changelog => changelog.depot_id === depotID);
                if (changelogIndex === -1) {
                    return;
                }
                const changelog = changesObject.changelogs[changelogIndex];
                const depotChanges = changelist.querySelectorAll("li");
                depotChanges.forEach((change) => {
                    if (change.className === "versions") {
                        changelog.manifest = manifestID;
                        return; // Return immediately after reaching manifest (last row)
                    }
                    const file = change.querySelector("ins") ||
                        change.querySelector("del") ||
                        change.querySelector("i");
                    const filePath = file.textContent.trim();
                    if (change.className === "diff-added") {
                        // If previously added, then skip
                        // If previously removed, then remove from the `removed` record
                        if (!changelog.added.includes(filePath)) {
                            removeFile(changelog.removed, filePath);
                            changelog.added.push(filePath);
                        }
                    } else if (change.className === "diff-removed") {
                        // If previously removed, then skip (should not occur)
                        // If previously added or modified, then remove from the `added` record
                        if (!changelog.removed.includes(filePath)) {
                            removeFile(changelog.added, filePath);
                            removeFile(changelog.modified, filePath);
                            changelog.removed.push(filePath);
                        }
                    } else if (change.className === "diff-modified") {
                        // If previously modified or added, then skip
                        // If previously removed, then remove from the `removed` record (should not occur)
                        if (!(changelog.modified.includes(filePath) || changelog.added.includes(filePath))) {
                            removeFile(changelog.removed, filePath);
                            changelog.modified.push(filePath);
                        }
                    }
                });
                changesObject.changelogs[changelogIndex] = changelog;
            });
            setChangesObject(changesObject);
            window.close();
        }
    });
    observer.observe(document, {childList: true, subtree: true});
}

function downloadChangesJob() {
    const changesObject = getChangesObject();
    const downloadObject = {
        app_name: changesObject.app_name,
        app_id: changesObject.app_id,
        initial_build: changesObject.initial_build,
        final_build: changesObject.final_build,
        changelogs: changesObject.changelogs,
    };
    const filename = downloadObject.app_id + "_changes.json";
    const element = document.createElement("a");
    element.setAttribute("href", "data:text/plain;charset=utf-8," + encodeURIComponent(JSON.stringify(downloadObject)));
    element.setAttribute("download", filename);
    element.click();
}

//endregion
