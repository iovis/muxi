local function table_merge(old, new)
  for k, v in pairs(new) do
    if (type(v) == "table") and (type(old[k] or false) == "table") then
      table_merge(old[k], new[k])
    else
      old[k] = v
    end
  end

  return old
end

return table_merge
