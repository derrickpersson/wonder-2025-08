mod app;
mod editor;

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
