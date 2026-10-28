# PhaseUP v0.4.5 dev — Android play test

This ARM64 sideload build uses the Modern & Clean theme, solid-color card faces,
monogram backs, multiple mobile selections with an immediate fixed raise, and the
saved designer layouts copied byte for byte from `C:\PhaseUP` for this build.

* App label: **PhaseUP 0.4.5 dev**
* Android package: `com.etherealproject3d.phaseup.dev` (separate from the older release)
* Version name/code: `0.4.5-dev` / `40501`
* Android 8.0/API 26 or later, ARM64, landscape
* Mobile mode and Modern & Clean are the defaults in this development build
* The closest 20:9 or 19.5:9 layout is selected from the initial viewport; Settings
  also allows choosing either of the two mobile presets
* Settings live in the app's internal files directory, without storage permissions
* Layouts and menus are embedded; later designer changes require another APK build

Source layout hashes at build preparation:

```text
prototype_mobile_compact_20_9.json   c339be207fa52fe02dceab2a5c5e1a293ee7f4b052deb2e7c2c4672584759f38
prototype_mobile_compact_19_5_9.json 54c6107e2650cd6d918c89a65fda887c36dbccd3ce0d96a108fd0f9804a1a3aa
menus_mobile_20_9.json               317dbc48f64d2620e3d3b1cab0ca0637c462ff79ad89875af7b7c5599d7b4c6d
menus_mobile_19_5_9.json             dbc752b3101b222af7998f432ab19968596c4110f22001a9f3fff7c5316ebb33
```

The Android workflow runs on an `android-dev-*` build tag, or through its manual
development input. It pins Rust 1.95.0, the APK builder revision, SDK 35 and NDK
28.2.13676358, compiles with `developer-tools`, verifies signing and APK structure,
and compares all four saved layout files with the packaged asset bytes. The output
includes `PhaseUP-v0.4.5-dev-Android-arm64.apk`, its SHA-256 checksum and package
information. This test build does not publish a production GitHub release.

Install the APK on the phone and test drawing, multi-select, staging, laying,
hitting, discarding, CPU turns, both aspect presets, menus and pause/resume. Build
and package checks do not replace on-device play testing.
