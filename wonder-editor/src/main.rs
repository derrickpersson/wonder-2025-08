mod app;
mod editor;
mod text_buffer;
mod markdown_parser;
mod preview_renderer;
mod hybrid_renderer;
// mod hybrid_editor_element;
mod core;
mod input;

use gpui::*;

use crate::app::WonderApp;

fn main() {
    Application::new().run(|cx: &mut App| {
        cx.open_window(WindowOptions::default(), |_, cx| {
            cx.new(|cx| WonderApp::new(cx))
        })
        .unwrap();
        
        cx.activate(true);
    });
}
