use fltk::{
    app::{App, Scheme},
    button,
    enums,
    // menu::*,
    // output::*,
    frame,
    group,
    dialog,
    input,
    valuator,
    widget,
    window::*,
};

const W: i32 = 600;
const H: i32 = 700;

use hapi_rs::{parameter::*, session::*};
use std::ops::{Deref, DerefMut};

fn yellow_box(x: i32, y: i32, width: i32) -> frame::Frame {
    let mut f = frame::Frame::new(x, y, width, 30, "@+");
    f.set_frame(enums::FrameType::BorderBox);
    f.set_color(enums::Color::Yellow);
    f
}

struct ColorWidget {
    color: button::Button,
    cr: input::FloatInput,
    cg: input::FloatInput,
    cb: input::FloatInput,
}

impl ColorWidget {
    fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        let mut x = x;
        let grp = group::Group::new(x, y, w, h, "");
        let w = w / 4;
        let mut color =button::Button::new(x, y, w, h, "");
        let cr= input::FloatInput::new(x + w * 1, y, w, h, "");
        let cg= input::FloatInput::new(x + w * 2, y, w, h, "");
        let cb= input::FloatInput::new(x + w * 3, y, w, h, "");
        grp.end();

        let mut _color = color.clone();
        let mut _cr = cr.clone();
        let mut _cg = cg.clone();
        let mut _cb = cb.clone();
        color.set_callback(move || {
            let clr = dialog::color_chooser("Color", 3).unwrap();
            _color.set_color(enums::Color::from_rgb(clr.0, clr.1, clr.2));
            _cr.set_value(&format!("{:.3}", clr.0 as f32/255.0));
            _cg.set_value(&format!("{:.3}", clr.1 as f32/255.0));
            _cb.set_value(&format!("{:.3}", clr.2 as f32/255.0));
        });
        Self {
            color,
            cr,
            cg,
            cb
        }
    }
}

fn build_ui(parms: Vec<Parameter>) -> Result<group::Group> {
    let mut root = group::Group::new(10, 10, W - 20, H - 20, "");
    root.set_frame(group::FrameType::DownFrame);

    for (i, parm) in parms
        .iter()
        // .take(50)
        .filter(|p| {
            let info = p.info();
            !matches!(info.parm_type(), ParmType::Folder | ParmType::Folderlist)
                && !info.invisible()
        })
        .enumerate()
    {
        let info = parm.info();
        let label = info.label()?;
        println!(
            "[{:?}] {}: {}",
            info.parm_type(),
            info.name().unwrap(),
            info.label().unwrap()
        );
        const HEIGHT: i32 = 30;
        let mut row = group::Group::new(
            root.x(),
            root.y() + (i as i32 * HEIGHT),
            root.width(),
            HEIGHT,
            "",
        );
        let mut w_label = frame::Frame::new(row.x(), row.y(), 200, HEIGHT, &label);
        let x = w_label.x() + w_label.width();
        let y = w_label.y();
        let width = root.width() - w_label.width();
        match parm {
            Parameter::Float(_) => {
                if info.size() == 1 {
                    let mut _slider = valuator::HorValueSlider::new(x, y, width, HEIGHT, &label);
                    _slider.set_bounds(0.0, 1.0);
                    _slider.set_range(0.0, 1.0);
                } else {
                    if info.parm_type() == ParmType::Color {
                        let color = ColorWidget::new(x, y, width, HEIGHT);
                    } else {
                        let w = width / info.size();
                        for i in 0..info.size() {
                            input::FloatInput::new(x + w * i, y, w, HEIGHT, "");
                        }
                    }
                }
            }
            Parameter::Int(_) => {
                if info.parm_type() == ParmType::Button {
                    let mut bt = button::Button::new(x, y, width, HEIGHT, &label);
                    bt.set_callback(|| println!("Hello there"));
                } else {
                    if info.size() == 1 {
                        let mut _slider = valuator::HorValueSlider::new(x, y, width, HEIGHT, &label);
                        _slider.set_bounds(0.0, 10.0);
                        _slider.set_range(0.0, 10.0);
                        _slider.set_step(1.0, 1);
                    }
                }
            }
            Parameter::String(_) => {
                yellow_box(
                    w_label.x() + w_label.width(),
                    w_label.y(),
                    root.width() - w_label.width(),
                );
            }
            Parameter::Other(_) => {
                yellow_box(
                    w_label.x() + w_label.width(),
                    w_label.y(),
                    root.width() - w_label.width(),
                );
            }
        }
        row.end();
    }
    let last = root.child(root.children() - 1).unwrap();
    let y = last.y() + last.height();
    let mut void = frame::Frame::new(root.x(), y, root.width(), root.height() - y, "");
    void.hide();
    root.end();
    root.resizable(&void);
    Ok(root)
}

fn run() -> Result<()> {
    let mut session = Session::connect_to_server("/tmp/hapi")?;
    session.initialize(&SessionOptions::default());
    let _lib = session.load_asset_file("/Users/alex/CLionProjects/hapi-rs/otls/hapi_parms.hda")?;
    let node = session.create_node_blocking("Object/hapi_parms", None, None)?;

    let app = App::default().with_scheme(Scheme::Gtk);
    let (screen_width, screen_height) = fltk::app::screen_size();
    let mut wind = Window::new(
        (screen_width / 2.0 - 250.0) as i32,
        (screen_height / 2.0 - 200.0) as i32,
        W,
        H,
        "Hapi Parms",
    );

    build_ui(node.parameters()?);
    wind.make_resizable(true);
    wind.end();
    wind.show();

    app.run().unwrap();
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
    }
}
