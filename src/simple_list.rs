use crate::common::{setup_window, ListItem, State, HEIGHT, WIDTH};
use crate::keymap_file::KeymapFile;
use gpui::*;

pub struct Main {
    list_state: ListState,
    state_model: Model<State>,
    gpuilist_actions: Vec<Box<dyn Fn(Div, &mut ViewContext<Self>) -> Div>>,
}

use gpui::actions;
actions!(gpuilist, [AddItem, Quit]);

impl Render for Main {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let mut context = KeyContext::default();
        context.add("gpuilist");

        let state_model_clone = self.state_model.clone();
        let add_item_button = div()
            .flex()
            .p_2()
            .bg(rgb(0x2a2a2a))
            .rounded_md()
            .hover(|s| s.bg(rgb(0x3a3a3a)))
            .text_color(rgb(0xffffff))
            .text_xl()
            .cursor(CursorStyle::PointingHand)
            .child("Add Item")
            .on_mouse_down(MouseButton::Left, move |_mde, cx| {
                cx.update_model(&state_model_clone, |model, cx| {
                    let new_item =
                        ListItem::new(format!("Item {}", model.count), "Subtitle".to_string());
                    model.items.push(new_item);
                    model.count += 1;
                    cx.notify();
                });
            });

        let quit_button = div()
            .flex()
            .p_2()
            .bg(rgb(0x2a2a2a))
            .rounded_md()
            .hover(|s| s.bg(rgb(0x3a3a3a)))
            .text_color(rgb(0xffffff))
            .text_xl()
            .cursor(CursorStyle::PointingHand)
            .child("Quit")
            .on_mouse_down(MouseButton::Left, move |_mde, cx| {
                println!("We got the quit click !");
                cx.quit();
            });

        self.actions(div(), cx)
            .key_context(context)
            .size_full()
            .flex()
            .flex_col()
            .child(list(self.list_state.clone()).w_full().h_full())
            .child(
                div()
                    .flex()
                    .w_full()
                    .py_2()
                    .justify_center()
                    .items_center()
                    .child(add_item_button)
                    .gap_3()
                    .child(quit_button),
            )
    }
}

impl Main {
    pub fn new(cx: &mut WindowContext) -> View<Self> {
        cx.new_view(|cx| {
            let state_model = cx.new_model(|_cx| State {
                count: 0,
                items: vec![],
            });

            cx.observe(&state_model, |this: &mut Main, model, cx| {
                let items = model.read(cx).items.clone();
                this.list_state = ListState::new(
                    items.len(),
                    ListAlignment::Bottom,
                    Pixels(20.),
                    move |idx, _cx| {
                        let item = items.get(idx).unwrap().clone();
                        div().child(item).into_any_element()
                    },
                );
            })
            .detach();

            Self {
                list_state: ListState::new(0, ListAlignment::Bottom, Pixels(20.), move |_, _| {
                    div().into_any_element()
                }),
                state_model,
                gpuilist_actions: Default::default(),
            }
        })
    }

    fn quit(&mut self, _: &Quit, cx: &mut ViewContext<Self>) {
        cx.spawn(|_this, mut cx| async move {
            cx.update(|cx| cx.quit())?;
            anyhow::Ok(())
        })
        .detach_and_log_err(cx);
    }

    fn add_workspace_actions_listeners(&self, div: Div, cx: &mut ViewContext<Self>) -> Div {
        let mut div = div.on_action(cx.listener(Self::quit));
        for action in self.gpuilist_actions.iter() {
            div = (action)(div, cx)
        }
        div
    }

    fn actions(&self, div: Div, cx: &mut ViewContext<Self>) -> Div {
        self.add_workspace_actions_listeners(div, cx)
            .on_action(cx.listener(Self::quit))
    }
}

pub fn init(cx: &mut AppContext) {
    cx.on_action(quit);
}

fn quit(_: &Quit, cx: &mut AppContext) {
    cx.spawn(|cx| async move {
        cx.update(|cx| cx.quit())?;
        anyhow::Ok(())
    })
    .detach_and_log_err(cx);
}

pub fn run_app(app: App) {
    app.run(|cx: &mut AppContext| {
        init(cx);
        let window_options = setup_window(WIDTH, HEIGHT, cx);
        load_keymap(cx);
        cx.open_window(window_options, |cx| Main::new(cx));
    });
}

fn load_keymap(cx: &mut AppContext) {
    KeymapFile::load_asset("keymaps/default.json", cx).unwrap();
}
