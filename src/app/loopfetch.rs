use crate::fetch::Info;
use katatui::mlua::Lua;
use katatui::*;

#[derive(Default)]
enum LAYOUT {
   #[default]
   Horiz,
   Vert,
}

impl LAYOUT {
   fn swap(&mut self) {
      *self = match self {
         LAYOUT::Horiz => LAYOUT::Vert,
         _ => LAYOUT::Horiz,
      }
   }
}

#[derive(Default)]
enum ORDER {
   #[default]
   InfoFirst,
   AsciFirst,
}

impl ORDER {
   fn swap(&mut self) {
      *self = match self {
         ORDER::InfoFirst => ORDER::AsciFirst,
         _ => ORDER::InfoFirst,
      }
   }
}

pub struct VARS {
   comp: String,
}

impl Default for VARS {
   fn default() -> Self {
      Self {
         comp: "unknown".to_string(),
      }
   }
}

impl VARS {
   pub fn comp(&self) -> &str {
      &self.comp
   }
}

pub struct SETTINGS {
   fps: u32,
   tps: u32,
   rps: u32,
   layout: LAYOUT,
   order: ORDER,
   vars: VARS,
}

impl Default for SETTINGS {
   fn default() -> Self {
      Self {
         fps: 60,
         tps: 30,
         rps: 5,
         layout: LAYOUT::default(),
         order: ORDER::default(),
         vars: VARS::default(),
      }
   }
}

impl SETTINGS {
   pub fn vars(&self) -> &VARS {
      &self.vars
   }
}

#[derive(Debug, Clone)]
pub struct Word {
   pub text: String,
   pub style: Style,
}

#[derive(Debug, Default)]
pub struct InfoBox {
   lines: LINES,
   max_len: usize,
}
#[derive(Debug, Default)]
pub struct AsciBox {
   lines: LINES,
   max_len: usize,
}

type LINES = Vec<Vec<Word>>;

pub struct LoopFetch {
   info: Info,
   lua: Lua,
   src: String,
   settings: SETTINGS,
   info_box: InfoBox,
   asci_box: AsciBox,
   refreshing: bool,
   reloading: bool,
}

impl App for LoopFetch {
   const APP_NAME: &'static str = "loopfetch";
   const CONFIG_FILE: &'static str = "cfg.lua";
   const DEFAULT_CONFIG_SRC: &'static str = include_str!("../../cfg_template.lua");

   fn init(gloop: &mut GLoop, src: String) -> AppOutput<Self> {
      let settings = SETTINGS::default();
      let mut app = Self {
         info: Info::fetch(&settings),
         lua: Lua::new(),
         src: "".to_string(),
         settings,
         info_box: InfoBox::default(),
         asci_box: AsciBox::default(),
         refreshing: false,
         reloading: false,
      };
      app.reload(gloop, src);
      gloop.set_fps(app.settings.fps);
      gloop.set_tps(app.settings.tps);
      AppOutput::Ok(app)
   }

   fn reload(&mut self, gloop: &mut GLoop, cfg_src: String) -> AppOutput<()> {
      self.src = cfg_src;
      self.load_lua();
      gloop.set_fps(self.settings.fps);
      gloop.set_tps(self.settings.tps);
      AppOutput::nil()
   }

   fn logic(&mut self, gloop: &mut GLoop, gstate: &mut GState, event: Option<Event>) {
      self.refreshing = if gloop.tick() % self.settings.rps == 0 {
         self.info.refresh(&self.settings);
         self.update(gloop);
         true
      } else {
         false
      };
      self.reloading = if gloop.tick() % (self.settings.rps * 10) == 0 {
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
      let (a, b, a_w, b_w, a_h, b_h) = match self.settings.order {
         ORDER::InfoFirst => (
            0,
            1,
            self.info_box.max_len,
            self.asci_box.max_len,
            self.info_box.lines.len(),
            self.asci_box.lines.len(),
         ),
         ORDER::AsciFirst => (
            1,
            0,
            self.asci_box.max_len,
            self.info_box.max_len,
            self.asci_box.lines.len(),
            self.info_box.lines.len(),
         ),
      };

      let (dir, a_c, b_c, total_w, total_h) = match self.settings.layout {
         LAYOUT::Vert => (
            Direction::Vertical,
            a_h,
            b_h,
            self.info_box.max_len.max(self.asci_box.max_len),
            self.info_box.lines.len() + self.asci_box.lines.len(),
         ),
         LAYOUT::Horiz => (
            Direction::Horizontal,
            a_w,
            b_w,
            self.info_box.max_len + self.asci_box.max_len,
            self.info_box.lines.len().max(self.asci_box.lines.len()),
         ),
      };

      let gap_w = (buf.area.width as usize).saturating_sub(total_w) as u16 / 2;
      let gap_h = (buf.area.height as usize).saturating_sub(total_h) as u16 / 2;

      let lines = Layout::new(
         Direction::Vertical,
         [
            Constraint::Max(gap_h),
            Constraint::Min(0),
            Constraint::Max(gap_h),
         ],
      )
      .split(buf.area);
      let top_box = lines[0];
      let mid_line = Layout::new(
         Direction::Horizontal,
         [
            Constraint::Max(gap_w),
            Constraint::Fill(1),
            Constraint::Max(gap_w),
         ],
      )
      .split(lines[1]);
      let bot_box = lines[2];

      self.render_blank_box(top_box, buf);
      self.render_blank_box(bot_box, buf);
      self.render_blank_box(mid_line[0], buf);
      self.render_blank_box(mid_line[2], buf);

      let content = Layout::new(
         dir,
         [
            Constraint::Length(a_c as u16),
            Constraint::Length(b_c as u16),
         ],
      )
      .split(mid_line[1]);

      self.render_info_box(gloop, gstate, area, content[a], buf);
      self.render_asci_box(gloop, gstate, area, content[b], buf);
   }
}

impl LoopFetch {
   fn update(&mut self, gloop: &GLoop) {
      self.update_lua(gloop);
      self.run_lua();
      self.parse_lua();
   }
   fn update_lua(&mut self, gloop: &GLoop) -> AppOutput<()> {
      match self.lua.globals().get::<mlua::Table>("SETTINGS") {
         Ok(table) => {
            let order = match table.get::<mlua::Table>("order") {
               Err(e) => return app_err!("failed to get SETTINGS.order in lua {e}"),
               Ok(t) => t,
            };
            match self.settings.order {
               ORDER::InfoFirst => {
                  order.set(1, "info");
                  order.set(2, "ascii");
               }
               ORDER::AsciFirst => {
                  order.set(1, "ascii");
                  order.set(2, "info");
               }
            };
            match self.settings.layout {
               LAYOUT::Vert => {
                  table.set("layout", "vertical");
               }
               LAYOUT::Horiz => {
                  table.set("layout", "horizontal");
               }
            }
         }
         Err(e) => return app_err!("failed to get SETTINGS in lua {e}"),
      };

      let info_lua = match self.info.to_lua(&self.lua) {
         Ok(i) => i,
         Err(e) => return app_err!("failed to convert Info to lua {e}"),
      };
      let loop_lua = match gloop.to_lua(&self.lua) {
         Ok(i) => i,
         Err(e) => return app_err!("failed to convert Loop to lua {e}"),
      };

      match self.lua.globals().set("Info", info_lua) {
         Ok(_) => {}
         Err(e) => return app_err!("failed to set Info in lua {}", e),
      };
      match self.lua.globals().set("Loop", loop_lua) {
         Ok(_) => {}
         Err(e) => return app_err!("failed to set Loop in lua {}", e),
      };

      AppOutput::nil()
   }

   fn load_lua(&mut self) -> AppOutput<()> {
      match self.lua.load(&self.src).exec() {
         Err(e) => app_err!("failed to load lua {}", e),
         _ => AppOutput::nil(),
      }
   }

   fn run_lua(&mut self) -> AppOutput<()> {
      match self.lua.globals().get::<mlua::Function>("update") {
         Ok(f) => {
            f.call::<()>(());
            AppOutput::nil()
         }
         Err(e) => app_err!("failed to run lua {}", e),
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
            let default_comp = default_settings.vars.comp.clone();
            let vars = match table.get::<mlua::Table>("vars") {
               Ok(table) => VARS {
                  comp: table.get::<String>("comp").unwrap_or(default_comp),
               },
               _ => VARS { comp: default_comp },
            };
            SETTINGS {
               fps: table.get("fps").unwrap_or(default_settings.fps),
               tps: table.get("tps").unwrap_or(default_settings.tps),
               rps: table.get("rps").unwrap_or(default_settings.rps),
               layout,
               order,
               vars,
            }
         }
         _ => default_settings,
      };

      let lines: Vec<Vec<Word>> = match globals.get::<mlua::Table>("INFO_LINES") {
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
      self.info_box.max_len = lines
         .iter()
         .map(|words| words.iter().map(|w| w.text.len()).sum::<usize>())
         .max()
         .unwrap_or(0);
      self.info_box.lines = lines;
      self.asci_box.max_len = 30;
      self.asci_box.lines = vec![Vec::<Word>::default(); 10];
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
      for line in &self.info_box.lines {
         let mut spans = Vec::<Span>::new();
         for word in line {
            spans.push(Span::styled(&word.text, word.style));
         }
         text.push(Line::from(spans));
      }
      let block = Block::new();
      //.borders(Borders::ALL)
      //.border_type(BorderType::Rounded);
      Paragraph::new(Text::from(text))
         .block(block)
         .style(Style::default().bg(Color::LightRed))
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
      let reloaded = if self.reloading { "reloaded..." } else { "" };

      let rps_line = Line::from(format!("rps: {:02.2} {refreshed}", self.settings.rps));
      let area_line = Line::from(format!(
         "area: {} x {}  ({}) {reloaded}",
         layout.width, layout.height, self.info_box.max_len,
      ));

      let block = Block::new();
      //.borders(Borders::ALL)
      //.border_type(BorderType::Rounded);
      Paragraph::new(Text::from(vec![fps_line, tps_line, rps_line, area_line]))
         .block(block)
         .style(Style::default().bg(Color::LightBlue))
         .render(layout, buf);
   }

   fn render_blank_box(&self, layout: Rect, buf: &mut Buffer) {
      let block = Block::new();
      //.borders(Borders::ALL)
      //.border_type(BorderType::Rounded);
      Paragraph::new("")
         .block(block)
         .style(Style::default().bg(Color::LightGreen))
         .render(layout, buf);
   }

   fn handle_key(&mut self, gloop: &mut GLoop, gstate: &mut GState, key_event: KeyEvent) {
      //let fps = gloop.target_fps();
      let kind = key_event.kind;
      if kind == KeyEventKind::Press {
         match key_event.code {
            KeyCode::Char('q') => gstate.request_exit(),
            KeyCode::Char('r') => gstate.request_reload(),
            KeyCode::Up | KeyCode::Down => self.settings.layout.swap(),
            KeyCode::Left | KeyCode::Right => self.settings.order.swap(),
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
