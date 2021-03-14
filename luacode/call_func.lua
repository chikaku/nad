local function max(...)
    local args = {...}
    local  val, idx
    for i = 1, #args do
        if val == nil or args[i] > val then
            val, idx = args[i], i
        end
    end
    return val, idx
end

local function assert(v)
    if not v then fail() end
end

local val, index = max(1, 3, 129, 256, 35)
assert(val == 256)
assert(index == 4)
