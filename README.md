# Muxi

Create dynamic shortcuts for your tmux sessions!

## Install
```sh
cargo install muxi
```

## Configuration

In your tmux configuration:
```tmux
if "type muxi" {
  run -b "muxi init"
}
```

Then provide a `settings.toml` in one of the following locations:
- `$MUXI_CONFIG_PATH`
- `$XDG_CONFIG_HOME/muxi/`
- `~/.config/muxi/`

```toml
muxi_prefix = "g"   # Muxi's table binding, `<prefix>g`
tmux_prefix = true  # Optional: Use <prefix> to define muxi's table (default: true)

# Optional bindings to be created on tmux's muxi table
[bindings]
e = { popup = true, command = "muxi sessions edit" }  # <prefix>ge => open a tmux popup to edit your sessions file
l = { command = "muxi sessions list" }  # <prefix>gl => tmux run-shell <command>
```

## Sessions

Running `muxi sessions edit` will open your `sessions.toml` file, which should look something like the following:

```toml
# <key> = { name = <session_name>, path = <session_path> }
d = { name = "dotfiles", path = "~/.dotfiles" }
m = { name = "muxi", path = "~/Sites/rust/muxi/" }
n = { name = "notes", path = "~/Library/Mobile Documents/com~apple~CloudDocs/notes" }
```

This is the file that `muxi` will use to generate your session bindings and keep state
