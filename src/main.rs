mod app;
mod fetch;

use app::LoopFetch;

fn main() {
   katatui::TUI::<LoopFetch>::run();
}
