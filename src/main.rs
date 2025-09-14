use katatui::buffer::Buffer;
use katatui::crossterm::event::Event;
use katatui::layout::Rect;
use katatui::*;

struct A {}

impl App for A {
   const APP_NAME: &'static str = "";
   const CONFIG_FILE: &'static str = "";

   fn init(gloop: &mut GLoop, cfg_src: std::string::String) -> AppOutput<Self>
   where
      Self: Sized,
   {
      todo!()
   }

   fn reload(&mut self, gloop: &mut GLoop, cfg_src: std::string::String) -> AppOutput<()>
   where
      Self: Sized,
   {
      todo!()
   }

   fn logic(&mut self, gloop: &mut GLoop, gstate: &mut GState, event: Option<Event>)
   where
      Self: Sized,
   {
      todo!()
   }

   fn render(&self, gloop: &GLoop, gstate: &GState, area: Rect, buf: &mut Buffer)
   where
      Self: Sized,
   {
      todo!()
   }
}

fn main() {
   run::<A>().out();
}
