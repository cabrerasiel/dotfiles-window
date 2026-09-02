local wezterm = require("wezterm")
local M = {}

function M.apply(config)
	-- 2. Visuals & Theme
	config.color_schemes = {
		["Retro 82"] = {
			foreground = "#f6dcac",
			background = "#05182e",
			cursor_bg = "#faa968",
			cursor_border = "#faa968",
			cursor_fg = "#05182e",
			selection_bg = "#134e5a",
			selection_fg = "#f6dcac",
			ansi = {
				"#031222", -- black
				"#f85525", -- red
				"#028391", -- green
				"#e97b3c", -- yellow
				"#3f8f8a", -- blue
				"#3f8f8a", -- magenta
				"#8cbfb8", -- cyan
				"#a7c9c6", -- white
			},
			brights = {
				"#0a2540", -- bright black
				"#f85525", -- bright red
				"#028391", -- bright green
				"#e97b3c", -- bright yellow
				"#faa968", -- bright blue
				"#3f8f8a", -- bright magenta
				"#8cbfb8", -- bright cyan
				"#f6dcac", -- bright white
			},
		},
	}
	config.color_scheme = "Retro 82"
	config.font = wezterm.font("JetBrains Mono", { weight = "Bold", italic = true })
	config.font_size = 12.0
	config.line_height = 1.175
	config.default_cursor_style = "BlinkingBar"
	config.disable_default_key_bindings = true

	local palette = M.get_palette(config)
	local SELECTOR_BG = palette.background -- Color de fondo del menú desplegable
	local SELECTOR_FG = palette.foreground -- Color del texto de los elementos no seleccionados

	config.command_palette_bg_color = SELECTOR_BG
	config.command_palette_fg_color = SELECTOR_FG

	config.command_palette_font_size = 16

	-- Estilo del input/buscador superior
	config.command_palette_rows = 12
end

function M.get_palette(config)
	local schemes = wezterm.color.get_builtin_schemes()
	local palette = schemes[config.color_scheme] or (config.color_schemes and config.color_schemes[config.color_scheme])

	if not palette then
		palette = schemes["Tokyo Night"] -- O cualquier tema por defecto
	end

	return palette
end

M.custom_colors = {
	red = "#D06F79", --> Tokyonight: "#F7768e"
	cyan = "#88C0D0", --> Tokyonight: "#7DCFFF"
	magenta = "#B48EAD", --> Tokyonight: "#BB9AF7"
	yellow = "#EBCB8B", --> Tokyonight: "#E0AF68"
}

function M.set_statusbar_colors(config)
	local palette = M.get_palette(config)
	config.colors = config.colors or {}
	config.colors.tab_bar = {
		background = palette.background,
		active_tab = {
			bg_color = palette.background,
			fg_color = palette.foreground,
		},
		inactive_tab = {
			bg_color = palette.background,
			fg_color = palette.ansi[5],
		},
		inactive_tab_hover = {
			bg_color = palette.background,
			fg_color = palette.ansi[5],
		},
		new_tab = {
			bg_color = palette.background,
			fg_color = palette.ansi[5],
		},
		new_tab_hover = {
			bg_color = palette.background,
			fg_color = palette.foreground,
		},
	}
end
return M
