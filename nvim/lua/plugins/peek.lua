return {
	"MeanderingProgrammer/render-markdown.nvim",
	-- Automatically loads whenever you open a markdown file
	ft = { "markdown" },
	dependencies = {
		"nvim-treesitter/nvim-treesitter",
		"nvim-mini/mini.icons", -- Optional, but gives nice icons
	},
	opts = {
		-- High-performance default layouts
		heading = {
			sign = true,
			icons = { "   ", "   ", "   ", "   ", "   ", "   " },
		},
		checkbox = {
			enabled = true,
			unchecked = { icon = "   " },
			checked = { icon = " " },
		},
		pipe_table = {
			preset = "round", -- Draws clean visual boundaries over text tables
		},
	},
}
