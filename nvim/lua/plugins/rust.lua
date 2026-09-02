return {
	{
		"mason-org/mason.nvim",
		opts = function(_, opts)
			opts.ensure_installed = opts.ensure_installed or {}
			vim.list_extend(opts.ensure_installed, { "rust-analyzer", "codelldb" })
		end,
	},
	{
		"mrcjkb/rustaceanvim",
		opts = function(_, opts)
			opts = opts or {}
			if vim.fn.has("win32") == 1 then
				local mason = vim.fn.stdpath("data") .. "\\mason"
				local codelldb = mason .. "\\bin\\codelldb.cmd"
				local liblldb = mason .. "\\packages\\codelldb\\extension\\lldb\\bin\\liblldb.dll"

				opts.dap = {
					adapter = require("rustaceanvim.config").get_codelldb_adapter(codelldb, liblldb),
				}
			end

			return opts
		end,
		config = function(_, opts)
			vim.g.rustaceanvim = vim.tbl_deep_extend("keep", vim.g.rustaceanvim or {}, opts or {})

			if vim.fn.executable("rust-analyzer") == 0 then
				vim.notify(
					"rust-analyzer not found in PATH. Run :MasonInstall rust-analyzer or install it with rustup.",
					vim.log.levels.WARN,
					{ title = "rustaceanvim" }
				)
			end
		end,
	},
}
