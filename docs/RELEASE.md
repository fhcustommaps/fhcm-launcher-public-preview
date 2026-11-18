# FHCM Community Preview release checklist

- Release the Windows x64 NSIS installer and the separate FHCM Map Tool ZIP with SHA-256 checksums.
- Test original FH2 start, an imported PCH Drive, all four traffic profiles, both PCH racer modes, selected vehicle, weather and HUD, exit, and personal-save restoration on a clean supported Windows setup.
- Smoke-test the clean desktop installer on Windows before public publication. Use Standard for the first session and retain the Experimental label for Tour.
- Record the exact supported runtime and keep the release notes clear about the unsigned installer.
- Review redistribution of the map payload and game-derived launcher build inputs before sharing. Keep release assets and repositories private until that gate is resolved.

## 0.2.1 verification

The Windows x64 NSIS package contains FHCM Launcher and Map Tool 0.2.1. The Rust suite passed 10 tests. The owned Gunsai fixture test was skipped. The standalone tool created a clean library from an owned FH2 and Azimuth 0.9.10 fixture, imported PCH Beta 1, and prepared all four traffic selections and both parking spawns without starting the game. A later live Medium / Standard Drive with the exact California Plates ZIP verified a traffic car, a bus, and all 26 personal-save files after exit.

| Download | SHA-256 |
| --- | --- |
| `FHCM-Community-Preview-0.2.1-x64-setup.exe` | `3f20c916cd502633b456b4a536b6fd5fcbb9732d900a5ff9bc195618228aa957` |
| `FHCM-Map-Tool-0.2.1-win-x64.zip` | `0afae1cf9430ce3ed2b0979f882da20f46b2eb88844c91a96c1f6c0acf81d793` |
