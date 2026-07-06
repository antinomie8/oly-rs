-- run as `nvim --headless -c "luafile get_nvim_colors.lua"`
local groupe_names = {
	"punctuation.special",
	"punctuation.delimiter",
	"punctuation.bracket",
	"operator",
	"keyword.import",
	"keyword",
	"keyword.repeat",
	"keyword.conditional",
	"number",
	"string",
	"boolean",
	"constant",
	"variable.member",
	"function.call",
	"markup.heading.1",
	"markup.heading.2",
	"markup.heading.3",
	"markup.heading.4",
	"markup.heading.5",
	"markup.heading.6",
	"markup.strong",
	"markup.italic",
	"markup.link.url",
	"markup.raw",
	"label",
	"markup.raw.block",
	"markup.link.label",
	"markup.link",
	"markup.math",
}

local str = "colorscheme:\n"
local M = {}

for _, group in ipairs(groupe_names) do
	local hl = vim.api.nvim_get_hl(0, { name = "@" .. group, link = false })
	local fg = hl.fg
	local hex = fg and string.format("#%x", fg) or nil
	local bold = hl.bold and "bold" or nil
	local italic = hl.italic and "italic" or nil
	str = str .. string.rep(" ", 4) .. group .. ': "' .. (hex or bold or italic or "") .. '"\n'
end

print(str)
vim.cmd.quit()
