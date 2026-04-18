local dap = require("dap")

dap.adapters.codelldb = {
  type = "server",
  port = "${port}",
  executable = {
    command = "codelldb",
    args = { "--port", "${port}" },
  },
}
dap.configurations.rust = {
	{
		name = "Lesson Manager",
		type = "codelldb",
		request = "launch",
		repl_lang = "rust",
		args = { "rofi", "courses" },
		program = function()
			return vim.fn.input("Path to executable: ", vim.fn.getcwd() .. "/", "file")
		end,
		cwd = "${workspaceFolder}",
		stopOnEntry = false,
	},
}
