return {
	{
		"seblyng/roslyn.nvim",
		ft = { "cs", "razor" },
		dependencies = {
			{ "mason-org/mason.nvim" }, -- Required for external tool binaries
		},
		config = function()
			local capabilities = vim.lsp.protocol.make_client_capabilities()
			local has_cmp, cmp_lsp = pcall(require, "cmp_nvim_lsp")

			if has_cmp then
				capabilities = vim.tbl_deep_extend("force", capabilities, cmp_lsp.default_capabilities())
			end

			-- Intentamos jalar directamente las capacidades del motor Blink
			local ok, blink = pcall(require, "blink.cmp")
			if ok then
				capabilities = blink.get_lsp_capabilities(capabilities)
			else
				-- Si usas una versión muy reciente de blink, se extrae así:
				capabilities = require("blink.cmp").get_lsp_capabilities()
			end

			require("roslyn").setup({
				args = {
					"--logLevel=Information",
					"--extensionLogDirectory=" .. vim.fs.dirname(vim.lsp.get_log_path()),
				},
				config = {
					capabilities = capabilities,
					-- Standard LSP keymaps and capabilities
					on_attach = function(client, bufnr)
						local opts = { buffer = bufnr, silent = true, desc = "Go to C# Definition" }
						pcall(vim.keymap.del, "n", "gd", { buffer = bufnr })

						vim.keymap.set("n", "gd", vim.lsp.buf.definition, opts)
						vim.keymap.set("n", "gr", vim.lsp.buf.references, opts)
						vim.keymap.set("n", "K", vim.lsp.buf.hover, opts)
						vim.keymap.set("n", "<leader>ca", vim.lsp.buf.code_action, opts)
						vim.keymap.set("n", "<leader>rn", vim.lsp.buf.rename, opts)
					end,
					settings = {
						["csharp|completion"] = {
							dotnet_show_completion_items_from_unimported_namespaces = true, -- <-- ACTIVA AUTOIMPORTS
							dotnet_provide_regex_completions = true,
						},
					},
				},
			})
		end,
	},
}
