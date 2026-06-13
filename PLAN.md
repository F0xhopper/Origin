# 📚 Reverse Etymology Timeline TUI — Production Build Plan

A terminal-based "linguistic time machine" in Rust. Users start at a modern word
and peel it backward through historical layers to its proto-language roots.

---

## 1. Goals & Non-Goals

### Goals
- A calm, immersive, **Vim-native** TUI for exploring reverse etymology chains.
- Three exploration modes: **Reverse Dive**, **Deep Origin**, **Discovery**.
- Curated dataset of **40+ words**, each with a full backward chain, loadable
  from `words.json` with a hardcoded fallback baked into the binary.
- Session-wide **global stats** that reinforce a sense of discovery.
- Mouse support (click word / click stage / scroll).
- Production hygiene: error handling, tests, CI, docs, clean architecture.

### Non-Goals (v1)
- No live network lookups / online dictionary APIs (dataset is local & curated).
- No editing/authoring of etymology data inside the app (read-only "editor for time").
- No persistence of stats across sessions (session-scoped only) — see Stretch.
- No audio / phonetic playback.

---

## 2. Tech Stack (locked)

| Concern        | Crate                  | Version  | Notes |
|----------------|------------------------|----------|-------|
| TUI rendering  | `ratatui`              | ≥ 0.29   | layout, widgets, styling |
| Terminal I/O   | `crossterm`            | 0.28+    | raw mode, key + mouse events |
| Serialization  | `serde` + `serde_json` | 1.x      | dataset load/parse |
| Errors (app)   | `anyhow`               | 1.x      | top-level error context |
| Errors (lib)   | `thiserror`            | 2.x      | typed domain errors |
| Random         | `rand`                 | 0.8      | random word feature |
| CLI args       | `clap`                 | 4.x (derive) | flags below |
| Alignment      | `unicode-width`        | 0.2      | correct column widths for non-ASCII forms |
| Logging        | `tracing` + `tracing-subscriber` | latest | file-only logging (never to the TUI screen) |

**Runtime decision:** use the **standard library event loop** (`std::time` +
`crossterm::event::poll`) — *not* tokio. Async buys us nothing here and adds a
heavy dependency; a poll-based loop with a fixed tick gives deterministic
animation timing for the auto-play feature. `tokio` is explicitly dropped.

**Toolchain:** latest stable Rust, edition 2021. Pin via `rust-toolchain.toml`.

---

## 3. Crate / Module Architecture

Single binary crate, organized into clear modules. Strict separation between
**domain model**, **application state**, **input mapping**, and **rendering** so
each is independently testable (rendering is the only part that's awkward to
unit-test; everything else is pure).

```
etymology-tui/
├── Cargo.toml
├── rust-toolchain.toml
├── README.md
├── LICENSE
├── assets/
│   └── words.json              # canonical dataset (also embedded via include_str!)
├── src/
│   ├── main.rs                 # entry: parse args, init logging, run, restore terminal
│   ├── app.rs                  # App struct: owns all state; update() logic; mode machine
│   ├── cli.rs                  # clap args
│   ├── error.rs                # thiserror domain errors
│   ├── model/
│   │   ├── mod.rs
│   │   ├── word.rs             # Word, EtymologyLayer, Language, Period
│   │   └── dataset.rs          # Dataset: load (json | embedded fallback), index, search
│   ├── stats.rs                # GlobalStats: counters + session timer
│   ├── input/
│   │   ├── mod.rs
│   │   ├── action.rs           # Action enum (intent, decoupled from keys)
│   │   └── keymap.rs           # Vim modal key/mouse → Action translation (incl. gg/G chords, counts)
│   ├── ui/
│   │   ├── mod.rs              # top-level draw(frame, app): builds layout
│   │   ├── theme.rs            # palette, Styles (gold modern → muted ancient)
│   │   ├── word_list.rs        # left nav panel
│   │   ├── timeline.rs         # center reverse-timeline widget (the centerpiece)
│   │   ├── detail.rs           # stage detail panel
│   │   ├── stats_panel.rs      # global stats panel
│   │   ├── statusline.rs       # mode indicator + search line + key hints
│   │   ├── help.rs             # ? overlay
│   │   └── search.rs           # search/filter overlay + result cycling
│   └── tui.rs                  # terminal setup/teardown, RAII guard, event source
└── tests/
    ├── dataset.rs              # load + schema + fallback parity tests
    ├── navigation.rs           # state-machine traversal tests
    ├── search.rs               # filter/cycle tests
    └── stats.rs                # counter accounting tests
```

### Design principles
- **Action-based input:** keys/mouse → `Action` enum → `App::update(Action)`.
  This decouples bindings from logic, makes the keymap a pure function, and lets
  navigation tests drive the app without a terminal.
- **Pure update, side-effecting shell:** `App::update` mutates state and returns
  a `Flow` (Continue / Quit). The terminal loop is the only impure part.
- **RAII terminal guard** (`tui.rs`): enables raw mode + alt screen + mouse
  capture on construction, restores on `Drop` — so a panic never leaves the
  user's terminal corrupted. Also install a panic hook that restores first.

---

## 4. Domain Model

```rust
/// One historical layer in a word's descent (index 0 = modern anchor).
struct EtymologyLayer {
    form: String,            // spelling at this stage, e.g. "salarium"
    language: Language,      // Latin, Old French, Proto-Indo-European, ...
    period: Period,         // approximate era
    meaning: String,         // gloss at this stage
    note: Option<String>,    // optional cultural/phonetic transition note
}

struct Word {
    id: String,                  // slug, stable key for search/sort
    headword: String,            // modern display form, e.g. "Salary"
    modern_meaning: String,      // anchor gloss shown at the top of the dive
    chain: Vec<EtymologyLayer>,  // ordered modern → oldest (chain[0] is modern)
}

enum Language { /* enum with Display + a Style hint; Other(String) escape hatch */ }

struct Period {                 // approximate historical point, comparable for "deepest reached"
    label: String,               // human label, e.g. "c. 1200 CE", "PIE root"
    sort_key: i64,               // signed year (negative = BCE) for depth comparison
}
```

- `chain` is **always ordered modern→ancient**, invariant validated at load.
- "Move backward in time" = increase the cursor index; "forward" = decrease.
- `Period.sort_key` enables the "deepest etymological descent reached" stat and
  cross-word depth comparison in Discovery mode.
- Validation at load: non-empty chain, layer[0] language is the modern language,
  monotonic-ish periods (warn, don't fail, on out-of-order).

---

## 5. Application State & Mode Machine

```rust
enum Mode { Navigation, Inspect }      // always rendered in the status line
enum Overlay { None, Search, Help }

struct App {
    dataset: Dataset,
    filtered: Vec<usize>,      // indices into dataset, post-search filter
    list_cursor: usize,        // selection within `filtered`
    open_word: Option<usize>,  // Some -> Inspect mode active
    stage_cursor: usize,       // index into open word's chain (0 = modern)
    mode: Mode,
    overlay: Overlay,
    search: SearchState,       // query, matches, current match idx
    autoplay: Option<AutoPlay>,// Space toggles; advances stage on tick
    pending_count: Option<u32>,// vim numeric prefix (e.g. 3j)
    pending_g: bool,           // first 'g' of the `gg` chord
    focus_stats: bool,         // 's' toggles stats emphasis
    stats: GlobalStats,
    flow: Flow,
}
```

### Mode transitions
- **Navigation** (default): browse word list. `Enter` → opens word → **Inspect**.
- **Inspect**: traverse the open word's timeline. `Esc` → back to **Navigation**.
- Overlays (Search/Help) float above either mode and capture input until closed.

### Auto-play (Space)
- Toggles a backward auto-traversal: every `ANIM_TICK` (~700ms) advance
  `stage_cursor` one layer deeper until the oldest root, then stop.
- Implemented with `std::time::Instant` checked against the loop's poll timeout.

---

## 6. Input System — Vim Modal (full mapping)

Keymap is a **pure function** `(Mode, Overlay, KeyEvent, pending) -> Option<Action>`.

### Global
| Key | Action |
|-----|--------|
| `q` | Quit (Esc when no overlay/word also backs out first) |
| `?` | Toggle help overlay |
| `s` | Toggle stats focus |
| `/` | Enter search overlay |
| `r` | Random word (opens it in Inspect) |
| digits `0-9` | accumulate numeric count prefix |

### Navigation mode
| Key | Action |
|-----|--------|
| `j` / `k` | move list cursor down / up (honors count) |
| `gg` / `G` | jump to first / last word |
| `Enter` | open selected word → Inspect |

### Inspect mode (timeline)
| Key | Action |
|-----|--------|
| `h` | go **deeper** into history (older form, cursor+1) |
| `l` | return **toward modern** (cursor-1) |
| `j` / `k` | (also map to h/l for ergonomics) deeper / shallower |
| `gg` | jump to **oldest origin** (Deep Origin) |
| `G` | jump to **modern** word |
| `Space` | play/pause backward auto-traversal |
| `Esc` | close word → Navigation |

### Search overlay
| Key | Action |
|-----|--------|
| type | edit query (live filter of word list) |
| `Enter` | confirm, keep filter, return to Navigation |
| `Esc` | cancel filter |
| `n` / `N` | cycle to next / previous match (works after confirm too) |

### Mouse (crossterm `MouseEventKind`)
- Click in word-list region → select that row (hit-test against rendered rects).
- Click on a timeline stage → set `stage_cursor` to that stage.
- Scroll up/down → move the focused list / timeline.
- App stores last-rendered `Rect`s for each panel each frame to hit-test clicks.

The status line **always** shows: current mode, pending count/chord, active
search query, and a short context-relevant key hint.

---

## 7. UI / Visual Design

### Layout (ratatui constraint-based)
```
┌──────────────────────────────────────────────────────────────┐
│ status line: [INSPECT]  /sal  3j        h:deeper l:newer ?help │
├────────────┬─────────────────────────────────┬───────────────┤
│ WORD LIST  │     REVERSE TIMELINE            │  STAGE DETAIL  │
│ (nav)      │  (centerpiece, vertical descent)│  language      │
│  Salary  ◀ │   ● Salary    English  modern   │  period        │
│  Quarantine│   │                             │  meaning       │
│  Calculate │   ◆ salarie    Old French ~1300 │  note          │
│  ...       │   │                             ├───────────────┤
│            │   ◆ salarium   Latin     ~50BCE │  GLOBAL STATS  │
│            │   │                             │  words: 7      │
│            │   ◇ sal (root) PIE       deep   │  layers: 23    │
│            │                                 │  langs: 5      │
│            │                                 │  deepest: PIE  │
│            │                                 │  time: 04:12   │
└────────────┴─────────────────────────────────┴───────────────┘
```
- Outer vertical split: status line (len 1) + body + optional help overlay.
- Body horizontal: WordList (~24 cols) | Timeline (min, flex) | right column.
- Right column vertical: Stage Detail (top) + Global Stats (bottom). `s`
  emphasizes/expands the stats panel.
- Overlays (Help, Search) rendered as centered floating `Clear` + bordered block.

### Theme (`theme.rs`)
- Background: near-black; borders muted slate.
- **Gold** (`Color::Rgb`) for the modern form / active anchor.
- Active stage: bright highlight (gold underline / reversed).
- **Ancient gradient:** layers fade from warm gold → muted brown/grey the deeper
  they go (computed from stage depth ratio) — visually reinforces "descent."
- Medieval/archival tone via box-drawing connectors (`●─◆─◇`) and restrained
  color. Clarity over decoration. All styles centralized so theming is one file.
- `unicode-width` used to truncate/pad forms so non-Latin glyphs align.

---

## 8. Global Stats System (`stats.rs`)

Tracked for the session:
- `words_explored: HashSet<word_id>` → count of distinct words opened.
- `layers_visited: HashSet<(word_id, stage_idx)>` → distinct layers actually viewed.
- `languages_encountered: HashSet<Language>`.
- `deepest_descent: Option<Period>` → min `sort_key` reached across all words.
- `session_start: Instant` → duration rendered as `MM:SS`.

Hooks: `App::update` records into stats whenever a word is opened or a stage
becomes the cursor. Pure counters → easy to unit-test. Rendered live in the
stats panel; `s` toggles a focused/expanded view.

---

## 9. Dataset (`assets/words.json` + embedded fallback)

- Canonical JSON at `assets/words.json`, **also embedded** via
  `include_str!` so the binary always has a working fallback (≥ 40 words).
- Load order: `--data <path>` flag → `./words.json` in CWD → embedded.
- Schema mirrors the domain model; `serde` derives with `#[serde(deny_unknown_fields)]`
  to catch typos. A test deserializes the embedded data and asserts invariants.
- Initial curated word set (sample, ≥40 total) — chosen for vivid, well-attested
  chains spanning English ← French ← Latin ← PIE, plus Greek, Old Norse, Arabic,
  Sanskrit branches for language variety:
  `salary, quarantine, calculate, muscle, candidate, companion, disaster,
  alphabet, algebra, assassin, sarcasm, clue, nice, silly, decimate, sinister,
  vaccine, robot, denim, ketchup, tsunami, avatar, juggernaut, sabotage,
  galaxy, planet, museum, school, dinosaur, philosophy, democracy, hippopotamus,
  whisky, cappuccino, magazine, sofa, candy, lemon, orange, zero ...`

---

## 10. CLI (`clap`)

```
etymology-tui [OPTIONS]
  --data <PATH>     load an external words.json
  --word <ID>       open directly into a word's timeline on startup
  --random          start on a random word
  --no-mouse        disable mouse capture
  --log <PATH>      enable tracing to a file
  -h --help / -V --version
```

---

## 11. Error Handling & Robustness

- `error.rs`: `thiserror` enum — `DataLoad`, `Parse`, `EmptyChain`,
  `InvalidLayer`, `Io`. `main` uses `anyhow` for context + clean top-level report.
- **Terminal safety:** RAII guard restores cooked mode/main screen on drop;
  a panic hook restores the terminal *then* prints the panic so a crash never
  bricks the user's shell.
- Graceful resize handling (`Event::Resize`) — re-layout next draw.
- Logging goes to a file only (never the TUI). No `println!` in the loop.

---

## 12. Testing Strategy

- **Unit (pure logic, no terminal):**
  - keymap: chords (`gg`), counts (`3j`), mode-sensitive bindings.
  - navigation: open/close, deeper/shallower clamping at ends, gg/G jumps.
  - search: filtering, `n`/`N` wraparound cycling.
  - stats: distinct counting, deepest-descent tracking, duration formatting.
  - dataset: embedded fallback parses; invariants hold; `--data` round-trip.
- **Integration (`tests/`):** drive `App` via `Action` sequences and assert
  resulting state — full traversals without a TTY.
- Rendering is smoke-tested with ratatui's `TestBackend` (assert it draws a
  frame at several sizes without panicking; optional buffer snapshot for the
  timeline widget).
- `cargo clippy -- -D warnings` and `cargo fmt --check` enforced.

---

## 13. CI / Tooling

- `.github/workflows/ci.yml`: `fmt --check`, `clippy -D warnings`, `test`,
  `build --release` on stable. (Matches the on-the-web SessionStart hook below.)
- `rust-toolchain.toml` pins stable + components (rustfmt, clippy).
- `SessionStart` hook (`.claude/`) so web sessions auto-`cargo build`/`test`.

---

## 14. Build Phases (incremental, each independently runnable)

1. **Scaffold** — `cargo init`, Cargo.toml deps, module skeleton, RAII terminal
   guard + panic hook, empty event loop that quits on `q`. *Runs, restores cleanly.*
2. **Model + Dataset** — domain types, serde, embedded fallback (~10 words to
   start), load order, invariants + dataset tests.
3. **Navigation mode** — word-list panel, `j/k/gg/G`, action enum + keymap,
   status line with mode indicator. Navigation tests.
4. **Inspect mode + Timeline widget** — open word, reverse-timeline render,
   `h/l/gg/G` traversal with clamping, stage detail panel. Navigation tests.
5. **Stats system** — counters, session timer, stats panel, `s` focus.
6. **Search & Discovery** — `/` overlay, live filter, `n/N` cycling, `r` random.
7. **Auto-play** — `Space` backward traversal on tick; deep-origin jump polish.
8. **Mouse support** — per-panel rect hit-testing for click/scroll.
9. **Theme & polish** — gold→ancient gradient, connectors, help overlay (`?`),
   unicode-width alignment, resize handling.
10. **Dataset to 40+ words** — curate full set, validate, snapshot timeline.
11. **Hardening** — clippy/fmt clean, full test pass, README + GIF/screens, CI.

Each phase ends green (`cargo test` + `clippy`) and is committed separately on
`claude/wonderful-planck-hglq89`.

---

## 15. Acceptance Criteria

- [ ] Launches, enters alt screen, restores terminal cleanly on `q` and on panic.
- [ ] ≥ 40 words, each with a multi-layer modern→ancient chain; loads from
      `words.json` or embedded fallback.
- [ ] Vim modal nav fully working: `j k h l gg G Enter Esc Space / n N r s ? q`,
      numeric counts, mode always visible.
- [ ] Reverse Dive, Deep Origin (`gg`), and Discovery (browse/compare) all usable.
- [ ] Mouse: click word, click stage, scroll.
- [ ] Live global stats (words, layers, languages, deepest descent, duration).
- [ ] Gold-accented modern → muted ancient theming; calm, readable, aligned.
- [ ] `cargo test` green, `clippy -D warnings` clean, CI passing, README complete.

---

## 16. Stretch (post-v1)
- Persist stats/history to a config dir (achievements: "reached PIE on N words").
- Side-by-side compare of two words' depths in Discovery mode.
- Forward "replay" animation from root → modern.
- Theme switcher; export a word's chain to markdown.
