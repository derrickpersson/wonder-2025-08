#[derive(Debug, Clone, PartialEq)]
pub enum InputEvent {
    Character(char),
    Backspace,
    Delete,
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    ArrowDown,
    ShiftArrowLeft,
    ShiftArrowRight,
    CmdArrowLeft,
    CmdArrowRight,
    CmdShiftArrowLeft,
    CmdShiftArrowRight,
    Home,
    End,
    PageUp,
    PageDown,
    Enter,
    Tab,
}

impl InputEvent {
    pub fn from_char(ch: char) -> Self {
        InputEvent::Character(ch)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpecialKey {
    Backspace,
    Delete,
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    ArrowDown,
    ShiftArrowLeft,
    ShiftArrowRight,
    CmdArrowLeft,
    CmdArrowRight,
    CmdShiftArrowLeft,
    CmdShiftArrowRight,
    Home,
    End,
    PageUp,
    PageDown,
    Enter,
    Tab,
}

impl From<SpecialKey> for InputEvent {
    fn from(key: SpecialKey) -> Self {
        match key {
            SpecialKey::Backspace => InputEvent::Backspace,
            SpecialKey::Delete => InputEvent::Delete,
            SpecialKey::ArrowLeft => InputEvent::ArrowLeft,
            SpecialKey::ArrowRight => InputEvent::ArrowRight,
            SpecialKey::ArrowUp => InputEvent::ArrowUp,
            SpecialKey::ArrowDown => InputEvent::ArrowDown,
            SpecialKey::ShiftArrowLeft => InputEvent::ShiftArrowLeft,
            SpecialKey::ShiftArrowRight => InputEvent::ShiftArrowRight,
            SpecialKey::CmdArrowLeft => InputEvent::CmdArrowLeft,
            SpecialKey::CmdArrowRight => InputEvent::CmdArrowRight,
            SpecialKey::CmdShiftArrowLeft => InputEvent::CmdShiftArrowLeft,
            SpecialKey::CmdShiftArrowRight => InputEvent::CmdShiftArrowRight,
            SpecialKey::Home => InputEvent::Home,
            SpecialKey::End => InputEvent::End,
            SpecialKey::PageUp => InputEvent::PageUp,
            SpecialKey::PageDown => InputEvent::PageDown,
            SpecialKey::Enter => InputEvent::Enter,
            SpecialKey::Tab => InputEvent::Tab,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_event_creation() {
        let char_event = InputEvent::from_char('a');
        assert_eq!(char_event, InputEvent::Character('a'));
    }

    #[test]
    fn test_special_key_conversion() {
        let backspace_event: InputEvent = SpecialKey::Backspace.into();
        assert_eq!(backspace_event, InputEvent::Backspace);
        
        let arrow_event: InputEvent = SpecialKey::ArrowLeft.into();
        assert_eq!(arrow_event, InputEvent::ArrowLeft);
    }
}