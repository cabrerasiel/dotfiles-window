-- Navegación de panes compartida entre psmux y nvim.
-- Usa las mismas teclas que psmux (Ctrl+Alt+flechas) para moverse entre
-- splits de nvim y, al llegar al borde, entre panes de psmux sin cortar.
return {
	"christoomey/vim-tmux-navigator",
	lazy = false,
	init = function()
		-- Evita que el plugin cree sus mappings por defecto (Ctrl+hjkl),
		-- definimos los nuestros abajo para no chocar con LazyVim.
		vim.g.tmux_navigator_no_mappings = 1
		vim.g.tmux_navigator_disable_when_zoomed = 1
	end,
	keys = {
		{ "<C-M-Left>", "<cmd>TmuxNavigateLeft<cr>", desc = "Ir al pane/split de la izquierda" },
		{ "<C-M-Right>", "<cmd>TmuxNavigateRight<cr>", desc = "Ir al pane/split de la derecha" },
		{ "<C-M-Up>", "<cmd>TmuxNavigateUp<cr>", desc = "Ir al pane/split de arriba" },
		{ "<C-M-Down>", "<cmd>TmuxNavigateDown<cr>", desc = "Ir al pane/split de abajo" },
	},
}
