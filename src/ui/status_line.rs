use crate::ui::{UiComponent, UiEvent, UiEventResult};
use ratatui::layout::Rect;
use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

pub enum StatusLine {
    NothingButHappy,
    Find(String),
}

impl UiComponent for StatusLine {
    fn handle_event(&mut self, event: UiEvent) -> UiEventResult {
        match event {
            UiEvent::FocusGained => UiEventResult::Handled,
            UiEvent::FocusLost => UiEventResult::Handled,
            UiEvent::Tick => UiEventResult::PassThrough,
            UiEvent::Key(crossterm::event::KeyCode::Backspace) => match self {
                Self::Find(s) => {
                    if !s.is_empty() {
                        s.pop();
                        UiEventResult::Handled
                    } else {
                        UiEventResult::PassThrough
                    }
                }
                _ => UiEventResult::PassThrough,
            },
            UiEvent::Key(crossterm::event::KeyCode::Char(c)) => match self {
                Self::Find(s) => {
                    s.push(c);
                    UiEventResult::Handled
                }
                _ => UiEventResult::PassThrough,
            },
            _ => UiEventResult::PassThrough,
        }
    }
    fn render(&self, frame: &mut Frame, area: Rect) {
        frame.render_widget(
            match self {
                Self::NothingButHappy => Paragraph::new("> Life goes on~"),
                Self::Find(s) => Paragraph::new("?".to_owned() + s),
            },
            area,
        );
    }
}
