local wezterm = require("wezterm")
local M = {}

function M.apply(config)
	-- 1. Set the default shell (Powershell, CMD, or WSL)
	-- For PowerShell Core:
	local is_windows = wezterm.target_triple == "x86_64-pc-windows-msvc"
	if is_windows then
		config.default_prog = { "pwsh.exe", "-NoLogo", "-ExecutionPolicy", "Bypass" }
	end
	require("modules.keys").apply(config)
	require("modules.mouse").apply(config)
	require("modules.tabar").apply(config)
	require("modules.theme").apply(config)
	require("modules.window").apply(config)
	require("modules.workspaces").apply(config)
end

return M
