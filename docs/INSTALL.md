# Install FHCM Community Preview on Windows

> **Community preview · provided as is.** FHCM Launcher and the standalone FHCM Map Tool are in active development. Expect bugs and changes between releases. Back up your personal save and start in Sandbox.

1. Download the Windows x64 installer and verify the SHA-256 from the release. The installer is unsigned for this preview.
2. Run the installer, then open FHCM. Setup needs WebView2 and a compatible owned FH2/Azimuth installation with an existing save. Select the folders that contain `Mount\forza_durango_release_final.exe` and `azimuth.exe`. Select a new library folder with enough space for a separate game and runtime copy. Installed Storm Island content is optional.
3. In FHCM, choose **Import ZIP** and pick the complete PCH package. Launch **Pacific Coast Highway** in **Sandbox**, with **Standard** Drivatars and your preferred traffic level. Close the game before switching maps or builds.

## Standalone FHCM Map Tool

The Windows x64 **FHCM Map Tool** is an as-is community preview for the command line. Download its ZIP, extract it, and run `.\fhcm-map.exe --help` in PowerShell. It shares setup, import, and Drive with the desktop app and does not require installing the desktop app or opening a WebView. The PCH documentation includes exact setup, import, and drive commands. [PCH installation guide](https://github.com/fhcustommaps/pch-public-preview/blob/main/docs/INSTALL.md)
