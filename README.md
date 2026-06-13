# Reverse Etymology Timeline TUI

A terminal app for exploring word origins. Start at a modern English word and
step backward through history — older spellings, earlier meanings, and the
languages it passed through — until you reach its ancient root.

Navigation is Vim-style: words are a list you move through, and each word's
history is a timeline you walk back in time.

```
┌──────────────────────────────────────────────────────────────────────────┐
│ INSPECT   /sal                       h:older l:newer gg:root G:modern ?help │
├────────────┬────────────────────────────────────────────┬─────────────────┤
│ Words      │ Reverse Timeline                            │ Stage           │
│ > Salary   │ > salary       English · modern             │ Form: salarium  │
│   School   │   salarie      Old French · c. 1300 CE       │ Language: Latin │
│   Sofa     │   salarium     Latin · c. 100 BCE            │ Period: c.100BCE│
│   Zero     │   sal          Latin · Old Latin             │ Meaning: salt   │
│            │   *sal-        Proto-Indo-European · root    │ allowance       │
└────────────┴────────────────────────────────────────────┴─────────────────┘
```

## Run

```sh
cargo run --release
```

### Options

| Flag             | Description                                  |
|------------------|----------------------------------------------|
| `--data <PATH>`  | Load an external `words.json` dataset.       |
| `--word <ID>`    | Open directly into a word's timeline (by id).|
| `--random`       | Start on a random word.                      |
| `--no-mouse`     | Disable mouse capture.                        |
| `--log <PATH>`   | Write logs to a file.                         |

## Controls

| Context     | Keys                                                           |
|-------------|---------------------------------------------------------------|
| Navigation  | `j`/`k` move, `Enter`/`l` open, `gg`/`G` first/last            |
| Inspect     | `h` older, `l` newer, `gg` root, `G` modern, `Space` autoplay, `Esc` back |
| Search      | `/` start, type to filter, `Enter` keep, `Esc` cancel, `n`/`N` cycle |
| Global      | `r` random, `s` stats, `?` help, `q` quit, `3j` numeric counts |
| Mouse       | click a word or stage, scroll to navigate                     |

The session stats panel tracks words explored, layers visited, languages
encountered, deepest point reached, and session time.

## Data format

`words.json` is an array of words, each with a chain ordered modern to oldest.
A full dataset is embedded in the binary, so the file is optional.

```json
[
  {
    "id": "salary",
    "headword": "Salary",
    "modern_meaning": "Fixed regular payment for work.",
    "chain": [
      { "form": "salary", "language": "English", "period": { "label": "modern", "sort_key": 2000 }, "meaning": "Fixed periodic wage." },
      { "form": "salarium", "language": "Latin", "period": { "label": "c. 100 BCE", "sort_key": -100 }, "meaning": "Money for salt." },
      { "form": "*sal-", "language": "Proto-Indo-European", "period": { "label": "PIE root", "sort_key": -4500 }, "meaning": "Salt." }
    ]
  }
]
```

`sort_key` is a signed year (negative = BCE) used to compare depth across words.

## Layout

```
src/
  model/   domain types and dataset loading
  input/   Action intents and the Vim keymap
  app.rs   state machine: update(Action) drives all state
  stats.rs session stats
  ui/      ratatui rendering
  tui.rs   terminal setup, teardown, panic safety
  main.rs  argument parsing and the event loop
```

Keys and mouse events become `Action` values, and `App::update` is the single
place state changes — so the logic is testable without a terminal.

## Development

```sh
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

## License

MIT — see [LICENSE](LICENSE).
