local Export = {}

Export._queue = {}

function Export.export_stl(filename, object, circular_segments)
  table.insert(Export._queue, {
    format = "stl",
    filename = filename,
    object = object,
    circular_segments = circular_segments or 128,
  })
end

function Export.export_3mf(filename, object, config)
  config = config or {}
  table.insert(Export._queue, {
    format = "3mf",
    filename = filename,
    object = object,
    units = config.units or "mm",
    include_colors = config.color ~= false,
  })
end

function Export.get_queue()
  return Export._queue
end

function Export.clear()
  Export._queue = {}
end

function Export.serialize()
  local result = {}
  for i, exp in ipairs(Export._queue) do
    local entry = {}
    for k, v in pairs(exp) do
      if k == "object" then
        if v and v.serialize then
          entry[k] = v:serialize()
        else
          entry[k] = v
        end
      else
        entry[k] = v
      end
    end
    result[i] = entry
  end
  return result
end

export_stl = Export.export_stl
export_3mf = Export.export_3mf

return Export
