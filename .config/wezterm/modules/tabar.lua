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
		if cwd then
			cwd = basename(cwd.file_path) --> URL object introduced in 20240127-113634-bbcac864 (type(cwd) == "userdata")
		else
			cwd = ""
		end

		local cmd = pane:get_foreground_process_name()
		--> CWD and CMD could be nil (e.g. viewing log using Ctrl-Alt-l)
		cmd = cmd and basename(cmd) or ""

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
			{ Text = wezterm.nerdfonts.md_folder .. "  " .. cwd },
			{ Text = " | " },
			{ Foreground = { Color = palette.ansi[1] } },
			{ Text = wezterm.nerdfonts.fa_code .. "  " .. cmd },
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
