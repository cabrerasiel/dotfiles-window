local wezterm = require("wezterm")
local act = wezterm.action
local M = {}

function M.apply(config)
	local default_workspace_name = "Principal"
	config.default_workspace = default_workspace_name

	local home = wezterm.home_dir
	local workspaces = {
		{ id = home .. "/.config", label = default_workspace_name },
		{ id = home .. "/source/repos", label = "Coding" },
		{ id = home .. "/.config", label = "Config" },
	}

	-- Ctrl+1..N: workspaces predefinidos (los crea si no existen con spawn+cwd)
	for i, ws in ipairs(workspaces) do
		local label = ws.label
		local cwd = ws.id
		table.insert(config.keys, {
			key = tostring(i),
			mods = "CTRL",
			action = act.SwitchToWorkspace({ name = label, spawn = { cwd = cwd } }),
		})
	end

	-- Ctrl+(N+1)..9: workspaces creados manualmente, en orden de aparición
	local predefined_count = #workspaces
	for i = predefined_count + 1, 9 do
		local idx = i
		table.insert(config.keys, {
			key = tostring(i),
			mods = "CTRL",
			action = wezterm.action_callback(function(window, pane)
				local predefined = {}
				for _, ws in ipairs(workspaces) do
					predefined[ws.label] = true
				end
				local extras = {}
				for _, name in ipairs(wezterm.mux.get_workspace_names()) do
					if not predefined[name] then
						table.insert(extras, name)
					end
				end
				local extra_idx = idx - predefined_count
				if extras[extra_idx] then
					window:perform_action(act.SwitchToWorkspace({ name = extras[extra_idx] }), pane)
				end
			end),
		})
	end

	-- Alt+w: selector dinámico — muestra todos los WS activos + predefinidos no creados aún
	table.insert(config.keys, {
		key = "w",
		mods = "ALT",
		action = wezterm.action_callback(function(window, pane)
			local existing = {}
			local choices = {}

			for _, name in ipairs(wezterm.mux.get_workspace_names()) do
				existing[name] = true
				table.insert(choices, { id = name, label = name })
			end

			-- Predefinidos que todavía no se han abierto
			for _, ws in ipairs(workspaces) do
				if not existing[ws.label] then
					table.insert(choices, { id = ws.id, label = ws.label })
				end
			end

			window:perform_action(
				act.InputSelector({
					title = "Select Workspace",
					choices = choices,
					action = wezterm.action_callback(function(w, p, id, label)
						if not id or not label then return end
						if existing[label] then
							-- Ya existe: solo salta
							w:perform_action(act.SwitchToWorkspace({ name = label }), p)
						else
							-- No existe aún: créalo con el cwd del predefinido
							w:perform_action(
								act.SwitchToWorkspace({ name = label, spawn = { cwd = id } }),
								p
							)
						end
					end),
				}),
				pane
			)
		end),
	})

	-- Alt+Shift+W: crear/ir a workspace por nombre libre
	table.insert(config.keys, {
		key = "W",
		mods = "ALT|SHIFT",
		action = act.PromptInputLine({
			description = "Workspace's Name:",
			action = wezterm.action_callback(function(window, pane, line)
				if line and line ~= "" then
					window:perform_action(act.SwitchToWorkspace({ name = line }), pane)
				end
			end),
		}),
	})
end

return M
