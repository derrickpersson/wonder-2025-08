mod app;
mod editor;
mod markdown_parser;
mod hybrid_renderer;
mod rendering;
mod core;
mod input;
mod benchmarks;

use gpui::*;
use std::env;
use std::fs;

use crate::app::WonderApp;

fn main() {
    let args: Vec<String> = env::args().collect();
    let debug_mode = args.contains(&"--debug".to_string());
    
    Application::new().run(move |cx: &mut App| {
        cx.open_window(WindowOptions::default(), move |_, cx| {
            if debug_mode {
                // Load example markdown content for debugging
                let example_content = match fs::read_to_string("example_markdown.md") {
                    Ok(content) => content,
                    Err(_) => {
                        // Fallback content if file doesn't exist
                        include_str!("../example_markdown.md").to_string()
                    }
                };
                cx.new(|cx| WonderApp::new_with_content(example_content, cx))
            } else {
                cx.new(|cx| WonderApp::new(cx))
            }
        })
        .unwrap();
        
        cx.activate(true);
    });
}
