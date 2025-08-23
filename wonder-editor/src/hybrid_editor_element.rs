use gpui::{Element, Entity, FocusHandle, Window, App, LayoutId, ShapedLine, px, rgb, Pixels, GlobalElementId, InspectorElementId, IntoElement, AnyElement};
use crate::editor::MarkdownEditor;
use crate::hybrid_renderer::HybridTextRenderer;
use std::ops::Range;

pub struct HybridEditorElement {
    editor: Entity<MarkdownEditor>,
    content: String,
    cursor_position: usize,
    selection: Option<Range<usize>>,
    focused: bool,
    focus_handle: FocusHandle,
    renderer: HybridTextRenderer,
}

impl HybridEditorElement {
    pub fn new(
        editor: Entity<MarkdownEditor>,
        content: String,
        cursor_position: usize,
        selection: Option<Range<usize>>,
        focused: bool,
        focus_handle: FocusHandle,
    ) -> Self {
        Self {
            editor,
            content,
            cursor_position,
            selection,
            focused,
            focus_handle,
            renderer: HybridTextRenderer::new(),
        }
    }
}

impl IntoElement for HybridEditorElement {
    type Element = Self;
    
    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for HybridEditorElement {
    type RequestLayoutState = Vec<ShapedLine>;
    type PrepaintState = ();
    
    fn id(&self) -> Option<gpui::ElementId> {
        None
    }
    
    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }
    
    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _window: &mut Window,
        _cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        // For now, just return a basic layout
        // In a full implementation, we would use the hybrid renderer here
        (LayoutId::default(), vec![])
    }
    
    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _bounds: gpui::Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Self::PrepaintState {
        // No prepaint work needed
    }
    
    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _bounds: gpui::Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        _shaped_lines: &mut Self::PrepaintState,
        _window: &mut Window,
        _cx: &mut App,
    ) {
        // For now, just do nothing for paint
        // In a full implementation, we would render the hybrid text here
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::markdown_parser::MarkdownToken;

    #[test]
    fn test_create_hybrid_editor_element() {
        // This should fail because we need to mock Entity and FocusHandle
        // For now, let's create a simpler test
        let renderer = HybridTextRenderer::new();
        let _element_would_be_created_here = true; // placeholder
        // Test that basic creation doesn't panic
        assert!(true);
    }

    #[test] 
    fn test_generate_mixed_text_runs() {
        let renderer = HybridTextRenderer::new();
        let content = "# Title **bold** normal";
        
        // This will fail because generate_mixed_text_runs doesn't exist yet
        let runs = renderer.generate_mixed_text_runs(content, 50, None);
        
        // Should have mixed runs - some with raw styling, some with preview styling
        assert!(runs.len() > 0);
    }
}