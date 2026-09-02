return {
  {
    "OldJobobo/retro-82.nvim",
    priority = 1000,
    config = function()
      -- Use the greenish teal accent for borders instead of the default orange,
      -- to keep the look consistent with the rest of the dotfiles (Zebar/WezTerm).
      vim.api.nvim_create_autocmd("ColorScheme", {
        pattern = "retro-82",
        callback = function()
          vim.api.nvim_set_hl(0, "FloatBorder", { fg = "#028391", bg = "#00172E" })
          vim.api.nvim_set_hl(0, "WinSeparator", { fg = "#028391", bg = "#00172E" })
        end,
      })
    end,
  },
  {
    "LazyVim/LazyVim",
    opts = {
      colorscheme = "retro-82",
    },
  },
}
