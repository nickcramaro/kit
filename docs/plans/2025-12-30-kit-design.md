# kit - CLI Tool Manager

A Rust CLI tool for tracking, installing, and managing your command-line tools.

## Use Cases

1. **Machine setup** - Know exactly what to install on a new machine
2. **Audit** - See what's installed, find conflicts
3. **Alias management** - Single source of truth for shell aliases and symlinks
4. **Export** - Sync your tool config with dotfiles

## Config Format

**Location:** `~/.config/kit/kit.toml`

```toml
[config]
bin_dir = "~/.kit/bin"
shell_rc = "~/.zshrc"

[tools.ripgrep]
source = "brew"
aliases = ["rg"]

[tools.python]
source = "mise"
version = "3.12"

[tools.starship]
source = "curl"
install_url = "https://starship.rs/install.sh"
binary = "starship"
```

### Sources

- `brew` - Homebrew
- `mise` - mise-managed tools
- `curl` - curl install scripts (rustup, starship, etc.)

### Curl Source

For tools installed via curl | sh scripts:

```toml
[tools.starship]
source = "curl"
install_url = "https://starship.rs/install.sh"
binary = "starship"  # verify this exists after install
```

## Commands

```
kit scan              # Discover PATH, diff against config
kit list              # Show configured tools + install status
kit install <tool>    # Install using source from config
kit install --all     # Install everything in kit.toml
kit aliases           # Regenerate aliases + re-inject into .zshrc
kit export            # Output kit.toml to stdout
kit add <tool>        # Detect source, add to config interactively
```

### Scan Output

```
$ kit scan

Configured & installed:
  ✓ ripgrep (brew)
  ✓ python (mise)

Configured but missing:
  ✗ starship (curl) - run `kit install starship`

Found but not in config:
  ? bat (/opt/homebrew/bin/bat) - run `kit add bat`
  ? fd (/opt/homebrew/bin/fd)

Conflicts:
  ⚠ python: ~/.local/share/mise/... shadows /usr/bin/python
```

## Shell Integration

### Directory Structure

```
~/.kit/
  bin/           # Symlinks to actual binaries
  aliases.zsh    # Generated alias definitions
```

### Generated Aliases

```zsh
# ~/.kit/aliases.zsh - managed by kit, do not edit
alias rg="ripgrep"
alias cat="bat"
```

### Managed Injection

Kit adds this block to `.zshrc` (once):

```zsh
# >>> kit >>>
export PATH="$HOME/.kit/bin:$PATH"
source "$HOME/.kit/aliases.zsh"
# <<< kit <<<
```

- Only touches code between markers
- Re-running `kit aliases` regenerates file, doesn't duplicate injection
- `kit aliases` runs automatically after `kit install`

### Symlinks

For ripgrep with `aliases = ["rg"]`:
- `~/.kit/bin/rg` → `/opt/homebrew/bin/rg`

Provides both alias (interactive) and symlink (scripts).

## Scanning & Discovery

### Process

1. Walk every directory in `$PATH`
2. List all executables
3. Classify by source:
   - Check `brew list --formula`
   - Check `mise ls`
   - Otherwise mark "unknown"
4. Diff against kit.toml
5. Detect conflicts (same binary in multiple PATH locations)

### Classification

```
if binary in brew_list → brew
else if path contains mise → mise
else → unknown
```

### Performance

- Cache brew/mise output at scan start
- No background daemon
- Should complete in seconds

## Rust Implementation

### Crate Structure

```
kit/
  src/
    main.rs          # CLI entry (clap)
    config.rs        # kit.toml parsing (serde + toml)
    scanner.rs       # PATH walking, source detection
    sources/
      mod.rs
      brew.rs
      mise.rs
      curl.rs
    shell.rs         # alias generation, zshrc injection
    commands/
      scan.rs
      list.rs
      install.rs
      add.rs
      aliases.rs
      export.rs
```

### Dependencies

- `clap` - CLI parsing
- `serde` + `toml` - config
- `walkdir` - PATH scanning
- `which` - binary resolution
- `dialoguer` - interactive prompts

### Principles

- Non-destructive: never deletes tools, only manages config/symlinks
- Explicit errors with context
- No async needed - all operations are quick local commands
- Zsh only for shell integration
