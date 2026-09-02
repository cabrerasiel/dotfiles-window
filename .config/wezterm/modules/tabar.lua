local wezterm = require("wezterm")
local theme = require("modules.theme")
local set_statusbar_colors = theme.set_statusbar_colors

local ARROW_RIGHT = ""

local M = {}

function M.apply(config)
	config.use_fancy_tab_bar = false
	config.status_update_interval = 1000
	config.show_new_tab_button_in_tab_bar = false

	set_statusbar_colors(config)

	local dot_symbol = wezterm.nerdfonts.cod_circle_filled
	local basename = function(s)
		--> Nothing a little regex can't fix
		return string.gsub(s, "(.*[/\\])(.*)", "%2")
	end

	local process_label = function(s)
		s = basename(s or ""):gsub("%.exe$", "")

		local labels = {
			nvim = "nvim",
			wezterm = "wezterm",
			["wezterm-gui"] = "wezterm",
			copilot = "copilot",
			pwsh = "pwsh",
			powershell = "powershell",
			cmd = "cmd",
			node = "node",
			git = "git",
		}

		return labels[s] or s
	end

	local normalize_path = function(path)
		if not path or path == "" then
			return ""
		end

		-- WezTerm returns Windows paths as /C:/Users/...; git -C expects C:/Users/...
		return path:gsub("^/([A-Za-z]:/)", "%1")
	end

	local git_cache = { cwd = nil, at = 0, info = nil }
	local git_info = function(cwd)
		if not cwd or cwd == "" then
			return nil
		end

		local now = os.time()
		if git_cache.cwd == cwd and now - git_cache.at < 5 then
			return git_cache.info
		end

		local ok, stdout = wezterm.run_child_process({ "git", "-C", cwd, "status", "--short", "--branch" })
		if not ok then
			git_cache = { cwd = cwd, at = now, info = nil }
			return nil
		end

		local lines = {}
		for line in stdout:gmatch("[^\r\n]+") do
			table.insert(lines, line)
		end

		local branch = (lines[1] or ""):gsub("^## ", ""):gsub("%.%.%..*", ""):gsub("%s%[.*%]", "")
		if branch == "" then
			git_cache = { cwd = cwd, at = now, info = nil }
			return nil
		end

		local info = { branch = branch, changes = math.max(#lines - 1, 0) }
		git_cache = { cwd = cwd, at = now, info = info }
		return info
	end

	wezterm.on("update-status", function(window, pane)
		local palette = theme.get_palette(config)
		local stat = window:active_workspace()
		local stat_bg = palette.background
		--> local stat_fg = wezterm.color.parse(palette.brights[5]):srgb_with_alpha(1.0)
		local stat_fg = palette.brights[5]

		if window:active_key_table() then
			stat = window:active_key_table()
			stat_fg = palette.ansi[5]
		end
		if window:leader_is_active() then
			stat = "LDR"
			stat_fg = palette.ansi[4]
		end

		local cwd = pane:get_current_working_dir()
		local cwd_path = ""
		local cwd_label = ""
		if cwd then
			cwd_path = normalize_path(cwd.file_path)
			cwd_label = basename(cwd_path) --> URL object introduced in 20240127-113634-bbcac864 (type(cwd) == "userdata")
		else
			cwd_label = ""
		end
		local git = git_info(cwd_path)

		local cmd = pane:get_foreground_process_name()
		--> CWD and CMD could be nil (e.g. viewing log using Ctrl-Alt-l)
		cmd = process_label(cmd)

		window:set_left_status(wezterm.format({
			{ Background = { Color = stat_fg } },
			{ Foreground = { Color = stat_bg } },
			{ Attribute = { Intensity = "Bold" } },
			{ Text = " " .. stat .. " " },

			{ Background = { Color = stat_bg } }, --> El fondo de la flecha se fusiona con la barra
			{ Attribute = { Intensity = "Normal" } },
			{ Foreground = { Color = stat_fg } }, --> El color de la flecha es el del workspace
			{ Text = ARROW_RIGHT .. " " },
		}))

		window:set_right_status(wezterm.format({
			{ Foreground = { Color = palette.brights[5] } },
			{ Text = wezterm.nerdfonts.fa_code .. "  " },
			{ Foreground = { Color = palette.foreground } },
			{ Attribute = { Intensity = "Bold" } },
			{ Text = cmd },
			{ Attribute = { Intensity = "Normal" } },
			{ Foreground = { Color = palette.ansi[5] } },
			{ Text = "  /  " },
			{ Foreground = { Color = palette.foreground } },
			{ Text = wezterm.nerdfonts.md_folder .. "  " .. cwd_label },
			{ Foreground = { Color = palette.ansi[5] } },
			{ Text = git and "  /  " or "" },
			{ Foreground = { Color = palette.ansi[5] } },
			{ Text = git and (wezterm.nerdfonts.dev_git_branch .. "  " .. git.branch) or "" },
			{ Foreground = { Color = palette.ansi[2] } },
			{ Text = git and git.changes > 0 and ("  " .. wezterm.nerdfonts.cod_circle_filled .. " " .. git.changes) or "" },
			{ Text = "  " },
		}))
	end)

	wezterm.on("format-tab-title", function(tab, _, _, _, _, _)
		local palette = theme.get_palette(config)
		if tab.is_active then
			return wezterm.format({
				{ Foreground = { Color = palette.ansi[2] } }, --> Rojo para el panel activo
				{ Text = "" .. dot_symbol .. " " },
			})
		else
			return wezterm.format({
				{ Foreground = { Color = palette.ansi[5] } },
				{ Text = "" .. dot_symbol .. " " },
			})
		end
	end)
end

return M
