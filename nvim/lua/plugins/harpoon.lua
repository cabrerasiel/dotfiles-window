return {
	"ThePrimeagen/harpoon",
	branch = "harpoon2",
	dependencies = { "nvim-lua/plenary.nvim" },
	config = function()
		local harpoon = require("harpoon")
		harpoon:setup()

		harpoon:extend({
			UI_CREATE = function(cx)
				vim.keymap.set("n", "<Esc>", function()
					vim.cmd("w") -- Guarda los cambios antes de cerrar
					harpoon.ui:toggle_quick_menu(harpoon:list())
				end, { buffer = cx.bufnr })
			end,
		})
		-- Atajo para agregar el archivo actual a la lista
		vim.keymap.set("n", "<leader>a", function()
			harpoon:list():add()
		end)
		vim.keymap.set("n", "<leader>r", function()
			harpoon:list():remove()
		end, { desc = "Quitar archivo actual de Harpoon" })

		-- Atajo para ver el menú flotante interactivo
		vim.keymap.set("n", "<C-e>", function()
			harpoon.ui:toggle_quick_menu(harpoon:list())
		end)

		-- Atajos directos para saltar a los primeros 4 archivos
		vim.keymap.set("n", "<leader>1", function()
			harpoon:list():select(1)
		end)
		vim.keymap.set("n", "<leader>2", function()
			harpoon:list():select(2)
		end)
		vim.keymap.set("n", "<leader>3", function()
			harpoon:list():select(3)
		end)
		vim.keymap.set("n", "<leader>4", function()
			harpoon:list():select(4)
		end)
	end,
}
