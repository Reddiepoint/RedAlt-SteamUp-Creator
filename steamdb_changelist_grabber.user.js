// ==UserScript==
// @name        RedAlt SteamDB Changelist Grabber
// @namespace   Violentmonkey Scripts
// @match       *://steamdb.info/app/*
// @match       *://steamdb.info/patchnotes/*
// @run-at      document-idle
// @grant       GM_setValue
// @grant       GM_getValue
// @grant       GM_deleteValue
// @grant       GM_openInTab
// @grant       window.close
// @version     0.1
// @author      Reddiepoint
// @description
// ==/UserScript==


if (GM_getValue("gettingChangelogs", 0) > 0 && window.location.href.includes("steamdb.info/patchnotes/")) {
    (function () {
        const depotID = GM_getValue("depotID", null);
        console.log(depotID);
        const depots = document.querySelector(`a[href*="/depot/${depotID}/"]`);
        if (!depots) {
            let queue = GM_getValue("gettingChangelogs", 0);
            queue--;
            GM_setValue("gettingChangelogs", queue);
            window.close();
        }

        const observer = new MutationObserver((mutations, observer) => {
            const parentSibling = depots.parentElement.nextElementSibling;
            const li = parentSibling.querySelector('li.versions');
            console.log("looking");
            if (parentSibling && li) {
                const versions = parentSibling.children;
                // Retrieve the existing changelogObject
                const existingChangelogObject = JSON.parse(GM_getValue("changelogObject", "{}"));
                if (!existingChangelogObject.added) {
                    existingChangelogObject.added = [];
                }
                if (!existingChangelogObject.removed) {
                    existingChangelogObject.removed = [];
                }
                if (!existingChangelogObject.modified) {
                    existingChangelogObject.modified = [];
                }

                for (let i = 0; i < versions.length; i++) {
                    const version = versions[i];
                    if (version.className === "diff-added") {
                        existingChangelogObject.added.push(version.querySelector("ins").textContent);
                    } else if (version.className === "diff-removed") {
                        existingChangelogObject.removed.push(version.querySelector("del").textContent);
                    } else if (version.className === "diff-modified") {
                        existingChangelogObject.modified.push(version.querySelector("i").textContent);
                    }
                }

                GM_setValue("changelogObject", JSON.stringify(existingChangelogObject));
                console.log(JSON.stringify(existingChangelogObject));
                let queue = GM_getValue("gettingChangelogs", 0);
                queue--;
                GM_setValue("gettingChangelogs", queue);
                // window.close();
                observer.disconnect();
            }
        });

        observer.observe(document, { childList: true, subtree: true });


    })();
} else {
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

        console.log(GM_getValue("changelogObject", "{}"));
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
        GM_deleteValue("changelogObject");
        GM_setValue("gettingChangelogs", intermediaryBuilds.length);
        // Get changelog for each build
        for (let i = 0; i < intermediaryBuilds.length; i++) {
            console.log(typeof intermediaryBuilds[i]);
            const changelog = getChangelog(depotID, intermediaryBuilds[i]);
            // if (i === intermediaryBuilds.length - 1) {
            //     GM_setValue("gettingDiff", false);
            // }
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


    async function getChangelog(depotID, buildID) {
        console.log(depotID);
        console.log(buildID);
        const url = document.querySelector(`a[href*="/patchnotes/${buildID}"]`).href;
        console.log(url);
        GM_openInTab(url, {
            active: false
        });
    }
}





