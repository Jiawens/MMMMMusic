mod tui;
pub use tui::Tui;
mod status_line;
pub use status_line::StatusLine;
mod library;
pub use library::Library;
mod playlist;
pub use playlist::Playlist;
mod player;
pub use player::Player;

use ratatui::prelude::*;
use ratatui::Frame;

pub enum UiEvent {
    Tick,
    Key(crossterm::event::KeyCode),
    FocusGained,
    FocusLost,
}
#[derive(PartialEq)]
pub enum UiEventResult {
    Handled,
    PassThrough,
}
pub trait UiComponent {
    fn handle_event(&mut self, event: UiEvent) -> UiEventResult;
    fn render(&self, frame: &mut Frame, area: Rect);
}

#[derive(Copy, Clone)]
pub enum Focus {
    Library,
    Playlist,
    Player,
    StatusLine,
}

struct ScrollStatus {
    steps: isize,
    selected: usize,
    offset: usize,
}
impl ScrollStatus {
    pub fn calculate_steps(&mut self, len: usize, height: usize) {
        for _ in self.steps..0 {
            match (self.selected, self.offset) {
                (0, 0) => {}
                (1, 1) => {
                    self.offset -= 1;
                    self.selected -= 1;
                }
                (_, 1) => {
                    self.selected -= 1;
                }
                (s, o) if s >= o => {
                    self.offset -= 1;
                    self.selected -= 1;
                }
                (_, _) => unreachable!(),
            }
        }
        for _ in 0..self.steps {
            match (
                (len - 1) - self.selected,
                (std::cmp::min(len, height) - 1) - self.offset,
            ) {
                (0, 0) => {}
                (1, 1) => {
                    self.offset += 1;
                    self.selected += 1;
                }
                (_, 1) => {
                    self.selected += 1;
                }
                (s, o) if s >= o => {
                    self.offset += 1;
                    self.selected += 1;
                }
                (_, _) => unreachable!(),
            }
        }
        self.steps = 0;
    }
}

#[derive(PartialEq)]
enum PlaylistPlaying {
    None,
    Index(usize),
    Done,
}
