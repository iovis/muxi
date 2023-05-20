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
# Optional: Use tmux <prefix> to define muxi's table (default: true)
tmux_prefix = true

# Optional: Muxi's table binding (default: "g"), will result in `<prefix>g`
muxi_prefix = "g"

# Optional: Uppercase letters will set the current session (default: false)
uppercase_overrides = false

# Optional bindings to be created on tmux's muxi table (Examples shown)
[bindings]
# <prefix>ge => edit your sessions file (You can pass optional arguments to your editor after "--")
e = { popup = { title = " sessions " }, command = "muxi sessions edit -- +ZenMode" }

# <prefix>gc => edit config
c = { popup = { title = " config " }, command = "muxi config edit" }

# <prefix>gs => session switcher
s = { popup = { title = " muxi " }, command = "muxi sessions switch --interactive" }

# <prefix>gt => session switcher (native tmux menu)
t = { command = "muxi sessions switch --tmux-menu" }

# You can bind your own commands too!
# `tmux run-shell "tmux switch-client -l"`
M-Space = { command = "tmux switch-client -l" }

[bindings.g]
command = "tmux send htop Enter"

[bindings.l]
# popup = {
#   title = "optional title",
#   width = "60%", (default: 75%)
#   height = "60%", (default: 75%)
# }
popup = { width = "75%", height = "60%" }
command = "muxi sessions | less"
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
