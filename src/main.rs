mod common;
mod simple_list;

use gpui::App;

fn main() {
    let app = App::new();
    simple_list::run_app(app);
}
