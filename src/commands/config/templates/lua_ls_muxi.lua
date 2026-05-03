---@meta muxi

---@class (exact) muxi.Config
---@field muxi_prefix? string
---@field tmux_prefix? boolean
---@field uppercase_overrides? boolean
---@field use_current_pane_path? boolean
---@field editor? muxi.EditorSettings
---@field fzf? muxi.FzfSettings
---@field plugins? muxi.Plugin[]
---@field bindings? table<string, muxi.Binding>

---@class (exact) muxi.EditorSettings
---@field command? string
---@field args? string[]

---@class (exact) muxi.FzfSettings
---@field input? boolean
---@field bind_sessions? boolean
---@field args? string[]

---@class (exact) muxi.Binding
---@field command string
---@field popup? muxi.Popup

---@class (exact) muxi.Popup
---@field title? string
---@field width? string
---@field height? string

---@alias muxi.Plugin string|muxi.PluginSpec

---@class (exact) muxi.PluginSpec
---@field url? string
---@field path? string
---@field opts? table<string, string>

---@class (exact) muxi.Api
---@field config muxi.Config
---@field inspect fun(value: any): string
---@field merge fun(old: table, new: table): table
---@field print fun(...: any)

---@type muxi.Api
muxi = muxi
