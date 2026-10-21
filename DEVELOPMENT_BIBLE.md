# PhaseUP — Development Bible

_Last updated: October 20, 2026_

> Living design memory for PhaseUP. Preserve good ideas across conversations without treating every brainstorm as an approved feature. The GitHub source code and tests are authoritative for actual implementation status.

## Project identity and constraints

- **Name:** PhaseUP. Keep this name unless explicitly revisited. "Phase Out!" was rejected because of a conflict; "PhaseStack" was disliked.
- **Tech stack:** Rust and Macroquad.
- **Gameplay direction:** Offline, ad-free, single human player versus three CPU opponents.
- **Platforms:** Windows 11 first; Android is a future target, not a promise of existing support.
- **Development workflow:** Review the latest GitHub `main` before editing; commit approved changes to the repository; provide PowerShell instructions for pulling and running locally.
- **Principle:** Do not destabilize working features to pursue speculative improvements.

## Working baseline — preserve

**Confirmed by the developer on October 20, 2026:** Resolution presets and window sizing now work as desired after reverting three sizing-related changes. **Do not change resolution fitting, preset selection, or 16:9/16:10 scaling unless explicitly requested.**

- Working rollback reference: commit `00f022bd0b2df53a9b94fc0d768e8801898b3fe2` (after restoring `src/settings.rs` and `src/main.rs` to their pre-change behavior).
- Both 16:9 and 16:10 are intentional layout targets.
- Earlier discussion covered main menu spacing, bottom stripe visibility, stage/lay areas, and centered turn instructions. **Verify current code/screenshots before marking each detail complete.**
- Do not confuse a selected resolution preset with a necessarily identical physical window size; existing desktop fitting was intentionally restored.

## Gameplay and CPU strategy roadmap

The following are **design ideas and prior discussion**, not a claim that they are already implemented.

### Defensive decision-making

1. **Strategic Skip cards:** Prefer using a Skip against a threatening opponent, especially the next player, when it could prevent a phase completion or going out.
2. **Smart discards:** Avoid discarding a card that is visibly valuable to the next opponent, based on their current phase and progress.
3. **Visible-card awareness:** Consider public information such as played/discarded cards, completed phases, and opponents' remaining hand sizes. Do **not** give CPU players knowledge of hidden human cards.
4. **Self-preservation first:** Avoid sabotaging the CPU's own chance to complete its phase merely to deny an opponent a useful card. Defensive play should weigh opportunity cost.
5. **Probability-based heuristics:** Use lightweight, offline Rust decision logic rather than an external AI service.

### Proposed five difficulty levels

| Level | Intended behavior | Status |
| --- | --- | --- |
| Novice | Basic legal play; little opponent awareness | Proposed |
| Easy | Avoids obvious helpful discards | Proposed |
| Normal | Baseline/current CPU behavior as reference | Proposed; verify actual baseline |
| Hard | Tracks public phase progress, hand sizes, and potential hits | Proposed |
| Expert | Estimates card demand; weighs Skip timing and defensive denial | Proposed |

**CPU personalities** were also discussed as a future idea. Possible differentiation includes more aggressive versus more conservative play, but specific personalities and tuning are **not yet approved**.

### Architecture direction

- Keep **game rules/state**, **CPU decisions**, and **Macroquad rendering/input** logically separated.
- Preserve deterministic and testable game logic where possible.
- Never let CPU strategy bypass the legal move rules.
- When adding advanced AI, test each difficulty against the same game situations and check both legality and meaningful behavior differences.

## UI, visual identity, and accessibility ideas

### Theme concepts

Three visual directions were explored in earlier mockups:

- **Modern & Clean** — typography concept: Inter.
- **Bold & Futuristic** — typography concept: Montserrat.
- **Colorful & Playful** — typography concept: Baloo 2.

Selectable themes were described in concept art as a **future settings option**. The current repository may already include theme-related code; inspect it before claiming theme selection is complete.

Another earlier visual comparison explored **Modern Digital** versus **Realistic Tabletop** presentation. These are design explorations, not a decision to replace the current UI.

### Gameplay presentation and animation ideas

- Natural CPU turn pacing and smooth transitions.
- Animated CPU phase laying, hitting, and Wild placement.
- Clear card movement between draw/discard piles, hand, lay area, and stage area.
- Readable player panels for **You, CPU 1, CPU 2, CPU 3**.
- Clear phase/card counts and current-turn instructions.
- Consistent layout across 16:9 and 16:10.
- Potential improvements to card effects, typography, and UI polish.

**Status:** Ideas/targets; inspect current code and gameplay before classifying as complete.

## Rules, testing, and stability

Previously discussed work sequence:

1. Automated gameplay tests, including legality and phase progression.
2. Separate rules into well-defined modules as needed.
3. Polish settings and setup flows.
4. Prepare for Android without breaking Windows.
5. Continue with How to Play/Game Rules, theme selection, animations, and advanced CPU.

Earlier conversations referenced an automated test suite with **19 tests**; treat this as historical context, **not a verified current test count**.

Suggested regression scenarios for future work:

- Draw/discard turn order and legal action constraints.
- Phase completion and progression.
- Skip behavior and target selection.
- Wild-card placement and rules.
- Hitting/laying cards and CPU decision legality.
- Window presets and both supported aspect ratios.
- Persistence of settings and return from fullscreen.

## Future roadmap (not a commitment or fixed schedule)

### Near-term candidates
- Verify and document existing gameplay/rule behavior.
- Improve How to Play / Game Rules clarity.
- Polish game-state feedback and animations without altering stable sizing.
- Add regression tests for recently fixed behavior.

### Mid-term candidates
- Implement and tune five CPU difficulty levels.
- Add opponent-aware CPU strategy based on public information.
- Evaluate optional CPU personalities.
- Refine theme selection and visual consistency.

### Long-term candidates
- Android adaptation: input, UI scale, packaging, and performance.
- Additional animation polish and accessibility options.
- Further gameplay strategy balancing informed by testing.

## Idea archive and decision log

| Date | Item | Classification | Notes |
| --- | --- | --- | --- |
| 2026-10-20 | Keep **PhaseUP** name | Preference/decision | Do not rename casually |
| 2026-10-20 | Offline, ad-free, 1 human + 3 CPU | Project direction | Preserve |
| 2026-10-20 | Windows 11 first, Android later | Platform direction | Android future |
| 2026-10-20 | Current resolution sizing | Confirmed working | Leave alone |
| Prior discussions | Five CPU difficulty levels | Planned concept | Not verified implemented |
| Prior discussions | Smart discards, Skips, public-card tracking | Planned concept | Avoid hidden-information cheating |
| Prior discussions | CPU personalities | Idea to explore | Details undecided |
| Prior discussions | Three visual themes | Explored concept | Implementation needs verification |
| Prior discussions | Modern Digital vs Realistic Tabletop | Brainstorm | No final choice established |
| Prior discussions | Smooth CPU/card animations | Planned polish | Implementation needs verification |

## Rules for maintaining this document

1. **Never delete a good idea just because it is not currently scheduled.** Move it to the Idea Archive with context.
2. Mark every item **Confirmed**, **Proposed**, **Exploring**, **Implemented (verified)**, **Rejected**, or **Deferred** as appropriate.
3. **Do not mark a feature implemented** based only on a prior conversation; verify code/tests or user confirmation.
4. When an idea changes, record **why** and preserve the previous decision in the log.
5. Before coding, read this file and the current source. Working behavior and user decisions outrank speculative roadmap items.
6. Update this document after significant design decisions or newly recovered older ideas.
7. Treat the list as **recoverable knowledge, not exhaustive transcripts**. Earlier conversations may contain details not yet captured; add them when found.


## CPU simulation and balancing (2026-10-20)
- **Confirmed:** User reported 43 passing unit tests and a successful release build after opponent-awareness work.
- **Implemented, awaiting local verification:** `src/simulation.rs` adds seeded headless CPU-vs-CPU round simulations and reproducibility checks, registered under `#[cfg(test)]` in `src/main.rs`.
- The initial harness exercises Phase 1 rounds, rotating five difficulty levels through four seats, and reports round completions, turns, phases laid, Wilds laid, Skips and difficulty wins. It is **not** a full ten-phase game simulation or a reliable difficulty ranking yet.
- **Next:** Validate harness on Windows with `cargo test -- --nocapture`; then extend to complete games, collect larger samples, and tune difficulty based on measured outcomes.
- **UI freeze remains in effect:** Do not change menus, sizing or resolution until CPU work is finished or the user explicitly requests UI changes.

- **2026-10-20 full-game simulation extension (awaiting local test):** Headless simulator now carries phase progress and scores across rounds, records completed games and difficulty winners, and rotates seats between games. Two tests added (expected 47). This remains an approximation of production turn orchestration. Review 120-round test statistics before drawing conclusions about difficulty balance.

- **2026-10-20 balancing diagnostics (awaiting local verification):** Simulation now reports stalled/exhausted rounds, longest round, and seat wins; rotates starting seat by game; resets incomplete game progress instead of mixing aborted rounds into full-game results. One new test (expected 48). Do not claim difficulty balance from small samples; inspect printed stats first.

- **2026-10-20 normalized balancing (awaiting local test):** Simulation records difficulty entries per round and per completed game, prints per-entry win rates, and adds an ignored five-batch/300-round-per-batch large-sample benchmark. Normal test suite gains one participation invariant test (49 active tests plus one ignored). Run `cargo test cpu_large_balance_sample -- --ignored --nocapture` separately; do not infer relative difficulty strength from unnormalized counts.

- **2026-10-20 advanced AI balancing experiment (awaiting user tests):** Five-batch baseline (1,500 rounds) Normal 33.0%, Hard 29.4%, Expert 23.4% participation-normalized round wins. Lowered Hard/Expert Wild-conservation weights from 100/200 to 55/65, reduced non-threat Skip retention from -180/-280 to -35/-55, and bounded defense risk using shared `cpu_bounded_defense_penalty` in both CPU gameplay and simulation. Re-run same seeded release benchmark to evaluate; revert/tune if performance worsens. UI unchanged.

- **2026-10-20 Hard/Easy balancing experiment (awaiting local tests):** Following five-batch results showing Expert improved but Hard weakened and Easy below Novice, changed Hard defense cap 130→90 and Wild conservation weight 55→35; Easy discard retention now additionally values run-neighbor and color support (18 per supporting card). Expert unchanged. Compare seeded release benchmark to prior output before deciding to retain changes. UI frozen.

- **2026-10-20 phase-completion discard improvement (awaiting local verification):** Easy, Hard, and Expert now evaluate whether drawing the top discard makes their current phase immediately layable, rather than rejecting such a card based only on retention heuristics. Added a focused regression test. No UI changes. Compare seeded large-sample results against prior baseline; performance impact not yet verified.

- **2026-10-20 three-level difficulty simplification (awaiting local verification):** Main-menu difficulty cycles Easy → Normal → Hard; nine named CPU roster members now use only those three levels (3 Easy, 2 Normal, 4 Hard). Existing saved `novice` maps to Easy, `expert` maps to Hard. Legacy Novice/Expert enum branches remain internally for regression coverage, but are not selectable or assigned to opponents. CPU simulator now benchmarks four seats with Easy/Normal/Hard/Hard, rotating seats, and reports three-level counts/rates. Difficulty-related tests updated. UI layout and window settings unchanged. Run cargo test and release benchmark to verify.

- **2026-10-20 long-run phase milestone (awaiting local verification):** Normal/Hard run-phase discard scoring now rewards unique-number coverage inside a legal run window (7/8/9-card runs; 4-card component in mixed phases). Easy no longer hoards duplicate-number pairs for run-only phases. Added tests for run-window coverage and Easy retention. Simulator records per-phase stall participation, unlaid CPUs by difficulty, and discard-vs-stock draw counts during stalled rounds. No UI changes. Compare new 1,500-round benchmark against baseline (21 stalls / 1,500 rounds).

- **2026-10-20 long-run discard-draw improvement (awaiting local verification):** Easy/Normal/Hard take a visible number discard for 7/8/9-card run phases when it increases maximum unique-rank coverage within a legal run window after replacing one non-Wild card. Skip and duplicate-only draws do not qualify through this override. Added regression tests. Prior 1,500-round benchmark: 9 stalls, 85 completed games; compare next benchmark and discard-draw diagnostics. No UI changes.

- **2026-10-20 selective long-run discard refinement (awaiting local verification):** The broad visible-discard override increased stalls from 9 to 15 per 1,500 rounds and caused excessive discard pickups in stalled rounds. Replaced it with a narrow override: for 7/8/9-card runs, the visible number must fill an absent rank in a legal window with at least `length - 2` distinct natural ranks already held and enough Wilds to leave at most one gap before taking the card. The original phase-aware discard heuristic remains the fallback; run-window retention scoring is preserved. Updated two regression tests. No UI changes.

- **2026-10-20 endurance validation (awaiting local run):** Added ignored `cpu_ten_thousand_round_benchmark` to run 10 independently seeded 1,000-round batches (350-turn cap), report per-batch results and aggregate 10,000-round difficulty participation, round/game wins, stalls, phase diagnostics and draw-source counts. Run `cargo test cpu_ten_thousand_round_benchmark --release -- --ignored --nocapture`. Existing 1,500-round benchmark unchanged. No CPU behavior or UI changes.

- **2026-10-20 fixed canvas UI consolidation (awaiting local verification):** Standardized menus and gameplay on a single logical 1920×1200 (16:10) canvas regardless of selected window aspect ratio. Camera and mouse scaling retain uniform fit/letterboxing; 16:9 presets remain available and show side margins. Removed 16:9-specific panel, board, pile, log, and guidance positioning branches; the resolution preset selection and saved window size are unchanged. CPU AI and game rules untouched. Verify menu, Settings resolution picker, gameplay hitboxes, and both aspect ratios locally.
