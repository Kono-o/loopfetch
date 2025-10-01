mod app;
mod fetch;

use app::LoopFetch;
use katatui::TUI;

fn main() {
   TUI::<LoopFetch>::run();
}
