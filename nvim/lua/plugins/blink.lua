return {
	{
		"saghen/blink.cmp",
		version = "*", -- Usa la última versión estable
		-- Opcional: añade iconos bonitos a las sugerencias
		dependencies = "rafamadriz/friendly-snippets",

		opts = {
			-- Configuración de teclas predeterminada
			-- 'default': Ctrl+space abre el menú, Enter confirma, Flechas/Ctrl+n/Ctrl+p navegan
			keymap = { preset = "default" },

			appearance = {
				use_nvim_cmp_as_default = true,
				nerd_font_variant = "mono",
			},

			-- Le dice a blink que use el LSP (Roslyn) como fuente de autocompletado
			sources = {
				default = { "lsp", "path", "snippets", "buffer" },
			},
		},
	},
}
