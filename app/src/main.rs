//! The `naran-app` executable. Everything it does is in the library; this only
//! starts the platform's event loop, which a test cannot run.

fn main() {
    gpui::Application::new().run(|cx| naran_app::start(cx, &naran_app::config_directory()));
}
