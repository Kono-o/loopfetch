mod app;
mod fetch;

use katatui::*;

fn main() {
   entry::tui::<app::LoopFetch>();
}
