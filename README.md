# 📚 Reverse Etymology Timeline TUI

A terminal-based **linguistic time machine**. Start at a modern English word and
peel it backward through history — older spellings, earlier meanings, and the
languages it passed through — until you reach its ancient proto-language root.

It feels like a read-only Vim editor for *time*: words are navigable objects,
each word's history is a structured timeline, and movement through the centuries
is as fluid as cursor movement.

```
┌──────────────────────────────────────────────────────────────────────────┐
│ INSPECT   /sal                       h:older l:newer gg:root G:modern ?help │
├────────────┬────────────────────────────────────────────┬─────────────────┤
│ Words      │ Reverse Timeline                            │ Stage           │
│ ▶ Salary   │ ▶ ● salary       English · modern           │ Form: salarium  │
│   School   │   │                                         │ Language: Latin │
│   Sofa     │   ◆ salarie      Old French · c. 1300 CE     │ Period: c.100BCE│
│   Zero     │   │                                         │                 │
│            │   ◆ salarium     Latin · c. 100 BCE          │ Meaning         │
│            │   │                                         │ Soldier's salt  │
│            │   ◆ sal          Latin · Old Latin           │ allowance.      │
│            │   │                                         ├─────────────────┤
│            │   ◇ *sal-        Proto-Indo-European · root  │ Discovery       │
│            │                                             │ words   3/42    │
│            │                                             │ deepest PIE root│
└────────────┴────────────────────────────────────────────┴─────────────────┘
```

## Features

- **Reverse Dive** — open a word and walk backward through its etymology.
- **Deep Origin** — jump straight to the oldest known root (`gg`), then forward.
- **Discovery** — browse and compare the etymological depth of 40+ words.
- **Vim-native modal controls** with chords (`gg`), numeric counts (`3j`), and
  always-visible mode.
- **Auto-play** a backward traversal with `Space`.
- **Mouse support** — click a word, click a timeline stage, scroll to navigate.
- **Live session stats** — words explored, layers visited, languages
  encountered, deepest descent reached, and session duration.
- Calm dark archival theme: **gold** modern forms fading to muted ancient tones.
- Loads an optional `words.json`, with a full dataset **embedded** as a fallback.

## Install & run

```sh
cargo run --release
```

### Options

| Flag             | Description                                   |
|------------------|-----------------------------------------------|
| `--data <PATH>`  | Load an external `words.json` dataset.         |
| `--word <ID>`    | Open directly into a word's timeline (by id).  |
| `--random`       | Start on a random word.                        |
| `--no-mouse`     | Disable mouse capture.                          |
| `--log <PATH>`   | Write logs to a file (never to the screen).     |

## Controls

| Context        | Keys                                                            |
|----------------|----------------------------------------------------------------|
| **Navigation** | `j`/`k` move · `Enter`/`l` dive · `gg`/`G` first/last           |
| **Inspect**    | `h` older · `l` newer · `gg` root · `G` modern · `Space` play · `Esc` back |
| **Search**     | `/` start · type to filter · `Enter` keep · `Esc` cancel · `n`/`N` cycle |
| **Global**     | `r` random · `s` stats focus · `?` help · `q` quit · `3j` counts |
| **Mouse**      | click a word / stage · scroll to navigate                       |

## Data format

`words.json` is an array of words, each a chain ordered **modern → oldest**:

```json
[
  {
    "id": "salary",
    "headword": "Salary",
    "modern_meaning": "Fixed regular payment for work.",
    "chain": [
      { "form": "salary",  "language": "English",    "period": { "label": "modern",      "sort_key": 2000 }, "meaning": "Fixed periodic wage." },
      { "form": "salarium","language": "Latin",      "period": { "label": "c. 100 BCE",  "sort_key": -100 }, "meaning": "Money for salt.", "note": "Roman soldiers were paid to buy salt." },
      { "form": "*sal-",   "language": "Proto-Indo-European", "period": { "label": "PIE root", "sort_key": -4500 }, "meaning": "Salt." }
    ]
  }
]
```

`language` is any label string (known ones get dedicated styling; others are
kept verbatim). `sort_key` is a signed year (negative = BCE) used to compare
etymological depth across words.

## Architecture

Logic is decoupled from rendering so everything except drawing is unit-testable
without a terminal:

```
src/
  model/     domain types (Word, EtymologyLayer, Language, Period) + Dataset
  input/     Action intents + the pure Vim keymap (chords, counts, modes)
  app.rs     App state machine: update(Action) drives all state
  stats.rs   session-wide discovery stats
  ui/        ratatui rendering (theme, panels, overlays) — the only impure part
  tui.rs     RAII terminal guard + panic hook (a crash never breaks your shell)
  main.rs    thin binary: parse args, run the event loop, restore
```

Keys and mouse events are translated into an `Action` enum; `App::update` is the
single place state changes. This makes the whole interaction model testable by
feeding actions and asserting state.

## Development

```sh
cargo test                                  # unit + integration + render tests
cargo clippy --all-targets -- -D warnings   # lints
cargo fmt --check                           # formatting
```

## License

MIT — see [LICENSE](LICENSE).
