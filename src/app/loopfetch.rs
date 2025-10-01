use crate::fetch::INFO;
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
         fps: 24,
         tps: 12,
         rps: 3,
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
   info: INFO,
   settings: SETTINGS,
   info_box: InfoBox,
   asci_box: AsciBox,
}

impl App for LoopFetch {
   const APP_NAME: &'static str = "loopfetch";
   const CONFIG_FILE: &'static str = "init.lua";
   const DEFAULT_CONFIG_SRC: &'static str = include_str!("../../init_template.lua");

   fn init(mut tui: TUIMutRef) -> AppOutput<Self> {
      let settings = SETTINGS::default();
      let mut app = Self {
         info: INFO::fetch(&settings),
         settings,
         info_box: InfoBox::default(),
         asci_box: AsciBox::default(),
      };
      app.read_cfg(&tui);
      app.update_tui_settings(&mut tui);
      AppOutput::Ok(app)
   }

   fn logic(&mut self, mut tui: TUIMutRef, event: Option<Event>) {
      if tui.gloop.tick() % self.settings.rps == 0 {
         self.info.refresh(&self.settings);
      };
      if !tui.gstate.just_reloaded() {
         self.write_cfg(&tui);
      }
      self.read_cfg(&tui);
      self.update_tui_settings(&mut tui);
      match event {
         Some(Event::Key(k)) => self.handle_key(&mut tui, k),
         _ => {}
      }
   }

   fn render(&self, tui: TUIRef, buf: &mut Buffer) {
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
            Constraint::Max(gap_w + 1),
         ],
      )
      .split(lines[1]);
      let bot_box = lines[2];

      self.render_blank_box(&tui, 0, top_box, buf);
      //self.render_blank_box(&tui, 1, bot_box, buf);
      self.render_blank_box(&tui, 2, mid_line[0], buf);
      //self.render_blank_box(&tui, 3, mid_line[2], buf);

      let content = Layout::new(
         dir,
         [
            Constraint::Length(a_c as u16),
            Constraint::Length(b_c as u16),
         ],
      )
      .split(mid_line[1]);

      self.render_info_box(&tui, content[a], buf);
      self.render_asci_box(&tui, content[b], buf);
   }
}

impl LoopFetch {
   fn write_cfg(&mut self, tui: &TUIMutRef) -> AppOutput<()> {
      match tui.cfg.globals().get::<mlua::Table>("SETTINGS") {
         Err(e) => return app_err!("failed to get SETTINGS in lua {e}"),
         Ok(table) => {
            let _ = table.set("fps", self.settings.fps);
            let _ = table.set("tps", self.settings.tps);
            let _ = table.set("rps", self.settings.rps);

            let order = match table.get::<mlua::Table>("order") {
               Err(e) => return app_err!("failed to get SETTINGS.order in lua {e}"),
               Ok(t) => t,
            };
            match self.settings.order {
               ORDER::InfoFirst => {
                  let _ = order.set(1, "info");
                  let _ = order.set(2, "ascii");
               }
               ORDER::AsciFirst => {
                  let _ = order.set(1, "ascii");
                  let _ = order.set(2, "info");
               }
            };
            match self.settings.layout {
               LAYOUT::Vert => {
                  let _ = table.set("layout", "vertical");
               }
               LAYOUT::Horiz => {
                  let _ = table.set("layout", "horizontal");
               }
            }
         }
      };

      let info_lua = match self.info.to_lua(&tui.cfg) {
         Err(e) => return app_err!("failed to convert Info to lua {e}"),
         Ok(i) => i,
      };

      let loop_lua = match tui.gloop.to_lua(&tui.cfg) {
         Err(e) => return app_err!("failed to convert Loop to lua {e}"),
         Ok(i) => i,
      };

      match tui.cfg.globals().set("Info", info_lua) {
         Err(e) => return app_err!("failed to set Info in lua {}", e),
         Ok(_) => {}
      };

      match tui.cfg.globals().set("Loop", loop_lua) {
         Err(e) => return app_err!("failed to set Loop in lua {}", e),
         Ok(_) => {}
      };
      AppOutput::nil()
   }

   fn read_cfg(&mut self, tui: &TUIMutRef) {
      let default_settings = SETTINGS::default();
      let default_layout = LAYOUT::default();
      let default_order = ORDER::default();
      let default_style = Style::new();
      let default_lines = LINES::new();

      let globals = tui.cfg.globals();

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

   fn update_tui_settings(&self, tui: &mut TUIMutRef) {
      tui.gloop.set_fps(self.settings.fps);
      tui.gloop.set_tps(self.settings.tps);
   }

   fn render_info_box(&self, _tui: &TUIRef, layout: Rect, buf: &mut Buffer) {
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

   fn render_asci_box(&self, tui: &TUIRef, layout: Rect, buf: &mut Buffer) {
      let gloop = tui.gloop;
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
      let reloaded = if tui.gstate.is_reloading() {
         "reloaded..."
      } else {
         ""
      };

      let rps_line = Line::from(format!("rps: {:02.2} {reloaded}", self.settings.rps));
      let area_line = Line::from(format!(
         "area: {} x {}  ({})",
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

   fn render_blank_box(&self, tui: &TUIRef, id: usize, layout: Rect, buf: &mut Buffer) {
      let block = Block::new();
      //.borders(Borders::ALL)
      //.border_type(BorderType::Rounded);
      let colors = [Color::Green, Color::Green, Color::Magenta, Color::Magenta];
      let mut p = Paragraph::new("").block(block);
      if tui.gstate.is_debug() {
         p = p.style(Style::default().bg(colors[id]));
      };
      p.render(layout, buf);
   }

   fn handle_key(&mut self, tui: &mut TUIMutRef, key_event: KeyEvent) {
      let kind = key_event.kind;
      if kind == KeyEventKind::Press {
         match key_event.code {
            KeyCode::Char('q') => tui.gstate.request_exit(),
            KeyCode::Char('d') => tui.gstate.toggle_debug(),
            KeyCode::Char('r') => tui.gstate.request_reload(),
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
