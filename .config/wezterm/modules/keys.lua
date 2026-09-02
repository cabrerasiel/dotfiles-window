local wezterm = require("wezterm")
local act = wezterm.action

local M = {}

function M.apply(config)
	config.disable_default_key_bindings = true
	config.leader = { key = "a", mods = "CTRL", timeout_milliseconds = 1000 }

	local choices = {
		{ id = "abrir_htop", label = " 📊 Ver procesos del sistema (htop)" },
		{ id = "ir_a_proyectos", label = " 📁 Abrir explorador de proyectos" },
		{ id = "limpiar_consola", label = " 🧹 Limpiar buffers de WezTerm" },
		{ id = "reiniciar_shell", label = " 🔄 Recargar entorno de la terminal" },
	}

	local keys = {
		{ key = "L", mods = "CTRL", action = act.ShowDebugOverlay },
		{ key = "a", mods = "LEADER|CTRL", action = act.SendKey({ key = "a", mods = "CTRL" }) },
		{ key = "[", mods = "LEADER", action = act.ActivateCopyMode },
		{ key = ":", mods = "LEADER", action = act.ActivateCommandPalette },

		{ key = "s", mods = "LEADER", action = act.ShowLauncherArgs({ flags = "FUZZY|WORKSPACES" }) },

		{
			key = "w",
			mods = "LEADER",
			action = act.SplitHorizontal({
				args = {
					"powershell.exe",
					"-NoProfile",
					"-Command",
					-- 1. Forzamos a guardar el ID de este panel efímero convirtiéndolo a número entero ([int])
					"$current_pane = [int]$env:WEZTERM_PANE; "
						-- 2. Traemos todos los paneles una sola vez en memoria
						.. "$all_panes = wezterm cli list --format json | ConvertFrom-Json; "
						-- 3. Obtenemos el nombre exacto de tu Workspace basándonos en el panel actual en el que estamos parados
						.. "$ws = ($all_panes | Where-Object { $_.pane_id -eq $current_pane }).workspace; "
						-- 4. FILTRO IMPENETRABLE: Mismo workspace Y convertimos $_.pane_id a [int] para que la comparación matemática sea exacta
						.. "$pane = ($all_panes | Where-Object { $_.workspace -eq $ws -and [int]$_.pane_id -ne $current_pane } | "
						-- 5. Le damos formato visual limpio para fzf (ID | Pestaña | Título)
						.. 'ForEach-Object { "$($_.pane_id) | TAB: $($_.tab_id) | $($_.title)" } | '
						.. 'fzf --no-preview --header="Tabs en Workspace: $ws"); '
						-- 6. Al presionar enter, aislamos el primer elemento (el ID numérico limpio) y saltamos de forma segura
						.. "if ($pane) { $id = $pane.Split(' ')[0]; wezterm cli activate-pane --pane-id $id }",
				},
			}),
		},

		{ key = "c", mods = "LEADER", action = act.SpawnTab("CurrentPaneDomain") },

		-- Abre el CWD real del shell como nuevo workspace (nombre = última carpeta).
		-- Usa wezterm CLI para leer el CWD del proceso via OS (no depende de OSC 7).
		-- Si ya existe un WS con ese nombre, salta a él sin crear uno nuevo.
		{
			key = "o",
			mods = "LEADER",
			action = wezterm.action_callback(function(window, pane)
				local pane_id = pane:pane_id()
				local workingDir = pane.home_dir

				wezterm.log_info(workingDir)
				local ok, stdout = wezterm.run_child_process({
					"wezterm",
					"cli",
					"list",
					"--format",
					"json",
				})
				if not ok or not stdout or stdout == "" then
					return
				end

				local ok2, list = pcall(wezterm.json_parse, stdout)
				if not ok2 or not list then
					return
				end

				local raw_cwd = nil
				for _, p in ipairs(list) do
					if p.pane_id == pane_id then
						raw_cwd = p.cwd
						break
					end
				end
				if not raw_cwd or raw_cwd == "" then
					return
				end

				-- Normaliza URI a path: file:///C:/... -> C:/...
				local cwd = raw_cwd:gsub("^file://[^/]*", "")
				cwd = cwd:match("^/([A-Za-z]:.+)") or cwd
				cwd = cwd:gsub("\\", "/"):gsub("/$", "")

				local ws_name = cwd:match("([^/]+)$") or cwd

				for _, name in ipairs(wezterm.mux.get_workspace_names()) do
					if name == ws_name then
						window:perform_action(act.SwitchToWorkspace({ name = ws_name }), pane)
						return
					end
				end

				window:perform_action(act.SwitchToWorkspace({ name = ws_name, spawn = { cwd = cwd } }), pane)
			end),
		},
		--{ key = "p", mods = "LEADER", action = act.ActivateTabRelative(-1) },
		{ key = "n", mods = "LEADER", action = act.ActivateTabRelative(1) },
		{ key = "p", mods = "LEADER", action = act.ActivateCommandPalette },
		{
			key = ",",
			mods = "LEADER",
			action = act.PromptInputLine({
				description = wezterm.format({
					{ Attribute = { Intensity = "Bold" } },
					{ Foreground = { AnsiColor = "Fuchsia" } },
					{ Text = "Renaming Tab Title...:" },
				}),
				action = wezterm.action_callback(function(window, _, line)
					if line then
						window:active_tab():set_title(line)
					end
				end),
			}),
		},

		{ key = ".", mods = "LEADER", action = act.ActivateKeyTable({ name = "move_tab", one_shot = false }) },

		-- Pane
		{ key = '"', mods = "LEADER", action = act.SplitVertical({ domain = "CurrentPaneDomain" }) },
		{ key = "%", mods = "LEADER", action = act.SplitHorizontal({ domain = "CurrentPaneDomain" }) },
		{ key = "h", mods = "CTRL", action = act.ActivatePaneDirection("Left") },
		{ key = "j", mods = "CTRL", action = act.ActivatePaneDirection("Down") },
		{ key = "k", mods = "CTRL", action = act.ActivatePaneDirection("Up") },
		{ key = "l", mods = "CTRL", action = act.ActivatePaneDirection("Right") },
		{ key = "phys:Space", mods = "LEADER", action = act.RotatePanes("Clockwise") },
		{ key = "z", mods = "LEADER", action = act.TogglePaneZoomState },
		{ key = "x", mods = "LEADER", action = act.CloseCurrentPane({ confirm = true }) },
		{
			key = "!",
			mods = "LEADER | SHIFT",
			action = wezterm.action_callback(function(win, pane)
				local tab, window = pane:move_to_new_tab()
			end),
		},
		-- We can make separate keybindings for resizing panes
		-- But Wezterm offers custom "mode" in the name of "KeyTable"
		{ key = "r", mods = "LEADER", action = act.ActivateKeyTable({ name = "resize_pane", one_shot = false }) },
	}

	for i = 1, 9 do
		table.insert(keys, {
			key = tostring(i),
			mods = "ALT",
			action = act.ActivateTab(i - 1),
		})
	end

	config.key_tables = {
		resize_pane = {
			{ key = "<", action = act.AdjustPaneSize({ "Left", 1 }) },
			{ key = "-", action = act.AdjustPaneSize({ "Down", 1 }) },
			{ key = "+", action = act.AdjustPaneSize({ "Up", 1 }) },
			{ key = ">", action = act.AdjustPaneSize({ "Right", 1 }) },
			{ key = "Escape", action = "PopKeyTable" },
			{ key = "Enter", action = "PopKeyTable" },
		},
		move_tab = {
			{ key = "h", action = act.MoveTabRelative(-1) },
			{ key = "j", action = act.MoveTabRelative(-1) },
			{ key = "k", action = act.MoveTabRelative(1) },
			{ key = "l", action = act.MoveTabRelative(1) },
			{ key = "Escape", action = "PopKeyTable" },
			{ key = "Enter", action = "PopKeyTable" },
		},
	}

	config.keys = keys
end

return M
