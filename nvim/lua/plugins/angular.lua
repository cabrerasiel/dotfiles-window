return {
	{
		"neovim/nvim-lspconfig",
		opts = {
			servers = {
				angularls = { formater = { enable = true } },
				-- Activamos el servidor pero dejamos la configuración vacía aquí
			},
		},
	},
}
