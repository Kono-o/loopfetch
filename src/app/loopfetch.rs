use crate::fetch::Info;
use katatui::mlua::Lua;
use katatui::*;

enum LAYOUT {
   Horiz,
   Vert,
}

impl Default for LAYOUT {
   fn default() -> Self {
      LAYOUT::Horiz
   }
}

enum ORDER {
   InfoFirst,
   AsciFirst,
}

impl Default for ORDER {
   fn default() -> Self {
      ORDER::InfoFirst
   }
}

struct SETTINGS {
   fps: u32,
   tps: u32,
   rps: u32,
   layout: LAYOUT,
   order: ORDER,
}

impl Default for SETTINGS {
   fn default() -> Self {
      Self {
         fps: 60,
         tps: 30,
         rps: 5,
         layout: LAYOUT::default(),
         order: ORDER::default(),
      }
   }
}

#[derive(Debug)]
pub struct Word {
   pub text: String,
   pub style: Style,
}

type LINES = Vec<Vec<Word>>;

pub struct LoopFetch {
   info: Info,
   lines: LINES,
   lua: Lua,
   src: String,
   settings: SETTINGS,
   refreshing: bool,
}

impl App for LoopFetch {
   const APP_NAME: &'static str = "loopfetch";
   const CONFIG_FILE: &'static str = "cfg.lua";
   const DEFAULT_CONFIG_SRC: &'static str = include_str!("../../cfg_template.lua");

   fn init(gloop: &mut GLoop, src: String) -> AppOutput<Self> {
      let mut app = Self {
         info: Info::fetch(),
         lines: LINES::new(),
         lua: Lua::new(),
         src: "".to_string(),
         settings: SETTINGS::default(),
         refreshing: false,
      };
      app.reload(gloop, src);
      gloop.set_fps(app.settings.fps);
      gloop.set_tps(app.settings.tps);
      AppOutput::Ok(app)
   }

   fn reload(&mut self, gloop: &mut GLoop, cfg_src: String) -> AppOutput<()> {
      self.src = cfg_src;
      let _ = self.update();
      gloop.set_fps(self.settings.fps);
      gloop.set_tps(self.settings.tps);
      AppOutput::nil()
   }

   fn logic(&mut self, gloop: &mut GLoop, gstate: &mut GState, event: Option<Event>) {
      self.refreshing = if gloop.tick() % self.settings.rps == 0 {
         self.info.refresh();
         self.update();
         gstate.request_reload();
         true
      } else {
         false
      };
      match event {
         Some(Event::Key(k)) => {
            self.handle_key(gloop, gstate, k);
         }
         //Some(Event::Mouse(m)) => {
         //   self.handle_mouse(gloop, gstate, m);
         //}
         _ => {}
      }
   }

   fn render(&self, gloop: &GLoop, gstate: &GState, area: Rect, buf: &mut Buffer) {
      let dir = match self.settings.layout {
         LAYOUT::Horiz => Direction::Horizontal,
         LAYOUT::Vert => Direction::Vertical,
      };
      let (a, b) = match self.settings.order {
         ORDER::InfoFirst => (0, 1),
         ORDER::AsciFirst => (1, 0),
      };
      let layout = Layout::default()
         .direction(dir)
         .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
         .split(area);
      self.render_info_box(gloop, gstate, area, layout[a], buf);
      self.render_asci_box(gloop, gstate, area, layout[b], buf);
   }
}

impl LoopFetch {
   fn update(&mut self) {
      self.update_lua();
      self.exec_lua();
      self.parse_lua();
   }
   fn update_lua(&mut self) -> AppOutput<()> {
      let info_lua = match self.info.to_lua(&self.lua) {
         Ok(i) => i,
         Err(e) => return app_err!("failed to convert Info to lua {e}"),
      };

      match self.lua.globals().set("Info", info_lua) {
         Ok(_) => {}
         Err(e) => return app_err!("failed to set Info in lua {}", e),
      };
      AppOutput::nil()
   }

   fn exec_lua(&mut self) -> AppOutput<()> {
      match self.lua.load(&self.src).exec() {
         Err(e) => app_err!("failed to execute lua {}", e),
         _ => AppOutput::nil(),
      }
   }

   fn parse_lua(&mut self) {
      let default_settings = SETTINGS::default();
      let default_layout = LAYOUT::default();
      let default_order = ORDER::default();
      let default_style = Style::new();
      let default_lines = LINES::new();
      let globals = self.lua.globals();

      let settings = match globals.get::<mlua::Table>("SETTINGS") {
         Ok(table) => {
            let layout = match table.get::<Option<String>>("layout") {
               Ok(os) => match os {
                  Some(s) => {
                     let first = s.chars().nth(0).unwrap_or('h');
                     match first {
                        'h' | 'H' => LAYOUT::Horiz,
                        'v' | 'V' => LAYOUT::Vert,
                        _ => default_layout,
                     }
                  }
                  _ => default_layout,
               },
               _ => default_layout,
            };
            let order = match table.get::<mlua::Table>("order") {
               Ok(table) => {
                  let f = table.get::<String>(1).unwrap_or("info".into());
                  match f.chars().nth(0).unwrap_or('i') {
                     'i' | 'I' => ORDER::InfoFirst,
                     'a' | 'A' => ORDER::AsciFirst,
                     _ => default_order,
                     _ => default_order,
                  }
               }
               _ => default_order,
            };
            SETTINGS {
               fps: table.get("fps").unwrap_or(default_settings.fps),
               tps: table.get("tps").unwrap_or(default_settings.tps),
               rps: table.get("rps").unwrap_or(default_settings.rps),
               layout,
               order,
            }
         }
         _ => default_settings,
      };

      let lines: Vec<Vec<Word>> = match globals.get::<mlua::Table>("LINES") {
         Ok(lines_table) => {
            let mut result = Vec::new();

            for line_res in lines_table.sequence_values::<mlua::Table>() {
               let line_tbl = match line_res {
                  Ok(tbl) => tbl,
                  _ => continue,
               };
               let mut words = Vec::new();

               for span_res in line_tbl.sequence_values::<mlua::Table>() {
                  let span_tbl = match span_res {
                     Ok(tbl) => tbl,
                     _ => continue,
                  };

                  let text: String = span_tbl.get("text").unwrap_or_default();
                  let style_tbl: mlua::Table = match span_tbl.get("style") {
                     Ok(tbl) => tbl,
                     _ => continue,
                  };
                  let mut style = default_style;

                  if let Ok(Some(fg_hex)) = style_tbl.get::<Option<String>>("fg") {
                     if let Some((r, g, b)) = hex_to_rgb(&fg_hex) {
                        style = style.fg(Color::Rgb(r, g, b));
                     }
                  }

                  if let Ok(Some(bg_hex)) = style_tbl.get::<Option<String>>("bg") {
                     if let Some((r, g, b)) = hex_to_rgb(&bg_hex) {
                        style = style.bg(Color::Rgb(r, g, b));
                     }
                  }

                  if let Ok(true) = style_tbl.get("bold") {
                     style = style.add_modifier(Modifier::BOLD);
                  }

                  if let Ok(true) = style_tbl.get("italic") {
                     style = style.add_modifier(Modifier::ITALIC);
                  }

                  words.push(Word { text, style });
               }

               result.push(words);
            }

            result
         }
         _ => default_lines,
      };
      self.settings = settings;
      self.lines = lines;
   }

   fn render_info_box(
      &self,
      _gloop: &GLoop,
      _gstate: &GState,
      _area: Rect,
      layout: Rect,
      buf: &mut Buffer,
   ) {
      let mut text = Vec::<Line>::new();
      for line in &self.lines {
         let mut spans = Vec::<Span>::new();
         for word in line {
            spans.push(Span::styled(&word.text, word.style));
         }
         text.push(Line::from(spans));
      }
      Paragraph::new(Text::from(text))
         .block(Block::new())
         .render(layout, buf);
   }

   fn render_asci_box(
      &self,
      gloop: &GLoop,
      _gstate: &GState,
      _area: Rect,
      layout: Rect,
      buf: &mut Buffer,
   ) {
      let fps_line = Line::from(format!(
         "fps: {:06.2} {} [{:06}/{:06} ms] ({:02})",
         gloop.fps(),
         gloop.target_fps(),
         gloop.f_ms(),
         gloop.budget(),
         gloop.frame() % gloop.target_fps(),
      ));
      let tps_line = Line::from(format!(
         "tps: {:06.2} {} [{:06}/{:06} ms] ({:02})",
         gloop.tps(),
         gloop.target_tps(),
         gloop.t_ms(),
         gloop.budget(),
         gloop.tick() % gloop.target_tps(),
      ));
      let refreshed = if self.refreshing { "refreshed..." } else { "" };
      let rps_line = Line::from(format!("rps: {:02.2} {refreshed}", self.settings.rps));
      Paragraph::new(Text::from(vec![fps_line, tps_line, rps_line]))
         .block(Block::new())
         .render(layout, buf);
   }

   fn handle_key(&mut self, gloop: &mut GLoop, gstate: &mut GState, key_event: KeyEvent) {
      let fps = gloop.target_fps();
      let kind = key_event.kind;
      if kind == KeyEventKind::Press {
         match key_event.code {
            KeyCode::Char('q') => gstate.request_exit(),
            KeyCode::Char('r') => gstate.request_reload(),
            KeyCode::Left => gloop.set_fps(fps - 5),
            KeyCode::Right => gloop.set_fps(fps + 5),
            _ => {}
         }
      }
   }
}

pub fn hex_to_rgb(hex: &str) -> Option<(u8, u8, u8)> {
   let hex = hex.trim_start_matches('#');
   if hex.len() != 6 {
      return None;
   }
   let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255);
   let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
   let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
   Some((r, g, b))
}
