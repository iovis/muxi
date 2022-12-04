# Muxi

## TODO

- [ ] Create `muxi` command
    - [ ] `init`
        - [ ] Set `$MUXI_CONFIG_PATH` if not exists
            - `$XDG_CONFIG_HOME/muxi/`
            - Create folder if it doesn't exist
        - [ ] Read `$MUXI_CONFIG_PATH/settings.toml`
            ```toml
            bindings = true             # default `true`
            muxi_prefix = 'g'           # no default
            tmux_prefix = true          # default `true`
            uppercase_overrides = true  # default `true`
            ```
        - [ ] Read `$MUXI_CONFIG_PATH/sessions.muxi` (gitignore)
            - `key session_name path`
            ```
            d dotfiles ~/.dotfiles
            k muxi ~/Sites/rust/muxi/
            Space tmux ~/Sites/rust/tmux/
            M-n notes ~/Library/Mobile Documents/com~apple~CloudDocs/notes (note spaces)
            ```
        - [ ] Set bindings
            - [ ] Clear table
                - `tmux unbind -aq -T muxi`
            - [ ] Muxi table prefix
                - `if settings.tmux_prefix` `tmux bind <settings.prefix> switch-client -T muxi`
                - `else` `tmux bind -n <settings.prefix> switch-client -T muxi`
            - [ ] Bookmarks
                - `tmux bind -T muxi <session_key> new-session -A -s <name> -c "<path>"`
            - [ ] `if uppercase_overrides`
                - Should I define bindings for all letters?
                - `tmux bind -T muxi <key.upper> run -b "muxi set <key>"`
    - [ ] `go <name>`
        - [ ] If not in list, display error
            - `tmux display "#{session} doesn't exist"`
        - [ ] If in list
            - `tmux has-session -t "$session_name" || tmux new-session -d -s "$session_name" -c "$session_path"`
            - `tmux switch-client -t "$session_name"`
        - [ ] How to do autocomplete?
    - [ ] `set <key>`
        - [ ] Set <key> to current session name and current session path
            - Optional `--name` and `--path`?
        - [ ] Reload
    - [ ] `del <key>`
        - [ ] Delete <key>
        - [ ] Reload
    - [ ] `edit`
        - [ ] `$EDITOR $MUXI_CONFIG_PATH/sessions.toml`
        - [ ] Reload
    - [ ] popup switcher?
        - fzf-tmux or custom (dialoguer, requestty)?
        - bindings
            - up/down
            - go to session
            - delete session
- [ ] Create TPM plugin?
    - [Source](https://github.com/tmux-plugins/tpm/blob/master/docs/how_to_create_plugin.md)
