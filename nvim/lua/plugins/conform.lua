return {
	{
		"stevearc/conform.nvim",
		event = { "BufWritePre" },
		cmd = { "ConformInfo" },
		opts = {
			formatters_by_ft = {
				typescript = { "prettier" },
				javascript = { "prettier" },
				html = { "prettier" },
				htmlangular = { "prettier" },
				css = { "prettier" },
				scss = { "prettier" },
				cs = { "csharpier" },
				rust = { "rustfmt" },
			},
		},
	},
	{
		"zapling/mason-conform.nvim",
		dependencies = { "mason-org/mason.nvim", "stevearc/conform.nvim" },
		config = function()
			require("mason-conform").setup({
				ensure_installed = { "prettier" },
			})
		end,
	},
}
