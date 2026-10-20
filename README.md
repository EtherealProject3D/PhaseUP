# PhaseUP

An ad-free, offline, Rust card game for Windows and (planned) Android.

## Build on Windows

Install Rust from https://rustup.rs/ and run:

```powershell
cargo run --release
```

The game currently uses Macroquad. This repository is an early prototype and the rules/AI are still being developed.

## Project goals

- One human player versus three computer opponents
- Ten-stage set/run/color card gameplay
- Offline and ad-free
- Shared Rust gameplay for Windows and Android

PhaseUP is an independent prototype; it is not affiliated with Mattel.

## The opponents: more than just computer players

**Our goal: challenging opponents, not cheating opponents.**

PhaseUP is being designed around a simple idea: playing against the computer should feel like sitting at a table with people who have learned the game over hundreds of rounds. The AI design draws on years of real-world Phase 10 play, where reading opponents, managing risk, and denying useful discards can matter just as much as building your own phase.

Rather than secretly knowing opponents' hands or the next card in the deck, advanced CPU opponents are intended to reason from **information a human player could actually observe**.

### Recognizable players with their own styles

The game now has the foundation for a fixed roster of named CPU identities, each with an assigned skill level and play style. The long-term goal is that familiar names mean familiar opponents: a defensive player tends to protect the next player's discard options, while an aggressive player focuses more on finishing their own phase. Their strategies can adapt without losing their personalities.

| Difficulty | Planned behavior |
| --- | --- |
| **Novice** | Learning-level decisions, occasional missed opportunities |
| **Easy** | Basic phase building and straightforward discards |
| **Normal** | Competent general play, the baseline for current CPU behavior |
| **Hard** | More deliberate phase planning, defensive discards, and opponent awareness |
| **Expert** | Advanced card tracking, inference, flexible hand building, and situational defense |

The planned matchmaking system keeps opponents within a narrow difficulty range, avoiding combinations such as Expert and Novice at the same table. A difficulty selector and mixed-difficulty options are planned; they are **not yet available in the game**.

### Strategies we're designing

- **Defensive discarding:** Avoid handing the next player a card that could complete a phase or immediately hit a laid-down group.
- **Phase-aware decisions:** A player building a long run may value middle numbers more than 1s or 12s; a player building a color phase may strongly prefer one color.
- **Visible card tracking:** Compare potential sets and runs against cards already laid down or seen in the discard pile. Account for the uncertainty of hidden hands and recycled discards.
- **Opponent profiling:** Remember which cards players deliberately take from the discard pile, and infer what numbers or colors they may be collecting.
- **Flexible hand building:** Evaluate competing sets, colors, and possible run ranges instead of committing too early.
- **Situational Skip cards:** Use Skips to disrupt threatening opponents, or play them early when they don't help complete a demanding phase.
- **Risk versus reward:** Sometimes hold a card to deny an opponent, but balance that against completing your own phase and the points you might be left holding.
- **Individual personalities:** Different opponents weigh aggression, defense, adaptability, and risk differently, even at similar skill levels.

These are **design goals, not claims about the current AI**. Today the CPU already plays legal turns, lays phases, hits available groups, and uses basic discard heuristics. The named roster and personality metadata are being introduced first; the deeper strategic decision engine will come later.

### Development priority

We're deliberately polishing **core gameplay, rules, UI, animations, usability, and stability first**. Once the base game is solid, we'll develop and test the advanced difficulty and personality systems incrementally. The aim is to make the computer *smarter*, not give it hidden information or unfair advantages.

## Roadmap and feature requests

See the [PhaseUP roadmap](ROADMAP.md) for planned releases and future ideas, and [GitHub Issues](https://github.com/EtherealProject3D/PhaseUP/issues) to track or discuss individual features and bugs.

## Version history

Explore [historical PhaseUP versions and source snapshots](CHANGELOG.md), including early playable builds and release notes.
