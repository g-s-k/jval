use gtk::prelude::*;
use gtk::{Window, WindowType};
use relm::{connect, connect_stream, Relm, Update, Widget};
use relm_derive::Msg;

use jval::Spacing;

struct Model {
    text: String,
}

#[derive(Msg)]
enum Msg {
    DoFmt,
    Text,
    Quit,
}

#[derive(Clone)]
struct Widgets {
    fmt_choose: gtk::ComboBoxText,
    text_buf: gtk::TextBuffer,
    window: Window,
}

struct Win {
    model: Model,
    widgets: Widgets,
}

impl Update for Win {
    type Model = Model;
    type ModelParam = ();
    type Msg = Msg;

    fn model(_: &Relm<Self>, _: ()) -> Model {
        Model {
            text: String::new(),
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            Msg::DoFmt => {
                let spacing = match self
                    .widgets
                    .fmt_choose
                    .get_active_text()
                    .expect("spacing should never be unselected")
                    .as_ref()
                {
                    "none" => Spacing::None,
                    "4 spaces" => Spacing::Space(4),
                    "2 spaces" => Spacing::Space(2),
                    "tabs" => Spacing::Tab,
                    _ => unreachable!(),
                };

                let json: jval::Json = self.model.text.parse().unwrap();
                let mut buf = Vec::with_capacity(self.model.text.len());
                json.print(&spacing, &mut buf).unwrap();
                self.model.text = String::from_utf8_lossy(&buf).into();
                self.widgets.text_buf.set_text(&self.model.text);
            }
            Msg::Text => {
                let (start, end) = self.widgets.text_buf.get_bounds();
                if let Some(text) = self.widgets.text_buf.get_text(&start, &end, true) {
                    self.model.text = text.into();
                }
            }
            Msg::Quit => gtk::main_quit(),
        }
    }
}

impl Widget for Win {
    type Root = Window;

    fn root(&self) -> Self::Root {
        self.widgets.window.clone()
    }

    fn view(relm: &Relm<Self>, model: Self::Model) -> Self {
        let textview = gtk::TextView::new();
        textview.set_editable(true);
        textview.set_monospace(true);
        let text_buf = textview
            .get_buffer()
            .expect("text view does not have a buffer");
        connect!(relm, text_buf, connect_changed(_), Msg::Text);

        let scroller = gtk::ScrolledWindow::new(
            Some(&gtk::Adjustment::new(1., 0., 0., 0., 0., 0.)),
            Some(&gtk::Adjustment::new(1., 0., 0., 0., 0., 0.)),
        );
        scroller.set_hexpand(true);
        scroller.set_vexpand(true);
        scroller.add(&textview);

        let next_err = gtk::Button::new_with_label("Go to next error");

        let fmt_choose = gtk::ComboBoxText::new();
        fmt_choose.append_text("none");
        fmt_choose.append_text("2 spaces");
        fmt_choose.append(Some("4"), "4 spaces");
        fmt_choose.append_text("8 spaces");
        fmt_choose.append_text("tabs");
        fmt_choose.set_active_id(Some("4"));

        let do_fmt = gtk::Button::new_with_label("Format");
        connect!(relm, do_fmt, connect_clicked(_), Msg::DoFmt);

        let grd = gtk::Grid::new();
        grd.attach(&scroller, 0, 0, 10, 5);
        grd.attach(&next_err, 0, 6, 1, 1);
        grd.attach(&fmt_choose, 7, 6, 2, 1);
        grd.attach(&do_fmt, 9, 6, 1, 1);

        let window = Window::new(WindowType::Toplevel);
        window.set_title("JSON Validator");
        window.set_resizable(false);
        window.set_property_default_width(600);
        window.set_property_default_height(400);
        window.add(&grd);

        connect!(
            relm,
            window,
            connect_delete_event(_, _),
            return (Some(Msg::Quit), Inhibit(false))
        );

        window.show_all();

        Win {
            model,
            widgets: Widgets {
                window,
                fmt_choose,
                text_buf,
            },
        }
    }
}

fn main() {
    Win::run(()).unwrap();
}
