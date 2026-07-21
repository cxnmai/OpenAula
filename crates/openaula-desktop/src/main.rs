mod app;
mod catalog;
mod theme;

use app::OpenAulaApp;
use gpui::{AppContext as _, Application, WindowBounds, WindowOptions, px, size};
use gpui_component::{Root, Theme, ThemeMode};

fn main() {
    Application::new().run(|cx| {
        gpui_component::init(cx);
        Theme::change(ThemeMode::Dark, None, cx);
        theme::apply(cx);

        let mut window_options = WindowOptions {
            window_bounds: Some(WindowBounds::centered(size(px(1440.), px(900.)), cx)),
            window_min_size: Some(size(px(1040.), px(700.))),
            app_id: Some("dev.openaula.OpenAula".into()),
            ..Default::default()
        };
        if let Some(titlebar) = window_options.titlebar.as_mut() {
            titlebar.title = Some("OpenAula".into());
        }

        cx.spawn(async move |cx| {
            cx.open_window(window_options, |window, cx| {
                let app = cx.new(OpenAulaApp::new);
                cx.new(|cx| Root::new(app, window, cx))
            })?;

            Ok::<_, anyhow::Error>(())
        })
        .detach();
    });
}
