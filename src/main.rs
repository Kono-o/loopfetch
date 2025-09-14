mod app;
mod fetch;

use katatui::*;

fn main() {
   entry::run::<app::LoopFetch>().out();
}
