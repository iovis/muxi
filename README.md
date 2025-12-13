# Muxi

Create dynamic shortcuts for your tmux sessions!

Stop wasting time navigating between tmux sessions. Muxi lets you create keyboard shortcuts for your most-used sessions, switch between them with FZF, and manage everything from a simple config file.

## Features

- 🚀 **Dynamic session bookmarks** - Create keyboard shortcuts for any tmux session
- ⚡ **FZF integration** - Fuzzy-find and switch sessions instantly
- 🎯 **Lua configuration** - Flexible, programmable config with sensible defaults
- 🔌 **Plugin management** - Install and update tmux plugins
- 🎨 **Multiple workflows** - Native tmux menu, FZF popup, or custom bindings
- 📝 **Simple session file** - Edit your sessions in TOML with your favorite editor

## Quick Start

1. Install muxi:
```sh
cargo install muxi
```

2. Add to your `tmux.conf`:
```tmux
if "type muxi" {
  run -b "muxi init"
}
```

3. Reload tmux config:
```sh
tmux source ~/.tmux.conf
```

4. Create your first session bookmark: `<prefix>gJ` will make a bookmark for `j`

5. Now press `<prefix>gj` to jump to your muxi session instantly!

## Installation

### From crates.io

```sh
cargo install muxi
```

### From source

```sh
git clone https://github.com/iovis/muxi
cd muxi
cargo install --path .
```

## Usage

```
❯ muxi
Create bookmarks for your tmux sessions on the fly! 🚀

Usage: muxi <COMMAND>

Commands:
  init         Register within Tmux and add bindings
  config       See and edit your settings [aliases: c]
  ls           List sessions
  sessions     See and manage your muxi sessions [aliases: s]
  plugins      See and manage your tmux plugins [aliases: p]
  completions  Generate completions for your shell
  fzf          Spawn a FZF popup to manage your muxi sessions [aliases: f]
  help         Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## Configuration

### Lua

You can provide an `init.lua` in one of the following locations:

- `$MUXI_CONFIG_PATH/init.lua`
- `$XDG_CONFIG_HOME/muxi/init.lua`
- `~/.config/muxi/init.lua`

Or run `muxi config edit` to open it in your favorite `$EDITOR`

```lua
return {
  -- Optional: Use tmux <prefix> to define muxi's table (default: true)
  tmux_prefix = true

  -- Optional: Muxi's table binding (default: "g")
  muxi_prefix = "g" -- will bind to <prefix>g if tmux_prefix is true

  -- Optional: Uppercase letters will set the current session (default: true)
  uppercase_overrides = true

  -- Optional: Set current session path to current pane's path (default: false)
  use_current_pane_path = false

  -- Optional: open editor with certain arguments
  editor = {
    command = "nvim", -- (default: $EDITOR or "vi")
    args = { "+ZenMode", "-c", "nmap q <cmd>silent wqa<cr>" }, -- (default: {})
  },

  -- Optional: Define tmux plugins
  plugins = {
    "tmux-plugins/tmux-continuum",
    "tmux-plugins/tmux-resurrect",
    "tmux-plugins/tmux-yank",
  },

  -- Optional: FZF integration
  -- Use <alt-x> to navigate directly to session `x`
  fzf = {
    input = false,  -- Use --no-input (default: false)
    bind_sessions = false,  -- Bind the key of the session to switch to it (default: false)
    args = { "--color=input-border:black" }, -- Extra arguments for FZF (default: {})
  },

  -- Optional bindings to be created on tmux muxi table (Examples shown)
  bindings = {
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

    -- <prefix>g/ => FZF integration
    ["/"] = { command = "muxi fzf" },

    -- <prefix>gt => session switcher (native tmux menu)
    t = { command = "muxi sessions switch --tmux-menu" },

    -- You can bind your own commands too!
    -- `tmux run-shell "tmux switch-client -l"`
    ["Space"] = { command = "tmux switch-client -l" },

    g = { command = "tmux new-window htop" },
  }
}
```

And start `muxi` in your `tmux.conf`:

```tmux
# ~/.tmux.conf
if "type muxi" {
  run -b "muxi init"
}
```

### Tmux variables

You can alternatively define settings entirely from your tmux config:

```tmux
# If you're going to define bindings on the muxi table, don't use `-b`
run "muxi init"

# Defining bindings on the muxi table:
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

This is the file that `muxi` will use to generate your session bindings and keep state. After exiting your editor, `muxi` will re-sync the sessions (same with your configuration!)

### Session Commands

```sh
# Set a session (create or update)
muxi sessions set KEY

# List all sessions
muxi sessions list

# Manage sessions with an FZF popup (`?` for a list of shortcuts)
muxi fzf

# Switch sessions using native tmux menu
muxi sessions switch --tmux-menu

# Edit sessions in your $EDITOR
muxi sessions edit

# Delete a session
muxi sessions delete KEY
```

## Plugins

Muxi provides support for managing tmux plugins. Plugins are cloned from GitHub and stored in `$XDG_DATA_HOME/muxi/plugins/` (or `~/.local/share/muxi/plugins/`).

### Configuration

Define plugins in your `init.lua`:

```lua
return {
  plugins = {
    "tmux-plugins/tmux-continuum",
    "tmux-plugins/tmux-resurrect",
    "https://gitlab.com/username/my-plugin",
    { path = "~/code/tmux/my-plugin/" },
    {
      url = "tmux-plugins/tmux-cpu",
      opts = {
        -- will be evaluated as tmux variables
        -- Ex: set -g @cpu_low_fg_color "#[fg=#51576d]"
        cpu_low_fg_color = "#[fg=#51576d]",
        cpu_medium_fg_color = "#[fg=#e5c890]",
        cpu_high_fg_color = "#[fg=#e78284]",
      },
    },
  },
}
```

### Commands

```
❯ muxi plugins help
See and manage your tmux plugins

Usage: muxi plugins [COMMAND]

Commands:
  init     Sources all plugins
  list     Print your current tmux plugins
  install  Install plugins
  update   Update plugins
  help     Print this message or the help of the given subcommand(s)
```

### Sourcing Plugins

To automatically source your plugins in tmux, add this to your `tmux.conf`:

```tmux
# You can still set plugin options in your tmux.conf
set -g @continuum-save-interval '5'

if "type muxi" {
  run -b "muxi init"
  run -b "muxi plugins init || tmux display 'muxi: Failed to init plugins'"
}
```

> [!NOTE]
> `muxi init` and `muxi plugins init` are independent; you can use one without the other.

## Why Muxi?

Muxi is designed for developers who want a lightweight, flexible way to manage tmux sessions without the overhead of full session managers.

**vs tmuxinator/tmuxp**: No YAML files, no session templates. Just bookmarks with keyboard shortcuts. Create sessions on the fly, not in advance.

**vs tmux-resurrect**: Muxi doesn't save/restore your entire environment. It gives you quick access to session directories you care about.

**vs native tmux**: Muxi adds the missing keyboard shortcuts layer. Instead of `<prefix>s` → navigate list → find session → enter, just press `<prefix>gm`.

**vs custom shell scripts**: Muxi handles tmux integration, path resolution, config reloading, and provides a consistent UI.

## Troubleshooting

### Command not found after install

Make sure `~/.cargo/bin` is in your `$PATH`:

```sh
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
# or ~/.zshrc, ~/.config/fish/config.fish, etc.
```

### Tmux doesn't detect muxi

Check that muxi is in your PATH from within tmux:

```sh
tmux run "which muxi"
```

If empty, add the full path to your `tmux.conf`:

```tmux
if "test -x $HOME/.cargo/bin/muxi" {
    run -b "$HOME/.cargo/bin/muxi init"
}
```

### Bindings don't work

1. Verify muxi is initialized: `muxi --help`
2. Reload your tmux config: `tmux source-file ~/.tmux.conf`
3. Check your muxi table bindings: `tmux list-keys -T muxi`

### Editor doesn't open

Muxi uses `$EDITOR` by default. Set it in your shell config:

```sh
export EDITOR=nvim  # or vim, code, etc.
```

Or override in your `init.lua`:

```lua
return {
  editor = { command = "nvim" },
}
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

MIT
