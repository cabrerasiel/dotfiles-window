local M = {}

local state = {
	history = {},
	resources = {},
	model = "gpt-5-mini",
	system_prompt = "You are a precise coding assistant. Answer in the user's language.",
}

local conversation_buf
local conversation_win
local input_buf
local input_win
local source_buf
local source_win
local streaming_response = ""
local history_path = vim.fn.stdpath("data") .. "/copilot-panel/history.json"
local archive_path = vim.fn.stdpath("data") .. "/copilot-panel/chats"
local namespace = vim.api.nvim_create_namespace("copilot-panel")
local placeholder_ns = vim.api.nvim_create_namespace("copilot-panel-placeholder")
local PROMPT_LINES = 3

local function notify(message, level)
	vim.notify(message, level or vim.log.levels.INFO, { title = "Copilot Panel" })
end

local function is_valid(win)
	return win and vim.api.nvim_win_is_valid(win)
end

local function is_open()
	return is_valid(conversation_win) and is_valid(input_win)
end

local function resource_label(resource)
	return string.format("#%s: %s", resource.kind, resource.name)
end

local function set_highlights()
	vim.api.nvim_set_hl(0, "CopilotPanelTitle", { fg = "#6dd6d1", bold = true })
	vim.api.nvim_set_hl(0, "CopilotPanelLabel", { fg = "#88bfc1", bold = true })
	vim.api.nvim_set_hl(0, "CopilotPanelValue", { fg = "#e8d49a" })
	vim.api.nvim_set_hl(0, "CopilotPanelHint", { fg = "#5f8496" })
	vim.api.nvim_set_hl(0, "CopilotPanelHintKey", { fg = "#88bfc1" })
	vim.api.nvim_set_hl(0, "CopilotPanelModel", { fg = "#00172E", bg = "#028391", bold = true })
	vim.api.nvim_set_hl(0, "CopilotPanelCtx", { fg = "#00172E", bg = "#e8d49a", bold = true })
	vim.api.nvim_set_hl(0, "CopilotPanelWinbar", { fg = "#6dd6d1", bg = "#012b41", bold = true })
	vim.api.nvim_set_hl(0, "CopilotPanelUser", { fg = "#e8d49a", bold = true })
	vim.api.nvim_set_hl(0, "CopilotPanelAssistant", { fg = "#6dd6d1", bold = true })
	vim.api.nvim_set_hl(0, "CopilotPanelSeparator", { fg = "#287687" })
	vim.api.nvim_set_hl(0, "CopilotPanelResponse", { fg = "#b7d2d5" })
end

local function render_placeholder()
	if not input_buf or not vim.api.nvim_buf_is_valid(input_buf) then
		return
	end
	vim.api.nvim_buf_clear_namespace(input_buf, placeholder_ns, 0, -1)
	local prompt = table.concat(vim.api.nvim_buf_get_lines(input_buf, 0, PROMPT_LINES, false), "")
	if vim.trim(prompt) ~= "" or vim.api.nvim_get_mode().mode:find("i") then
		return
	end
	vim.api.nvim_buf_set_extmark(input_buf, placeholder_ns, 0, 0, {
		virt_text = { { " Ask Copilot    i insert    <CR> send", "CopilotPanelHint" } },
		virt_text_pos = "overlay",
		hl_mode = "combine",
	})
end

local function render_header()
	if not input_buf or not vim.api.nvim_buf_is_valid(input_buf) then
		return
	end
	local badge = " " .. state.model .. " "
	local ctx = string.format(" %d ctx ", #state.resources)
	local hints = {
		{ { "⌥a", "buf" }, { "⌥f", "file" }, { "⌥d", "diag" }, { "⌥m", "model" }, { "⌥s", "prompt" } },
		{ { "⌥n", "new" }, { "⌥h", "hist" }, { "⌥g", "diff" }, { "⌥q", "close" } },
	}

	local virt_lines = {
		{
			{ " ", "CopilotPanelHint" },
			{ badge, "CopilotPanelModel" },
			{ " ", "CopilotPanelHint" },
			{ ctx, "CopilotPanelCtx" },
		},
	}

	for _, row in ipairs(hints) do
		local chunks = { { " ", "CopilotPanelHint" } }
		for _, hint in ipairs(row) do
			table.insert(chunks, { hint[1], "CopilotPanelHintKey" })
			table.insert(chunks, { " " .. hint[2] .. "  ", "CopilotPanelHint" })
		end
		table.insert(virt_lines, chunks)
	end

	vim.api.nvim_buf_clear_namespace(input_buf, namespace, 0, -1)
	vim.api.nvim_buf_set_extmark(input_buf, namespace, PROMPT_LINES - 1, 0, {
		virt_lines = virt_lines,
	})
end

local function render()
	if not conversation_buf or not vim.api.nvim_buf_is_valid(conversation_buf) then
		return
	end

	local lines = {}
	local highlights = {}

	if #state.resources > 0 then
		table.insert(lines, " Attached context:")
		table.insert(highlights, { #lines - 1, "CopilotPanelLabel", 0, -1 })
		for _, resource in ipairs(state.resources) do
			table.insert(lines, "  " .. resource_label(resource))
			table.insert(highlights, { #lines - 1, "CopilotPanelHint", 0, -1 })
		end
		table.insert(lines, "────────────────────────────────────────────────────────────────")
		table.insert(highlights, { #lines - 1, "CopilotPanelSeparator", 0, -1 })
	end

	for _, message in ipairs(state.history) do
		table.insert(lines, message.role == "user" and " YOU" or " COPILOT")
		table.insert(highlights, {
			#lines - 1,
			message.role == "user" and "CopilotPanelUser" or "CopilotPanelAssistant",
			0,
			-1,
		})
		for _, line in ipairs(vim.split(message.content, "\n", { plain = true })) do
			table.insert(lines, " " .. line)
			table.insert(highlights, { #lines - 1, "CopilotPanelResponse", 0, -1 })
		end
		table.insert(lines, "────────────────────────────────────────────────────────────────")
		table.insert(highlights, { #lines - 1, "CopilotPanelSeparator", 0, -1 })
	end

	if streaming_response ~= "" then
		table.insert(lines, " COPILOT (thinking)")
		table.insert(highlights, { #lines - 1, "CopilotPanelAssistant", 0, -1 })
		for _, line in ipairs(vim.split(streaming_response, "\n", { plain = true })) do
			table.insert(lines, " " .. line)
			table.insert(highlights, { #lines - 1, "CopilotPanelResponse", 0, -1 })
		end
	end

	if #lines == 0 then
		table.insert(lines, " Start a conversation below. Attach context from the fixed header when needed.")
		table.insert(highlights, { 0, "CopilotPanelHint", 0, -1 })
	end

	vim.bo[conversation_buf].modifiable = true
	vim.api.nvim_buf_set_lines(conversation_buf, 0, -1, false, lines)
	vim.bo[conversation_buf].modifiable = false
	vim.api.nvim_buf_clear_namespace(conversation_buf, namespace, 0, -1)
	for _, highlight in ipairs(highlights) do
		vim.api.nvim_buf_add_highlight(conversation_buf, namespace, highlight[2], highlight[1], highlight[3], highlight[4])
	end
	render_header()
	if is_valid(conversation_win) then
		vim.api.nvim_win_set_cursor(conversation_win, { #lines, 0 })
	end
end

local function persist_history()
	local directory = vim.fs.dirname(history_path)
	vim.fn.mkdir(directory, "p")
	local file, err = io.open(history_path, "w")
	if not file then
		notify("Could not save chat history: " .. err, vim.log.levels.ERROR)
		return
	end
	file:write(vim.json.encode({
		history = state.history,
		model = state.model,
		system_prompt = state.system_prompt,
	}))
	file:close()
end

local function archive_current_chat()
	if #state.history == 0 then
		return
	end
	vim.fn.mkdir(archive_path, "p")
	local filename = archive_path .. "/" .. os.date("%Y%m%d-%H%M%S") .. ".json"
	local file, err = io.open(filename, "w")
	if not file then
		notify("Could not archive chat history: " .. err, vim.log.levels.ERROR)
		return
	end
	file:write(vim.json.encode({
		history = state.history,
		model = state.model,
		system_prompt = state.system_prompt,
	}))
	file:close()
end

local function load_history()
	local file = io.open(history_path, "r")
	if not file then
		return
	end
	local content = file:read("*a")
	file:close()
	local ok, saved = pcall(vim.json.decode, content)
	if not ok or type(saved) ~= "table" then
		notify("Could not read saved chat history.", vim.log.levels.WARN)
		return
	end
	if type(saved.history) == "table" then
		state.history = saved.history
	end
	if type(saved.model) == "string" then
		state.model = saved.model
	end
	if type(saved.system_prompt) == "string" then
		state.system_prompt = saved.system_prompt
	end
end

local function add_resource(resource)
	for _, existing in ipairs(state.resources) do
		if existing.kind == resource.kind and existing.name == resource.name then
			notify(resource.name .. " is already attached.", vim.log.levels.WARN)
			return
		end
	end
	table.insert(state.resources, resource)
	render()
end

local function buffer_resource(bufnr)
	bufnr = bufnr or source_buf
	if not bufnr or not vim.api.nvim_buf_is_valid(bufnr) then
		notify("No source buffer is available.", vim.log.levels.WARN)
		return
	end
	local name = vim.api.nvim_buf_get_name(bufnr)
	if name == "" then
		name = "[No Name]"
	end
	add_resource({
		kind = "buffer",
		name = name,
		data = table.concat(vim.api.nvim_buf_get_lines(bufnr, 0, -1, false), "\n"),
		mimetype = vim.bo[bufnr].filetype,
		uri = name,
	})
end

local function diagnostics_resource()
	if not source_buf or not vim.api.nvim_buf_is_valid(source_buf) then
		notify("No source buffer is available.", vim.log.levels.WARN)
		return
	end
	local diagnostics = vim.diagnostic.get(source_buf)
	if #diagnostics == 0 then
		notify("The source buffer has no diagnostics.", vim.log.levels.INFO)
		return
	end
	local lines = {}
	for _, diagnostic in ipairs(diagnostics) do
		table.insert(lines, string.format("L%d:C%d [%s] %s", diagnostic.lnum + 1, diagnostic.col + 1, vim.diagnostic.severity[diagnostic.severity], diagnostic.message))
	end
	add_resource({
		kind = "diagnostics",
		name = vim.api.nvim_buf_get_name(source_buf),
		data = table.concat(lines, "\n"),
		mimetype = "text",
		uri = "diagnostics",
	})
end

local function selection_resource(selection)
	if not selection or selection.start_line > selection.end_line then
		notify("No visual selection is available.", vim.log.levels.WARN)
		return
	end
	local lines = vim.api.nvim_buf_get_lines(selection.bufnr, selection.start_line - 1, selection.end_line, false)
	lines[1] = lines[1]:sub(selection.start_col + 1)
	lines[#lines] = lines[#lines]:sub(1, selection.end_col)
	add_resource({
		kind = "selection",
		name = vim.api.nvim_buf_get_name(selection.bufnr) .. ":" .. selection.start_line .. "-" .. selection.end_line,
		data = table.concat(lines, "\n"),
		mimetype = vim.bo[selection.bufnr].filetype,
		uri = vim.api.nvim_buf_get_name(selection.bufnr),
	})
end

local function submit()
	local prompt = vim.trim(table.concat(vim.api.nvim_buf_get_lines(input_buf, 0, PROMPT_LINES, false), "\n"))
	if prompt == "" then
		return
	end

	vim.api.nvim_buf_set_lines(input_buf, 0, PROMPT_LINES, false, { "", "", "" })
	render_placeholder()
	table.insert(state.history, { role = "user", content = prompt })
	streaming_response = ""
	render()

	require("plenary.async").run(function()
		local ok, response = xpcall(function()
			return require("CopilotChat.client"):ask({
				headless = false,
				history = state.history,
				resources = state.resources,
				tools = {},
				system_prompt = state.system_prompt,
				model = state.model,
				temperature = M.opts.temperature,
				on_progress = function(message)
					streaming_response = streaming_response .. message.content
					vim.schedule(render)
				end,
			})
		end, debug.traceback)

		vim.schedule(function()
			streaming_response = ""
			if not ok then
				notify(response, vim.log.levels.ERROR)
				render()
				return
			end
			table.insert(state.history, response.message)
			persist_history()
			render()
		end)
	end)
end

local function select_model()
	require("plenary.async").run(function()
		local ok, models = xpcall(function()
			return require("CopilotChat.client"):models()
		end, debug.traceback)
		vim.schedule(function()
			if not ok then
				notify(models, vim.log.levels.ERROR)
				return
			end
			local choices = vim.tbl_values(models)
			table.sort(choices, function(left, right)
				return left.name < right.name
			end)
			vim.ui.select(choices, {
				prompt = "Copilot model",
				format_item = function(item)
					return item.name .. (item.reasoning and " [reasoning]" or "")
				end,
			}, function(choice)
				if choice then
					state.model = choice.id
					render()
				end
			end)
		end)
	end)
end

local function set_system_prompt()
	vim.ui.input({ prompt = "System prompt: ", default = state.system_prompt }, function(value)
		if value and vim.trim(value) ~= "" then
			state.system_prompt = value
			persist_history()
			notify("System prompt updated.")
		end
	end)
end

local function attach_file(path)
	if not path or vim.trim(path) == "" then
		return
	end
	local resolved = vim.fs.normalize(path)
	if not vim.startswith(resolved, vim.fs.normalize(vim.fn.getcwd())) then
		resolved = vim.fs.joinpath(vim.fn.getcwd(), path)
	end
	local file, err = io.open(resolved, "r")
	if not file then
		notify("Could not read " .. resolved .. ": " .. err, vim.log.levels.ERROR)
		return
	end
	local content = file:read("*a")
	file:close()
	add_resource({
		kind = "file",
		name = resolved,
		data = content,
		mimetype = vim.filetype.match({ filename = resolved }) or "text",
		uri = resolved,
	})
end

local function add_file()
	local ok_fzf, fzf = pcall(require, "fzf-lua")
	if ok_fzf then
		fzf.files({
			prompt = "Attach to Copilot> ",
			cwd = vim.fn.getcwd(),
			actions = {
				["default"] = function(selected)
					for _, entry in ipairs(selected or {}) do
						attach_file(fzf.path.entry_to_file(entry).path)
					end
				end,
			},
		})
		return
	end

	local ok_snacks, snacks = pcall(require, "snacks")
	if ok_snacks and snacks.picker then
		snacks.picker.files({
			cwd = vim.fn.getcwd(),
			title = "Attach to Copilot",
			confirm = function(picker, item)
				picker:close()
				if item then
					attach_file(item._path or item.file)
				end
			end,
		})
		return
	end

	vim.ui.input({ prompt = "Project file (relative to cwd): " }, attach_file)
end

local function apply_diff()
	local response = state.history[#state.history]
	if not response or response.role ~= "assistant" then
		notify("There is no Copilot response with a diff to apply.", vim.log.levels.WARN)
		return
	end
	local diff = response.content:match("```diff%s*\n(.-)\n```") or response.content:match("```patch%s*\n(.-)\n```")
	if not diff then
		notify("The latest response does not contain a fenced diff block.", vim.log.levels.WARN)
		return
	end
	vim.ui.select({ "Apply", "Cancel" }, { prompt = "Apply the generated patch?" }, function(choice)
		if choice ~= "Apply" then
			return
		end
		local result = vim.system({ "git", "apply", "--check", "-" }, { cwd = vim.fn.getcwd(), stdin = diff, text = true }):wait()
		if result.code ~= 0 then
			notify("Patch validation failed:\n" .. result.stderr, vim.log.levels.ERROR)
			return
		end
		result = vim.system({ "git", "apply", "-" }, { cwd = vim.fn.getcwd(), stdin = diff, text = true }):wait()
		if result.code ~= 0 then
			notify("Patch could not be applied:\n" .. result.stderr, vim.log.levels.ERROR)
			return
		end
		notify("Patch applied.")
		vim.cmd("checktime")
	end)
end

local function configure_buffer_keymaps()
	local actions = {
		{ key = "a", action = buffer_resource, desc = "Attach current buffer" },
		{ key = "f", action = add_file, desc = "Attach project file" },
		{ key = "d", action = diagnostics_resource, desc = "Attach diagnostics" },
		{ key = "m", action = select_model, desc = "Select Copilot model" },
		{ key = "s", action = set_system_prompt, desc = "Edit system prompt" },
		{ key = "n", action = M.new_chat, desc = "New Copilot chat" },
		{ key = "h", action = M.select_history, desc = "Load chat history" },
		{ key = "g", action = apply_diff, desc = "Apply latest diff" },
		{ key = "q", action = M.close, desc = "Close Copilot panel" },
	}

	for _, entry in ipairs(actions) do
		-- Plain letters stay available in the read-only conversation buffer.
		vim.keymap.set("n", entry.key, entry.action, {
			buffer = conversation_buf,
			silent = true,
			nowait = true,
			desc = entry.desc,
		})
		-- The prompt keeps native editing, so actions move to Alt combinations.
		vim.keymap.set({ "n", "i" }, "<M-" .. entry.key .. ">", entry.action, {
			buffer = input_buf,
			silent = true,
			desc = entry.desc,
		})
	end

	vim.keymap.set("n", "<CR>", submit, { buffer = input_buf, silent = true, desc = "Send Copilot prompt" })
	vim.keymap.set("i", "<C-s>", submit, { buffer = input_buf, silent = true, desc = "Send Copilot prompt" })
	vim.keymap.set("i", "<C-c>", function()
		vim.cmd("stopinsert")
	end, { buffer = input_buf, silent = true, desc = "Leave Copilot prompt" })

	vim.api.nvim_create_autocmd({ "InsertEnter", "InsertLeave" }, {
		buffer = input_buf,
		callback = vim.schedule_wrap(render_placeholder),
	})

	vim.api.nvim_buf_attach(input_buf, false, {
		on_lines = function()
			vim.schedule(function()
				if not input_buf or not vim.api.nvim_buf_is_valid(input_buf) then
					return
				end
				if vim.api.nvim_buf_line_count(input_buf) > PROMPT_LINES then
					vim.api.nvim_buf_set_lines(input_buf, PROMPT_LINES, -1, false, {})
				end
				render_placeholder()
			end)
		end,
	})
end

function M.open()
	if is_open() then
		vim.api.nvim_set_current_win(input_win)
		return
	end

	source_win = vim.api.nvim_get_current_win()
	source_buf = vim.api.nvim_get_current_buf()
	local width = math.max(45, math.floor(vim.o.columns * 0.38))
	vim.cmd("botright vertical " .. width .. "new")
	conversation_win = vim.api.nvim_get_current_win()
	conversation_buf = vim.api.nvim_get_current_buf()
	vim.cmd("belowright 6new")
	input_win = vim.api.nvim_get_current_win()
	input_buf = vim.api.nvim_get_current_buf()

	vim.bo[conversation_buf].buftype = "nofile"
	vim.bo[conversation_buf].bufhidden = "wipe"
	vim.bo[conversation_buf].swapfile = false
	vim.bo[conversation_buf].filetype = "copilot-panel"
	vim.bo[conversation_buf].modifiable = false
	vim.bo[input_buf].buftype = "nofile"
	vim.bo[input_buf].bufhidden = "wipe"
	vim.bo[input_buf].swapfile = false
	vim.bo[input_buf].filetype = "copilot-panel"
	vim.api.nvim_buf_set_lines(input_buf, 0, -1, false, { "", "", "" })
	vim.api.nvim_buf_set_name(conversation_buf, "copilot-panel://conversation")
	vim.api.nvim_buf_set_name(input_buf, "copilot-panel://input")
	vim.wo[conversation_win].number = false
	vim.wo[conversation_win].relativenumber = false
	vim.wo[conversation_win].winbar = "%#CopilotPanelWinbar# Copilot%="
	vim.wo[conversation_win].winhighlight = "WinBar:CopilotPanelWinbar,WinBarNC:CopilotPanelWinbar"
	vim.wo[input_win].number = false
	vim.wo[input_win].relativenumber = false
	vim.wo[input_win].winfixheight = true
	vim.wo[conversation_win].winfixwidth = true
	vim.wo[input_win].winfixwidth = true
	vim.api.nvim_win_set_height(input_win, 6)
	configure_buffer_keymaps()
	render()
	render_placeholder()
	vim.api.nvim_set_current_win(input_win)
	vim.api.nvim_win_set_cursor(input_win, { 1, 0 })
end

function M.close()
	for _, win in ipairs({ conversation_win, input_win }) do
		if is_valid(win) then
			vim.api.nvim_win_close(win, true)
		end
	end
	conversation_win, input_win, conversation_buf, input_buf = nil, nil, nil, nil
	if is_valid(source_win) then
		vim.api.nvim_set_current_win(source_win)
	end
end

function M.toggle()
	if is_open() then
		M.close()
	else
		M.open()
	end
end

function M.open_with_selection()
	local start_pos = vim.fn.getpos("'<")
	local end_pos = vim.fn.getpos("'>")
	local selection = {
		bufnr = vim.api.nvim_get_current_buf(),
		start_line = start_pos[2],
		start_col = start_pos[3] - 1,
		end_line = end_pos[2],
		end_col = end_pos[3],
	}
	M.open()
	selection_resource(selection)
end

function M.new_chat()
	archive_current_chat()
	state.history = {}
	state.resources = {}
	streaming_response = ""
	persist_history()
	render()
	if is_valid(input_win) then
		vim.api.nvim_set_current_win(input_win)
		vim.api.nvim_win_set_cursor(input_win, { 1, 0 })
		render_placeholder()
	end
end

function M.select_history()
	local files = vim.fn.globpath(archive_path, "*.json", false, true)
	if #files == 0 then
		notify("No archived chats are available.", vim.log.levels.INFO)
		return
	end
	table.sort(files, function(left, right)
		return left > right
	end)
	vim.ui.select(files, {
		prompt = "Load archived chat",
		format_item = function(path)
			return vim.fs.basename(path):gsub("%.json$", "")
		end,
	}, function(choice)
		if not choice then
			return
		end
		local file, err = io.open(choice, "r")
		if not file then
			notify("Could not load archived chat: " .. err, vim.log.levels.ERROR)
			return
		end
		local ok, saved = pcall(vim.json.decode, file:read("*a"))
		file:close()
		if not ok or type(saved) ~= "table" or type(saved.history) ~= "table" then
			notify("The selected chat history is invalid.", vim.log.levels.ERROR)
			return
		end
		state.history = saved.history
		state.resources = {}
		state.model = type(saved.model) == "string" and saved.model or state.model
		state.system_prompt = type(saved.system_prompt) == "string" and saved.system_prompt or state.system_prompt
		persist_history()
		render()
	end)
end

function M.setup(opts)
	M.opts = opts
	set_highlights()
	load_history()
	vim.api.nvim_create_autocmd("ColorScheme", {
		callback = set_highlights,
	})
	vim.api.nvim_create_user_command("CopilotPanelToggle", M.toggle, { desc = "Toggle custom Copilot panel" })
	vim.api.nvim_create_user_command("CopilotPanelNew", M.new_chat, { desc = "Start a new Copilot chat" })
	vim.api.nvim_create_user_command("CopilotPanelHistory", M.select_history, { desc = "Manage Copilot chat history" })
	vim.api.nvim_create_user_command("CopilotPanelAddFile", add_file, { desc = "Attach a project file to Copilot" })
	vim.api.nvim_create_user_command("CopilotPanelAddDiagnostics", diagnostics_resource, { desc = "Attach diagnostics to Copilot" })
	vim.api.nvim_create_user_command("CopilotPanelModel", select_model, { desc = "Select Copilot model" })
	vim.api.nvim_create_user_command("CopilotPanelSystemPrompt", set_system_prompt, { desc = "Edit Copilot system prompt" })
	vim.api.nvim_create_user_command("CopilotPanelApplyDiff", apply_diff, { desc = "Apply latest Copilot diff" })
end

return M
