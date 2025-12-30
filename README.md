# kit

A CLI tool for tracking, installing, and managing your command-line tools.

## Use Cases

- **Machine setup** - Know exactly what to install on a new machine
- **Audit** - See what's installed, find what's not in your config
- **Alias management** - Single source of truth for shell aliases
- **Export** - Sync your tool config with dotfiles

## Installation

```bash
cargo build --release
mkdir -p ~/.kit/bin
cp target/release/kit ~/.kit/bin/

# Set up PATH (run once, then restart shell or source ~/.zshrc)
~/.kit/bin/kit setup
```

## Quick Start

```bash
# Scan your PATH to see what's installed
kit scan

# Add a tool interactively
kit add ripgrep

# Install all configured tools
kit install --all

# Regenerate aliases and symlinks
kit regen
```

## Commands

| Command | Description |
|---------|-------------|
| `kit setup` | Check dependencies, inject PATH into shell rc |
| `kit scan` | Discover binaries in PATH, diff against config |
| `kit list` | Show configured tools with install status |
| `kit add <tool>` | Interactively add a tool to config |
| `kit install [tool]` | Install a specific tool from config |
| `kit install --all` | Install all tools from config |
| `kit regen` | Regenerate aliases and symlinks |
| `kit export` | Output kit.toml to stdout |

## Configuration

Config lives at `~/.config/kit/kit.toml`:

```toml
[config]
bin_dir = "~/.kit/bin"
shell_rc = "~/.zshrc"

# Homebrew tools
[tools.ripgrep]
source = "brew"
aliases = ["rg"]

[tools.bat]
source = "brew"
aliases = ["cat"]

# Mise-managed tools (version managers)
[tools.python]
source = "mise"
version = "3.12"

[tools.node]
source = "mise"
version = "20"

# Curl install scripts
[tools.starship]
source = "curl"
install_url = "https://starship.rs/install.sh"
binary = "starship"
```

## Sources

Kit supports three tool sources:

| Source | Description | Example |
|--------|-------------|---------|
| `brew` | Homebrew packages | ripgrep, bat, fd |
| `mise` | mise-managed runtimes | python, node, ruby |
| `curl` | curl \| sh install scripts | rustup, starship |

## Shell Integration

Running `kit setup` will:

1. Check that required tool sources (brew, mise, curl) are installed
2. Inject a managed block into your `.zshrc`:

Running `kit regen` will:

1. Create `~/.kit/bin/` with symlinks to your tools
2. Generate `~/.kit/aliases.zsh` with shell aliases

The injected block in `.zshrc`:

```zsh
# >>> kit >>>
export PATH="$HOME/.kit/bin:$PATH"
source "$HOME/.kit/aliases.zsh"
# <<< kit <<<
```

## License

MIT
