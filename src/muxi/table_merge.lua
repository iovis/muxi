function MuxiTableMerge(old, new)
  for k, v in pairs(new) do
    if (type(v) == "table") and (type(old[k] or false) == "table") then
      MuxiTableMerge(old[k], new[k])
    else
      old[k] = v
    end
  end

  return old
end
