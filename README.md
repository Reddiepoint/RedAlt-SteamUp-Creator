# Prerequisites

- Windows
- Script manager
- Depot Downloader
- 7-zip / WinRAR

# Instructions

1. Download RedAlt-Steam-Update-Creator, and Depot Downloader and install
   RedAlt-SteamDB-Changelist-Grabber.user.js with a script manager such as Violentmonkey.
    - Put the Creator and Depot Downloader in the same folder.

2. Run RedAlt-Steam-Update-Creator and check for updates to download RedAlt-Steam-Update-Installer.

3. Get the changes for a depot from SteamDB.
   - Go to https://steamdb.info/app/{APP}/patchnotes/ and click the "Get changes" button.
   - Choose a depot and select the build IDs (double-click the box to get a list of builds).
   - Note: The script may have problems with Edge. Try Firefox if the script doesn't work properly.

4. Open the resulting JSON file with RedAlt-Steam-Update-Creator.

5. Download the files for the changes with RedAlt-Steam-Update-Creator.
    - The downloaded files will be in the `Downloads` folder.
    - The archives will appear in the `Completed` folder.