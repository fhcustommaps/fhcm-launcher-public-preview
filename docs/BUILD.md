# Build FHCM Launcher and Map Tool

Build on Windows x64 with Node.js and npm, Rust stable with the MSVC target, Visual Studio C++ Build Tools, Windows SDK, and the Tauri NSIS toolchain.

1. Run `npm ci` from the repository root.
2. Stage inputs matching `build-inputs.json` with `.\Stage-BuildInputs.ps1 -InputDirectory '<prepared-inputs>'`. It verifies input hashes before copying.
3. Run `.\Build-Desktop.ps1` from PowerShell. The installer is in `src-tauri/target/release/bundle/nsis`. Build the independent console tool with `cargo build --release --locked --manifest-path src-tauri/Cargo.toml --bin fhcm-map`.
4. Run `cargo test --locked --manifest-path src-tauri/Cargo.toml` after generating the frontend. Keep the test library separate from your personal install. The owned-garage test additionally requires a preconfigured Gunsai fixture under `FHCM_MANAGER_DIR` and must be run explicitly with `--ignored`.

The input manifest covers the compatible platform DLLs, sandbox garage templates, vehicle metadata, display module, and conditions fixtures. Keep private build inputs outside Git. Package the map tool executable with its notices, the installation guide, and a checksum.
