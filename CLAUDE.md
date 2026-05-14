# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

A small Rust CLI tool that downloads the latest [NeverSink loot filter](https://github.com/NeverSinkDev/NeverSink-Filter) from GitHub and installs it into the correct Path of Exile configuration directory. Releases are built on Windows via GitHub Actions and published as `.zip` archives.

## Commands

```bash
cargo build              # debug build
cargo build --release    # release build (size-optimized: opt-level='z', LTO, single codegen unit)
cargo run                # run (compares versions, skips download if up-to-date)
cargo run -- -f          # force re-download even if already on latest version
cargo run -- -q          # quiet mode (Windows only: skips "press enter to close" prompt)
cargo test               # run tests
cargo fmt                # format
cargo clippy -- -D warnings  # lint
```

## Architecture

All logic lives in two files:

- **`src/lib/mod.rs`** — library functions: GitHub API calls (`determine_latest_release`, `fetch_url_to_buffer`), PoE directory resolution (`determine_poe_dir`), reading/removing filter files, and the `ReleaseInfo` serde struct.
- **`src/main.rs`** — async `main` wiring together the lib functions: reads current installed version tag from filter file content (`# VERSION:` line), fetches the latest GitHub release tag, compares them, then downloads and extracts the zip if they differ.

**PoE directory resolution** (`determine_poe_dir` in lib):
- macOS: `~/Library/Preferences/Path of Exile/ItemFilters/`  (via `dirs::config_dir`)
- Windows/Linux: `~/Documents/My Games/Path of Exile/`

**Version detection**: reads the `# VERSION: <tag>` comment from an existing filter file and compares it to the GitHub release `tag_name`. The current installed "version" is actually the stored tag string, not a semver — equality check only.

**Zip extraction**: fetches the zipball, iterates entries, and extracts only `.filter` files that sit exactly one directory deep (i.e., skips nested paths). The zipball URL is rewritten to use `/zipball/refs/tags/` to avoid ambiguity with branch names of the same name.

**Build script** (`build.rs`): on Windows, embeds `icon.ico` and metadata into the `.exe` via `winres`; on Unix, no-op.

## Release process

Push a git tag — the `release.yml` workflow builds on `windows-latest`, runs tests, copies the binary, zips it, and uploads it to the GitHub release.
