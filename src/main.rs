mod assets;
mod common;
mod keymap_file;
mod simple_list;

use gpui::App;

fn main() {
    let app = App::new();
    simple_list::run_app(app);
}
