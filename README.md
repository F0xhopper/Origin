# origin

A terminal time machine for word origins. Run `origin <word>` and it opens a
**horizontal etymology tree** — modern form on the left, ancient root on the
right — that you walk back through time with Vim-style keys.

```
 origin                                                    ?:help  q:quit

   salary   ──▶  salarie  ──▶  salarium  ──▶    sal    ──▶   *sal-
   English       OldFrench       Latin           Latin       Proto-IE
   modern        c.1300 CE    c.100 BCE        Old Latin     PIE root

┌ Stage ──────────────────────────────────────────────────────────────────┐
│ Form: salary    Language: English    Period: modern                      │
│                                                                          │
│ Meaning  Fixed periodic wage.                                            │
│                                                                          │
│ stage 1/5                                                                │
└──────────────────────────────────────────────────────────────────────────┘
```

## Run

```sh
origin salary          # trace a word's etymology
origin disaster        # English → Italian → Latin → Greek → PIE
origin muscle          # English → French → Latin → Latin → PIE ("little mouse")
cargo run -- salary    # from a checkout
```

Running with no word prints usage; an unknown word prints close matches and
exits.

## Controls

| Keys                | Action                                         |
|---------------------|------------------------------------------------|
| `l` / `→`           | step back in time (older form)                 |
| `h` / `←`           | step forward (newer form)                      |
| `gg` / `0`          | jump to the modern word                        |
| `G` / `$`           | jump to the oldest root                        |
| `Space`             | play / pause backward auto-traversal           |
| `3l`                | numeric counts repeat a motion                 |
| mouse               | click a node, scroll to navigate               |
| `?`                 | toggle help                                    |
| `q` / `Esc`         | quit                                           |

## Options

| Flag             | Description                           |
|------------------|---------------------------------------|
| `--data <PATH>`  | Load an external `words.json` dataset |
| `--no-mouse`     | Disable mouse capture                 |
| `--log <PATH>`   | Write logs to a file                  |

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

`sort_key` is a signed year (negative = BCE).

## Development

```sh
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

## License

MIT — see [LICENSE](LICENSE).
