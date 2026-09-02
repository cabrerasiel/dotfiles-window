return {
	{
		"CopilotC-Nvim/CopilotChat.nvim",
		dependencies = { "nvim-lua/plenary.nvim" },
		keys = {
			{ "<leader>cp", function() require("config.copilot_panel").toggle() end, desc = "Copilot panel" },
			{
				"<leader>cp",
				function()
					require("config.copilot_panel").open_with_selection()
				end,
				mode = "v",
				desc = "Ask Copilot about selection",
			},
		},
		cmd = {
			"CopilotPanelToggle",
			"CopilotPanelNew",
			"CopilotPanelHistory",
			"CopilotPanelAddFile",
			"CopilotPanelAddDiagnostics",
			"CopilotPanelModel",
			"CopilotPanelSystemPrompt",
			"CopilotPanelApplyDiff",
		},
		opts = {
			model = "gpt-5-mini",
			temperature = 0.1,
			trusted_tools = nil,
		},
		config = function(_, opts)
			require("CopilotChat").setup(opts)
			require("config.copilot_panel").setup(opts)
		end,
	},
}
