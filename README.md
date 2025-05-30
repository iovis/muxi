# Muxi

Create dynamic shortcuts for your tmux sessions!

## Install

```sh
cargo install muxi
```

## Usage

```sh
❯ muxi
Create bookmarks for your tmux sessions on the fly! 🚀

Usage: muxi <COMMAND>

Commands:
  init         Register within Tmux and add bindings
  config       See and edit your settings
  sessions     See and manage your muxi sessions
  completions  Generate completions for your shell
  fzf          Spawn a FZF popup to manage your muxi sessions
  help         Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## Configuration

### Lua

You can provide an `init.lua` in one of the following locations:

- `$MUXI_CONFIG_PATH`
- `$XDG_CONFIG_HOME/muxi/`
- `~/.config/muxi/`

Or run `muxi config edit` to open it in your favorite `$EDITOR`

```lua
-- Optional: Use tmux <prefix> to define muxi's table (default: true)
muxi.tmux_prefix = true

-- Optional: Muxi's table binding (default: "g"), will result in `<prefix>g`
muxi.muxi_prefix = "g"

-- Optional: Uppercase letters will set the current session (default: false)
muxi.uppercase_overrides = false

-- Optional: Set current session path to current pane's path
muxi.use_current_pane_path = false

-- Optional bindings to be created on tmux's muxi table (Examples shown)
muxi.bindings = {
  -- <prefix>ge => edit your sessions file (You can pass optional arguments to your editor after "--")
  e = {
    popup = { title = " sessions " },
    command = "muxi sessions edit -- +ZenMode",
  },

  -- <prefix>gc => edit config
  c = {
    popup = {
      title = " config ",
      width = "75%",
      height = "60%",
    },
    command = "muxi config edit -- -c 'nmap <silent> q :wqa<cr>'",
  },

  -- <prefix>gs => session switcher
  s = { popup = { title = " muxi " }, command = "muxi sessions switch --interactive" },

  -- <prefix>gf => FZF integration
  f = { command = "muxi fzf" },

  -- <prefix>gt => session switcher (native tmux menu)
  t = { command = "muxi sessions switch --tmux-menu" },

  -- You can bind your own commands too!
  -- `tmux run-shell "tmux switch-client -l"`
  ["M-Space"] = { command = "tmux switch-client -l" },

  g = { command = "tmux send htop Enter" },
}
```

And start `muxi` in your `tmux.conf`:

```tmux
if "type muxi" {
    run -b "muxi init"
}
```

### Tmux variables

You can alternatively define settings entirely from your tmux config:

```tmux
# Init muxi
if "type muxi" {
  # If you're going to define bindings on the muxi table, don't use `-b`
  run "muxi init"
}

# Define bindings on the muxi table:
# <prefix>ge => Edit sessions in your editor
bind -T muxi e popup -w 76% -h 75% -b rounded -T " sessions " -E "muxi sessions edit -- +ZenMode -c 'nmap <silent> q :wqa<cr>'"

# TIP: Using the native tmux menu is a good alternative to the common workflow,
# it'll map your session bindings to the menu
bind 'f' run 'muxi sessions switch --tmux-menu'
```

## Sessions

Running `muxi sessions edit` will open your `sessions.toml` file, which should look something like the following:

```toml
# <key> = { name = <session_name>, path = <session_path> }
d = { name = "dotfiles", path = "~/.dotfiles" }
m = { name = "muxi", path = "~/Sites/rust/muxi/" }
n = { name = "notes", path = "~/Library/Mobile Documents/com~apple~CloudDocs/notes" }
```

This is the file that `muxi` will use to generate your session bindings and keep state. After exiting your editor, `muxi` will resync the sessions (same with your configuration!)
