# fool

A command-line tool for inspecting and modifying Fable's data files.

## Commands

- `fool big dump <input.big> [output_dir]` — extract every asset from a `.big`
  archive into `output_dir/<bank>/<asset>`.
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
