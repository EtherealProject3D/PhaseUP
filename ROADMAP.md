# PhaseUP Roadmap

PhaseUP is an offline, ad-free Phase 10-inspired card game built with Rust and Macroquad for Windows, with Android planned later.

> This is a tentative feature roadmap, not a release schedule. Version assignments and priorities may change as gameplay testing continues.

## In progress — v0.3: Usability
- [x] Scale the game canvas to fit resized windows (initial implementation)
- [x] Show prominent draw/play/discard turn guidance (initial implementation)
- [x] Refine responsive layouts for different aspect ratios and avoid unused space
- [x] Remember window size and position where supported
- [x] Add an F3 debug overlay with dimensions, FPS and turn state
- [x] Continue gameplay testing and fix drag/drop or table interaction issues

## Planned — v0.4: Modern digital presentation
- [x] Polish cards, panels, typography and selection states
- [x] Improve CPU player areas and visible laid cards
- [x] Add paced CPU turns and subtle animations

## Planned — v0.5: Polish and settings
Audio work follows the v0.4.0 play-test and release. Track implementation in [issue #6](https://github.com/EtherealProject3D/PhaseUP/issues/6).

- [ ] Add theme-matched sound effects for buttons, card actions, turn cues, phase completion, and round/game results
- [ ] Add a gameplay music playlist using supplied tracks, with a synthwave/cyberpunk direction for Neon Forge
- [ ] Add Sound Settings with separate music/SFX volume and mute controls, saved between sessions
- [ ] Add song selection, a Now Playing display, next/previous controls, shuffle, and repeat
- [ ] Use separate menu music; opening the menu fades out and pauses gameplay music, then fades in menu music
- [ ] Resuming gameplay fades out menu music and fades the gameplay track back in from its paused position
- [ ] Start each new game on a randomly selected track, avoiding the previous game's starting track when multiple tracks are available
- [ ] Advance through the gameplay playlist automatically and transition smoothly when changing tracks
- [ ] Confirm bundled music has permission for distribution inside the game
- [ ] Settings menu
- [ ] Accessibility and usability improvements

## Planned — v0.6: Android
- [ ] Android build and packaging
- [ ] Touch controls and mobile-responsive layout

## Planned — v0.7: Save and resume
- [ ] Save an in-progress game locally
- [ ] Resume saved games

## Goal — v1.0: First polished release
- [ ] Finish gameplay, interface, and platform testing
- [ ] Prepare release builds and documentation

## Future ideas (not scheduled)
- [ ] Optional difficulty settings and house/custom rules
- [ ] Alternate themes
- [ ] **PhaseUP: Down Under Edition** — a playful April Fools' mode with upside-down visuals, correctly mapped input, Australian-themed text and a secret toggle; normal rules stay intact

## How to track work
See [GitHub Issues](https://github.com/EtherealProject3D/PhaseUP/issues) for individual tasks and discussion. A GitHub Projects board can be created later to group issues as **Backlog**, **Planned**, **In Progress**, and **Done**.

Ideas and bug reports are welcome. Please keep standard gameplay as the default; experimental rules belong behind optional settings.
