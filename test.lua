#!/usr/bin/env lua

-- colors
local ERROR = "\27[38;5;196m"
local SUCCESS = "\27[38;5;46m"
local BOLD = "\27[1m"
local RESET = "\27[0m"

-- Resolve the oly binary once (avoid `cargo run` overhead per call)
local OLY_BIN do
	local candidates = { "target/debug/oly", "target/release/oly" }
	for _, p in ipairs(candidates) do
		local f = io.open(p, "r")
		if f then
			f:close()
			OLY_BIN = p
			break
		end
	end
	OLY_BIN = OLY_BIN or "cargo" -- fallback (will use `cargo run`)
end

---Runs an oly command
---@param cmd string[]
---@param opts {silent: boolean?}?
---@return {success: boolean, lines: string[]?}?
local function oly(cmd, opts)
	local uv = require("luv")

	opts = opts or {}

	local stdout = uv.new_pipe(false)
	local stderr = uv.new_pipe(false)

	local output = {}
	local exit_code = nil

	local bin, args
	if OLY_BIN == "cargo" then
		bin = "cargo"
		args = { "run", "--quiet", "--" }
		for _, arg in ipairs(cmd) do
			table.insert(args, arg)
		end
	else
		bin = OLY_BIN
		args = cmd
	end

	local handle
	handle = uv.spawn(bin, {
		args = args,
		stdio = { nil, stdout, stderr },
	}, function(code, signal)
		exit_code = code

		stdout:read_stop()
		stderr:read_stop()
		stdout:close()
		stderr:close()
		handle:close()
	end)

	if not handle then
		io.write("uv.spawn failed !")
		return nil
	end

	if not opts.silent then
		stdout:read_start(function(err, data)
			assert(not err, err)
			if data then table.insert(output, data) end
		end)
	else
		stdout:read_start(function() end)
	end

	stderr:read_start(function(err, data)
		assert(not err, err)
	end)

	uv.run()

	local lines = {}
	if not opts.silent then
		local full_output = table.concat(output)
		for line in full_output:gmatch("[^\n]+") do
			table.insert(lines, line)
		end
	end

	return {
		success = exit_code == 0,
		lines = opts.silent and nil or lines,
	}
end

---@param lines string[]
local function list_failed(lines)
	local first = true
	for _, pb in ipairs(lines) do
		if first then
			first = false
		else
			io.write(", ")
		end
		io.write(ERROR .. pb .. RESET)
	end
end

-- tests
local tests = {
	{
		name = "problem retrieval",
		test = function()
			local failed = {}
			for _, lang in ipairs({ "latex", "typst" }) do
				local handle = oly({ "list", "--filter-lang", "--lang", lang })
				if not handle then return 2 end
				if #handle.lines == 0 then goto continue end
				-- batch: one `show` for all problems (show supports multiple args)
				-- keeps per-process overhead O(1) instead of O(N) cargo spawns
				local batch = { "show", "--color", "never", "--lang", lang }
				for _, pb in ipairs(handle.lines) do
					table.insert(batch, pb)
				end
				local ok = oly(batch, { silent = true })
				if ok and ok.success then goto continue end
				-- batch failed (or binary missing) → fall back to per-problem to isolate failures
				for _, pb in ipairs(handle.lines) do
					local show = oly({ "show", pb, "--lang", lang, "--color", "never" }, { silent = true })
					if not show or not show.success then
						table.insert(failed, pb)
					end
				end
				::continue::
			end

			if #failed > 0 then
				list_failed(failed)
				io.write(" failed !\n")
				return 1
			end
		end,
	},
	{
		name = "problem naming",
		test = function()
			local handle = oly({ "list", "--test" })
			if not handle then return 2 end
			if #handle.lines > 0 then
				io.write("Problem names not matching their source:\n")
				list_failed(handle.lines)
				io.write("\n")
				return 1
			end
		end,
	},
} --[[@as table<{name: string, test: fun():nil|integer}>]]

-- run tests
for i, test in ipairs(tests) do
	print(BOLD .. "Testing " .. test.name .. "..." .. RESET)
	if test.test() == nil then
		print(SUCCESS .. "All good !" .. RESET)
	end

	if i ~= #tests then
		print()
	end
end
