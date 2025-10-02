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
    pastel8 = { fg = COLORS.PASTEL_GOLD },
    pastel9 = { fg = COLORS.PASTEL_TEAL, italic = true },
    pastel10 = { fg = COLORS.PASTEL_MAGENTA, bold = true, italic = true },
    pastel11 = { fg = COLORS.PASTEL_SILVER },
    pastel12 = { fg = COLORS.PASTEL_ORANGE, bold = true },

    border = { fg = "#FFFFFF", bold = true },
}

function tick()
    -- helper functions
    local function span(text, style_name)
        return { text = text, style = STYLES[style_name] }
    end

    local function line(...)
        local spans = { ... }
        table.insert(spans, 1, span("|", "pastel2"))
        table.insert(spans, span("|", "pastel2"))
        return spans
    end

    -- regenerate info lines
    FETCH_LINES = {
        line(span("fps: ", "pastel1"), span(TUI.fps, "pastel2"), span(TUI.frame, "pastel2")),
        line(span("tps: ", "pastel1"), span(TUI.tps, "pastel2"), span(TUI.tick, "pastel2")),
        line(span("elapsed: ", "pastel1"), span(TUI.elapsed, "pastel2")),

        line(span("User: ", "pastel1"), span(FETCH.user, "pastel2")),
        line(span("Host: ", "pastel3"), span(FETCH.host, "pastel4")),
        line(span("Device: ", "pastel5"), span(FETCH.device, "pastel6")),
        line(span("BIOS: ", "pastel7"), span(FETCH.bios, "pastel8")),
        line(span("Uptime: ", "pastel9"), span(FETCH.uptime, "pastel10")),
        line(span("OS: ", "pastel1"), span(FETCH.os_n .. " " .. FETCH.os_v, "pastel11")),
        line(span("Kernel: ", "pastel2"), span(FETCH.kern, "pastel12")),
        line(span("Log: ", "pastel3"), span(FETCH.log_m, "pastel4")),
        line(span("Desktop Env: ", "pastel5"), span(FETCH.desk_e or "<none>", "pastel6")),
        line(span("Window Manager: ", "pastel7"), span(FETCH.win_m, "pastel8")),
        line(span("Compositor: ", "pastel9"), span(FETCH.comp, "pastel10")),
        line(span("Terminal: ", "pastel1"), span(FETCH.term, "pastel2")),
        line(span("Shell: ", "pastel3"), span(FETCH.shell, "pastel4")),
        line(span("Text Editor: ", "pastel5"), span(FETCH.text_e, "pastel6")),
        line(span("CPU: ", "pastel7"), span(FETCH.cpu_n .. " (" .. FETCH.cpu_c .. " cores)", "pastel8")),
        line(span("CPU Usage: ", "pastel9"), span(FETCH.cpu_u .. "%", "pastel10")),
        line(span("CPU Temp: ", "pastel1"), span(FETCH.cpu_t .. "°C", "pastel2")),
        line(span("RAM: ", "pastel3"), span(FETCH.ram.avail .. "/" .. FETCH.ram.total, "pastel4")),
        line(span("GPU: ", "pastel5"), span(FETCH.gpu_n, "pastel6")),
        line(span("GPU Freq: ", "pastel7"), span(FETCH.gpu_f .. "GHz", "pastel8")),
        line(span("GPU Temp: ", "pastel9"), span(FETCH.gpu_t .. "°C", "pastel10")),
        line(span("VRAM: ", "pastel1"), span(FETCH.vram.avail .. "/" .. FETCH.vram.total, "pastel2")),
    }

    -- disks
    for i = 1, #FETCH.disks do
        local d = FETCH.disks[i]
        table.insert(
                FETCH_LINES,
                line(
                        span("Disk" .. i .. ": ", "pastel1"),
                        span(d.name .. " @ " .. d.mnt .. " total " .. d.mem.total .. " free " .. d.mem.avail, "pastel2")
                )
        )
    end

    -- media
    if FETCH.media and #FETCH.media > 0 then
        for i = 1, #FETCH.media do
            local m = FETCH.media[i]
            local status = m.paused and "paused" or "playing"
            table.insert(
                    FETCH_LINES,
                    line(
                            span("Media: ", "pastel3"),
                            span(m.artist .. " - " .. m.song, "pastel4"),
                            span(" [" .. status .. "]", "pastel5"),
                            span(" " .. m.name, "pastel6")
                    )
            )
        end
    end
end
