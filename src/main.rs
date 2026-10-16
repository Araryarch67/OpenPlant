//! OpenPlant — a StarUML / draw.io style UML workbench with PlantUML
//! round-tripping, rendered natively with GPUI.

mod app;
mod canvas;
mod commands;
mod model;
mod plantuml;
mod services;
mod theme;
mod ui;
#[cfg(test)]
mod ui_tests;
mod workbench;

use gpui_kit::*;

fn main() {
    let file = std::env::args().nth(1).map(std::path::PathBuf::from);
    gpui_kit::application()
        .with_assets(gpui_kit::assets::Assets)
        .run(move |cx| {
            gpui_kit::init(cx);
            theme::apply(cx);
            commands::bind_keys(cx);
            let bounds = Bounds::centered(None, size(px(1600.), px(900.)), cx);
            let options = WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some("OpenPlant".into()),
                    ..Default::default()
                }),
                app_id: Some("openplant".into()),
                window_min_size: Some(size(px(900.), px(560.))),
                ..Default::default()
            };
            gpui_kit::open_window(options, cx, |window, cx| {
                cx.new(|cx| {
                    let mut app = app::OpenPlant::new(window, cx);
                    if let Some(path) = file.clone() {
                        app.open_path(path, window, cx);
                    }
                    app
                })
            })
            .expect("failed to open the OpenPlant window");
            cx.activate(true);
        });
}
