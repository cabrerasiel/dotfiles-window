local M = {}

function M.apply(config)
	-- 3. Window Behavior
	config.window_decorations = "RESIZE" -- Removes the default title bar for a sleeker look
	-- config.hide_tab_bar_if_only_one_tab = true
	config.window_background_opacity = 0.30 -- Adds subtle transparency
	config.win32_system_backdrop = "Acrylic" -- Enables native Windows acrylic blur effect
	config.window_padding = {
		left = 0,
		right = 0,
		top = 0,
		bottom = 0,
	}

	config.window_close_confirmation = "NeverPrompt"

	-- Enable Kitty graphics protocol so image previews (e.g. from Copilot CLI) render inline
	config.enable_kitty_graphics = true
end

return M
