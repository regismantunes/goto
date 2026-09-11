# goto

[![CI](https://github.com/regismantunes/goto/actions/workflows/ci.yml/badge.svg)](https://github.com/regismantunes/goto/actions/workflows/ci.yml)
[![GitHub Release](https://img.shields.io/github/v/release/regismantunes/goto)](https://github.com/regismantunes/goto/releases/latest)
[![GitHub Downloads](https://img.shields.io/github/downloads/regismantunes/goto/total)](https://github.com/regismantunes/goto/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.80%2B-orange?logo=rust)](https://www.rust-lang.org/)

`goto` is a small, cross-platform directory bookmark manager. Save a workplace once, then jump to it by name from PowerShell, CMD, Bash, Zsh, or Fish.

```text
goto -s C:\Source\MyWork --name MyWork
goto MyWork
```

## Why shell setup is required

An executable cannot change its parent shell's working directory. `goto init` prints a small, auditable shell function that resolves a bookmark with the native executable and performs the directory change in the current shell.

## Install

Download the archive for your platform from the [latest release](../../releases/latest), extract it, and put `goto` on `PATH`. Release assets also contain `install.sh` and `install.ps1`; these verify the downloaded archive using the published SHA-256 checksums and install it for the current user.

To build from source:

```text
cargo install --path .
```

When using CMD with a source installation, also copy `shell\goto-shell.cmd` and `shell\goto-init.cmd` beside the installed `goto.exe`.

### Activate your shell

Run the matching command now and add the same line to your shell profile for future sessions.

PowerShell:

```powershell
Invoke-Expression (& goto.exe init powershell | Out-String)
```

Bash (`~/.bashrc`):

```bash
eval "$(goto init bash)"
```

Zsh (`~/.zshrc`):

```zsh
eval "$(goto init zsh)"
```

Fish (`~/.config/fish/config.fish`):

```fish
goto init fish | source
```

CMD already owns the names `goto` and, depending on parsing context, `goto.exe`. Its integration therefore uses a `doskey` macro and two scripts shipped beside the executable. Activate the current session with the full path to the installed initializer:

```batch
call "%LOCALAPPDATA%\Programs\goto\goto-init.cmd"
```

For a manually extracted archive, replace the path above with its extraction directory. For future CMD sessions, call `goto-init.cmd` from your existing CMD `AutoRun` script. Preserve any existing `HKCU\Software\Microsoft\Command Processor\AutoRun` value instead of replacing it. In `.bat` programs, use `call goto-shell.cmd BB`, because the native batch `goto` statement retains its normal meaning.

## Usage

```text
# Explicit path and name
goto -s C:\Source\MyWork --name MyWork
goto save C:\Source\MyWork --name MyWork

# Current directory and explicit name
goto -s --name MyWork

# Infer the name from the directory
goto -s C:\Source\MyWork
goto -s

# Navigate, list, and remove
goto MyWork
goto -l
goto list
goto -r MyWork
goto remove MyWork
```

Names are compared without ASCII case sensitivity, so `MyWork` and `mywork` identify the same workplace. Names may contain ASCII letters, numbers, dots, hyphens, and underscores, and must start with a letter or number. Use `goto -- list` if a bookmark has the same name as a subcommand.

The target must be an existing directory when saved. If it is later deleted, `goto` reports the stale bookmark rather than silently removing it.

## Configuration

Bookmarks are stored as a plain JSON object:

```json
{
  "MyWork": "C:\\Source\\MyWork"
}
```

Default locations:

| Platform | Location |
| --- | --- |
| Windows | `%APPDATA%\goto\workplaces.json` |
| Linux | `$XDG_CONFIG_HOME/goto/workplaces.json` or `~/.config/goto/workplaces.json` |
| macOS | `~/Library/Application Support/goto/workplaces.json` |

Set `GOTO_CONFIG` to use a different file. Writes are locked and atomically replaced so simultaneous commands cannot corrupt or discard entries.

## Development

```text
cargo fmt -- --check
cargo clippy --all-targets -- -D warnings
cargo test
```

The project is licensed under the [MIT License](LICENSE).
