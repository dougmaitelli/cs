# cs — CLI Snippet Manager

`cs` is a command-line snippet manager written in Rust. It stores shell commands and provides fast search, selection, and execution.

[![cs demo](./demo.gif)](./demo.gif)

## Installation

```bash
curl -fsSL https://raw.githubusercontent.com/dougmaitelli/cs/master/install.sh | bash
```

## Commands

| Command | Description |
|---------|-------------|
| `cs <query>` | Search snippets, select, edit inline, execute (filtered by current OS) |
| `cs add` | Interactive prompts to create a new snippet |
| `cs import` | Import snippets (interactive) |
| `cs list` | Show all snippets (filtered by current OS) |
| `cs delete <query>` | Search, select, confirm deletion |

## Global Flags

| Flag | Description |
|------|-------------|
| `--all-os` | Show/search snippets for all operating systems |
| `--no-fzf` | Disable fzf and use the built-in selector |

## Configuration

- **Primary path**: `$XDG_CONFIG_HOME/cs/snippets.toml`
- **Fallback path**: `~/.config/cs/snippets.toml`

### Configuration

```toml
[config]
nerd_fonts = true
includes = ["extra.toml"]
```

### Fields

- **nerd_fonts**: Enable nerd font icons in the UI (default: `false`)
- **includes**: Additional TOML files to load snippets from (relative to config directory)

### Snippet Structure

```toml
[[snippet]]
cmd = "dnf update -y"
description = "Update all Fedora packages"
tags = ["fedora", "dnf", "update", "packages"]
os = "fedora"
```

### Fields

- **cmd**: The shell command to execute (required)
- **description**: Human-readable description (required)
- **tags**: List of searchable tags (required, can be empty)
- **os**: Target operating system (required)

