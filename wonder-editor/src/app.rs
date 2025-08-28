use gpui::{
    div, prelude::*, rgb, Context, Render,
};

use crate::editor::MarkdownEditor;

pub struct WonderApp {
    editor: gpui::Entity<MarkdownEditor>,
}

impl WonderApp {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let editor = cx.new(|cx| MarkdownEditor::new(cx));
        
        Self { editor }
    }
    
    pub fn new_with_content(content: String, cx: &mut Context<Self>) -> Self {
        let editor = cx.new(|cx| MarkdownEditor::new_with_content(content, cx));
        
        Self { editor }
    }
}

impl Render for WonderApp {
    fn render(&mut self, _window: &mut gpui::Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(0x1e1e2e))
            .child(
                div()
                    .flex()
                    .h_12()
                    .w_full()
                    .bg(rgb(0x313244))
                    .px_4()
                    .items_center()
                    .child(
                        div()
                            .text_lg()
                            .text_color(rgb(0xcdd6f4))
                            .child("Wonder - Markdown Editor")
                    )
            )
            .child(
                div()
                    .flex_1()
                    .w_full()
                    .child(self.editor.clone())
            )
    }
}

#[cfg(test)]
mod tests {
    // Tests to be implemented when we set up full GPUI test infrastructure
    
    #[test]
    fn test_app_creation() {
        // Test will be implemented when we set up test infrastructure
    }
}