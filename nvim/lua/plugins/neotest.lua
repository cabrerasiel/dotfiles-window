return {
  {
    "nvim-neotest/neotest",
    dependencies = {
      "nvim-neotest/nvim-nio",
      "Issafalcon/neotest-dotnet",
    },
    opts = {
      adapters = { "neotest-dotnet" },
    },
  },
}
