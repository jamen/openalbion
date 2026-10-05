# fool

A command-line tool for inspecting and modifying Fable's data files.

## Commands

- `fool big dump <input.big> [output_dir]` — extract every asset from a `.big`
  archive into `output_dir/<bank>/<asset>`.
- `fool fmp info <input.fmp>` — print an `.fmp`'s version, content type and bank
  table. The `.fmp` container reuses the `.big` header and bank-table layout with a
  `B\0\0\0` magic, version 101 and a content-type word; wrapped (`12345` + zlib)
  packages are inflated first.
- `fool fmp list <input.fmp> [--bank NAME] [--text]` — list each bank's records.
  A bank's records are `.big` `AssetMetadata` entries (minus the type map), so the
  symbol names, payload sizes and `extras` come straight from the existing BIG
  parser; `GameBINEntries` extras name the definition type. `--text` decodes
  text-bank payloads (UTF-16LE).
- `fool fmp dump <input.fmp> [output_dir]` — write each bank's raw bytes to
  `output_dir/<bank>.bin`. Bank contents are opaque at this level (they are not
  `.big` asset tables).
- `fool wad unpack <input.wad> [output_dir]` — extract every file from a `.wad`
  archive.
- `fool wad pack <input_dir> [output.wad] [-P prefix]` — pack a directory of
  files into a `.wad` archive. `-P/--entry-prefix` sets the in-game path prefix
  (e.g. `"Data\Levels\FinalAlbion\"`).
- `fool texture export` / `fool texture import` — placeholders, not implemented
  yet.

## History

`fool` is the latest in a line of Fable asset CLIs (`defable`, `fool_wad`,
`fool`). This incarnation was restored from the last version on `main` and
re-integrated against the current `fable-data` API. See `HISTORY_REPORT.md` for
the full lineage.
