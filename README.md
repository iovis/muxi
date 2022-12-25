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

Or run `muxi config edit`

```toml
muxi_prefix = "g"   # Muxi's table binding, `<prefix>g`
tmux_prefix = true  # Optional: Use tmux <prefix> to define muxi's table (default: true)

# Optional bindings to be created on tmux's muxi table
[bindings]
# `popup = true` will run your command on a tmux popup
e = { popup = true, command = "muxi sessions edit" }    # <prefix>ge => edit your sessions file
c = { popup = true, command = "muxi config edit" }      # <prefix>gc => edit config
s = { popup = true, command = "muxi sessions | less" }  # <prefix>gs => list sessions

M-Space = { command = "tmux switch-client -l" }  # `tmux run-shell "tmux switch-client -l"`

# Quick session bindings (like vim's harpoon)
J = { command = "muxi sessions set j && tmux display 'bound current session to j'" }
K = { command = "muxi sessions set k && tmux display 'bound current session to k'" }
L = { command = "muxi sessions set l && tmux display 'bound current session to l'" }

[bindings.g]
command = "tmux send htop Enter" # Make tmux do the hard work for you!
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
