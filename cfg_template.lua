--========== LOOPFETCH SETTINGS ==========--
SETTINGS = {
    fps = 60, -- frames aka render calls (per second)
    tps = 30, -- ticks aka logical updates (per second)
    rps = 5, -- rate of fetch refreshes (per second)
    order = { "info", "ascii" }, -- order of boxes: info, ascii (a, i also works)
    layout = "horizontal", -- stacking of boxes: horizontal, vertical (or h, v)
    vars = { comp = "idk" }, -- hardcode unfetchables (e.g. comp = 'picom')
}

--========== LOOPFETCH COLORS ==========-- define ur own colors here
COLORS = {
    PASTEL_PINK = "#FFB6C1",
    PASTEL_GREEN = "#77DD77",
    PASTEL_BLUE = "#AEC6CF",
    PASTEL_ORANGE = "#FFCC99",
    PASTEL_CYAN = "#B2FFFF",
    PASTEL_LIME = "#CCFF99",
    PASTEL_PURPLE = "#CDA4DE",
    PASTEL_GOLD = "#FFD700",
    PASTEL_TEAL = "#99EEDD",
    PASTEL_MAGENTA = "#FF99CC",
    PASTEL_SILVER = "#C0C0C0",
    PASTEL_ORANGE2 = "#FFB347",
    PASTEL_YELLOW = "#FFFF99",
}

--========== LOOPFETCH STYLES ==========-- define ur own ansi styles using those colors
STYLES = {
    pastel1 = { fg = COLORS.PASTEL_PINK, bold = true, italic = true },
    pastel2 = { fg = COLORS.PASTEL_GREEN, bold = true },
    pastel3 = { fg = COLORS.PASTEL_BLUE, italic = true },
    pastel4 = { fg = COLORS.PASTEL_ORANGE, bold = true },
    pastel5 = { fg = COLORS.PASTEL_CYAN },
    pastel6 = { fg = COLORS.PASTEL_LIME, italic = true },
    pastel7 = { fg = COLORS.PASTEL_PURPLE, bold = true },
    pastel8 = { fg = COLORS.PASTEL_GOLD, },
    pastel9 = { fg = COLORS.PASTEL_TEAL, italic = true },
    pastel10 = { fg = COLORS.PASTEL_MAGENTA, bold = true, italic = true },
    pastel11 = { fg = COLORS.PASTEL_SILVER },
    pastel12 = { fg = COLORS.PASTEL_ORANGE, bold = true },

    border = { fg = "#FFFFFF", bold = true },
}

-- span helper --
function span(text, style_name)
    return { text = text, style = STYLES[style_name] }
end

-- line helper --
function line(border, ...)
    local spans = { ... }

    if border then
        -- find total content width
        local len = 0
        for _, sp in ipairs(spans) do
            len = len + #sp.text
        end
        -- wrap with │ on left/right
        local new_spans = { span("│", "border") }
        for _, sp in ipairs(spans) do
            table.insert(new_spans, sp)
        end
        table.insert(new_spans, span("│", "border"))
        spans = new_spans
    end

    return spans
end

--========== LOOPFETCH INFO LINES ==========--
LINES = {
    line(span("User: ", "pastel1"), span(Info.user, "pastel2")),
    line(span("Host: ", "pastel3"), span(Info.host, "pastel4")),
    line(span("Device: ", "pastel5"), span(Info.device, "pastel6")),
    line(span("BIOS: ", "pastel7"), span(Info.bios, "pastel8")),
    line(span("Uptime: ", "pastel9"), span(Info.uptime, "pastel10")),
    line(span("OS: ", "pastel1"), span(Info.os_n .. " " .. Info.os_v, "pastel11")),
    line(span("Kernel: ", "pastel2"), span(Info.kern, "pastel12")),
    line(span("Log: ", "pastel3"), span(Info.log_m, "pastel4")),
    line(span("Desktop Env: ", "pastel5"), span(Info.desk_e or "<none>", "pastel6")),
    line(span("Window Manager: ", "pastel7"), span(Info.win_m, "pastel8")),
    line(span("Compositor: ", "pastel9"), span(Info.comp, "pastel10")),
    line(span("Terminal: ", "pastel1"), span(Info.term, "pastel2")),
    line(span("Shell: ", "pastel3"), span(Info.shell, "pastel4")),
    line(span("Text Editor: ", "pastel5"), span(Info.text_e, "pastel6")),
    line(span("CPU: ", "pastel7"), span(Info.cpu_n .. " (" .. Info.cpu_c .. " cores)", "pastel8")),
    line(span("CPU Usage: ", "pastel9"), span(Info.cpu_u .. "%", "pastel10")),
    line(span("CPU Temp: ", "pastel1"), span(Info.cpu_t .. "°C", "pastel2")),
    line(span("RAM: ", "pastel3"), span(Info.ram.avail .. "/" .. Info.ram.total, "pastel4")),
    line(span("GPU: ", "pastel5"), span(Info.gpu_n, "pastel6")),
    line(span("GPU Freq: ", "pastel7"), span(Info.gpu_f .. "GHz", "pastel8")),
    line(span("GPU Temp: ", "pastel9"), span(Info.gpu_t .. "°C", "pastel10")),
    line(span("VRAM: ", "pastel1"), span(Info.vram.avail .. "/" .. Info.vram.total, "pastel2")),
}
-- disk lines --
for i = 1, #Info.disks do
    local d = Info.disks[i]
    table.insert(LINES, line(
            span("Disk" .. i .. ": ", "pastel1"),
            span(d.name .. "@" .. d.mnt .. " free " .. d.mem.avail, "pastel2")
    ))
end

-- media lines --
if Info.media and #Info.media > 0 then
    for i = 1, #Info.media do
        local m = Info.media[i]
        local status = m.paused and "paused" or "playing"
        table.insert(LINES, line(
                span("Media: ", "pastel3"),
                span(m.artist .. " - " .. m.song, "pastel4"),
                span(" [" .. status .. "]", "pastel5"),
                span(" on " .. m.name, "pastel6")
        ))
    end
end

local max_len = 0
for _, l in ipairs(LINES) do
    local len = 0
    for _, sp in ipairs(l) do
        len = len + #sp.text
    end
    if len > max_len then
        max_len = len
    end
end

-- insert top border
table.insert(LINES, 1, line(false, span("┌" .. string.rep("─", max_len) .. "┐", "border")))
