# PhaseUP Roadmap

PhaseUP is an offline, ad-free Phase 10-inspired card game built with Rust and Macroquad for Windows, with Android planned later.

> This is a tentative feature roadmap, not a release schedule. Version assignments and priorities may change as gameplay testing continues.

## In progress — v0.3: Usability
- [x] Scale the game canvas to fit resized windows (initial implementation)
- [x] Show prominent draw/play/discard turn guidance (initial implementation)
- [ ] Refine responsive layouts for different aspect ratios and avoid unused space
- [ ] Remember window size and position where supported
- [ ] Add an F3 debug overlay with dimensions, FPS and turn state
- [ ] Continue gameplay testing and fix drag/drop or table interaction issues

## Planned — v0.4: Modern digital presentation
- [ ] Polish cards, panels, typography and selection states
- [ ] Improve CPU player areas and visible laid cards
- [ ] Add paced CPU turns and subtle animations

## Planned — v0.5: Polish and settings
- [ ] Sound effects and volume controls
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
