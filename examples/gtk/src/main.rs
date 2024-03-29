use std::ops::Range;

use gtk::prelude::*;
use gtk::{
    Adjustment, Button, ComboBoxText, Grid, ScrolledWindow, TextBuffer, TextView, Window,
    WindowType,
};
use relm::{connect, connect_stream, Relm, Update, Widget};
use relm_derive::Msg;

use jval::{Json, Spacing};

struct Model {
    errs: Vec<Range<usize>>,
    json: Option<Json>,
}

#[derive(Msg)]
enum Msg {
    NextErr,
    DoFmt,
    Text,
    Quit,
}

#[derive(Clone)]
struct Widgets {
    err_btn: Button,
    fmt_btn: Button,
    fmt_choose: ComboBoxText,
    text_buf: TextBuffer,
    text_view: TextView,
    window: Window,
}

struct Win {
    model: Model,
    widgets: Widgets,
}

impl Win {
    fn get_text(&self) -> Option<String> {
        let (start, end) = self.widgets.text_buf.get_bounds();
        if let Some(text) = self.widgets.text_buf.get_text(&start, &end, true) {
            Some(text.into())
        } else {
            None
        }
    }
}

impl Update for Win {
    type Model = Model;
    type ModelParam = ();
    type Msg = Msg;

    fn model(_: &Relm<Self>, _: ()) -> Model {
        Model {
            errs: Vec::new(),
            json: None,
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            Msg::NextErr => {
                if let Some((Range { start, end }, _)) = self.model.errs.split_first() {
                    let (mut buf_start, mut buf_end) = self.widgets.text_buf.get_bounds();
                    buf_start.set_offset(*start as i32);
                    buf_end.set_offset(*end as i32);
                    self.widgets.text_buf.select_range(&buf_start, &buf_end);
                    self.widgets.text_view.grab_focus();
                }
            }

            Msg::DoFmt => {
                if let Some(json) = &self.model.json {
                    let spacing = match self
                        .widgets
                        .fmt_choose
                        .get_active_text()
                        .expect("spacing should never be unselected")
                        .as_ref()
                    {
                        "none" => Spacing::None,
                        "8 spaces" => Spacing::Space(8),
                        "4 spaces" => Spacing::Space(4),
                        "2 spaces" => Spacing::Space(2),
                        "tabs" => Spacing::Tab,
                        _ => unreachable!(),
                    };

                    let mut buf = Vec::new();
                    json.print(&spacing, &mut buf)
                        .expect("was valid JSON, but could not print it");
                    self.widgets
                        .text_buf
                        .set_text(&String::from_utf8_lossy(&buf));
                    self.widgets.text_view.grab_focus();
                }
            }

            Msg::Text => {
                if let Some(text) = self.get_text() {
                    match text.parse::<Json>() {
                        Ok(json) => {
                            self.model.errs = Vec::new();
                            self.model.json = Some(json);

                            self.widgets.err_btn.set_sensitive(false);
                            self.widgets.fmt_btn.set_sensitive(true);
                        }
                        Err(v) => {
                            self.model.errs = v.into_iter().map(|(_, r)| r).collect();
                            self.model.json = None;

                            self.widgets.err_btn.set_sensitive(!text.trim().is_empty());
                            self.widgets.fmt_btn.set_sensitive(false);
                        }
                    }
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
        let text_view = TextView::new();
        text_view.set_editable(true);
        text_view.set_monospace(true);
        let text_buf = text_view
            .get_buffer()
            .expect("text view does not have a buffer");
        connect!(relm, text_buf, connect_changed(_), Msg::Text);

        let scroller = ScrolledWindow::new(
            Some(&Adjustment::new(1., 0., 0., 0., 0., 0.)),
            Some(&Adjustment::new(1., 0., 0., 0., 0., 0.)),
        );
        scroller.set_hexpand(true);
        scroller.set_vexpand(true);
        scroller.add(&text_view);

        let err_btn = Button::new_with_label("Go to next error");
        err_btn.set_sensitive(false);
        connect!(relm, err_btn, connect_clicked(_), Msg::NextErr);

        let fmt_choose = ComboBoxText::new();
        fmt_choose.append_text("none");
        fmt_choose.append_text("2 spaces");
        fmt_choose.append(Some("4"), "4 spaces");
        fmt_choose.append_text("8 spaces");
        fmt_choose.append_text("tabs");
        fmt_choose.set_active_id(Some("4"));

        let fmt_btn = Button::new_with_label("Format");
        fmt_btn.set_sensitive(false);
        connect!(relm, fmt_btn, connect_clicked(_), Msg::DoFmt);

        let grd = Grid::new();
        grd.attach(&scroller, 0, 0, 10, 5);
        grd.attach(&err_btn, 0, 6, 1, 1);
        grd.attach(&fmt_choose, 7, 6, 2, 1);
        grd.attach(&fmt_btn, 9, 6, 1, 1);

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
                err_btn,
                fmt_btn,
                fmt_choose,
                text_buf,
                text_view,
                window,
            },
        }
    }
}

fn main() {
    Win::run(()).unwrap();
}
