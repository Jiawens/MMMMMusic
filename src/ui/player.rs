use crate::ui::{Playlist, UiComponent, UiEvent, UiEventResult};
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Color, Style};
use ratatui::widgets::Paragraph;
use ratatui::Frame;
use std::borrow::Cow;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::Ordering;

pub struct Player {
    playlist: Option<Rc<RefCell<Playlist>>>,
    focused: bool,
}

impl Player {
    pub fn new() -> Self {
        Self {
            playlist: None,
            focused: false,
        }
    }
    pub fn set_ref_to_playlist(&mut self, playlist: Rc<RefCell<Playlist>>) {
        self.playlist = Some(playlist);
    }
}

impl UiComponent for Player {
    fn handle_event(&mut self, event: UiEvent) -> UiEventResult {
        match event {
            UiEvent::FocusGained => {
                self.focused = true;
                UiEventResult::Handled
            }
            UiEvent::FocusLost => {
                self.focused = false;
                UiEventResult::Handled
            }
            _ => UiEventResult::PassThrough,
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        let playing = self.playlist.as_ref().unwrap().borrow().playing();
        let duration = playing.and_then(|x| x.get_duration().ok());
        let total_duration_secs = duration.map(|x| x.as_secs());
        let progress_duration_secs = self
            .playlist
            .as_ref()
            .unwrap()
            .borrow()
            .progress_hundred_ms
            .load(Ordering::Acquire)
            / 10;

        let block = ratatui::widgets::Block::new()
            .borders(ratatui::widgets::Borders::all())
            .border_type(ratatui::widgets::BorderType::Rounded)
            .title("Player")
            .title_alignment(Alignment::Center)
            .title_style(Style::default().fg(if self.focused {
                Color::Blue
            } else {
                Color::Reset
            }));
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(1), Constraint::Length(1)])
            .split(block.inner(area));
        frame.render_widget(block, area);

        let progress_bar = match total_duration_secs {
            Some(0) | None => Cow::Borrowed("[||||||||||||||||||||||||||]"),
            Some(d) => {
                let percent = (progress_duration_secs as f64) / (d as f64);
                let percent = percent * 26f64;
                let percent = percent.round() as usize;
                Cow::Owned(
                    "[".to_owned()
                        + (0..percent).map(|_| '=').collect::<String>().as_str()
                        + (0..26 - percent).map(|_| ' ').collect::<String>().as_str()
                        + "]",
                )
            }
        };
        let progress_bar = Paragraph::new(progress_bar).alignment(Alignment::Center);
        let progress = Paragraph::new(format!(
            "[{:2}m{:2}s::{}]",
            progress_duration_secs / 60,
            progress_duration_secs % 60,
            match total_duration_secs {
                Some(0) | None => Cow::Borrowed("--m--s"),
                Some(x) => Cow::Owned(format!("{:2}m{:2}s", x / 60, x % 60)),
            }
        ))
        .alignment(Alignment::Center);
        frame.render_widget(progress_bar, layout[0]);
        frame.render_widget(progress, layout[1]);
    }
}
