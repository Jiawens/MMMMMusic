mod ui;
use ui::{UiComponent, UiEvent, UiEventResult};
mod config;
use config::{sources, FOCUSED_FRAME_DELAY, UNFOCUSED_FRAME_DELAY};
mod song;
use ratatui::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
use ui::Tui;

pub struct Core {
    frame_delay: f64,
    focus: ui::Focus,
    status_line: ui::StatusLine,
    library: Rc<RefCell<ui::Library>>,
    playlist: Rc<RefCell<ui::Playlist>>,
    player: ui::Player,
}
impl Core {
    pub fn switch_focus(&mut self, focus: ui::Focus) {
        self.focus = focus;
        self.status_line.handle_event(UiEvent::FocusLost);
        self.library.borrow_mut().handle_event(UiEvent::FocusLost);
        self.playlist.borrow_mut().handle_event(UiEvent::FocusLost);
        self.player.handle_event(UiEvent::FocusLost);
        match focus {
            ui::Focus::Library => self.library.borrow_mut().handle_event(UiEvent::FocusGained),
            ui::Focus::Playlist => self
                .playlist
                .borrow_mut()
                .handle_event(UiEvent::FocusGained),
            ui::Focus::Player => self.player.handle_event(UiEvent::FocusGained),
            ui::Focus::StatusLine => self.status_line.handle_event(UiEvent::FocusGained),
        };
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut tui = Tui::run()?;
    tui.setup_panic();

    let (_s, stream) = rodio::OutputStream::try_default()?;

    use tokio_stream::StreamExt;
    let mut core = Core {
        frame_delay: 1f64 / 10f64,
        focus: ui::Focus::Library,
        status_line: ui::StatusLine::NothingButHappy,
        library: Rc::new(RefCell::new(ui::Library::new())),
        playlist: Rc::new(RefCell::new(ui::Playlist::new(&stream)?)),
        player: ui::Player::new(),
    };
    core.playlist
        .borrow_mut()
        .set_ref_to_library(Rc::clone(&core.library));
    core.player.set_ref_to_playlist(Rc::clone(&core.playlist));
    sources(&mut core.library.borrow_mut())?;
    core.library.borrow_mut().handle_event(UiEvent::FocusGained);

    let mut event_stream = crossterm::event::EventStream::new();
    loop {
        core.status_line.handle_event(UiEvent::Tick);
        core.library.borrow_mut().handle_event(UiEvent::Tick);
        core.playlist.borrow_mut().handle_event(UiEvent::Tick);
        core.player.handle_event(UiEvent::Tick);
        tui.draw(|f| {
            let status_line_and_others = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(0), Constraint::Length(1)])
                .split(f.size());
            core.status_line.render(f, status_line_and_others[1]);
            let library_and_others = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Min(0), Constraint::Length(30)])
                .split(status_line_and_others[0]);
            core.library.borrow().render(f, library_and_others[0]);
            let playlist_and_others = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(0), Constraint::Length(4)])
                .split(library_and_others[1]);
            core.playlist.borrow_mut().render(f, playlist_and_others[0]);
            core.player.render(f, playlist_and_others[1]);
        })?;
        tokio::select! {
            Some(Ok(e)) = event_stream.next() => {
                use crossterm::event::Event as E;
                match e {
                    E::FocusGained => core.frame_delay = FOCUSED_FRAME_DELAY,
                    E::FocusLost => core.frame_delay = UNFOCUSED_FRAME_DELAY,
                    E::Key(crossterm::event::KeyEvent{code: c,..}) => {
                        use crossterm::event::KeyCode as C;
                        use ui::Focus as F;
                        match (core.focus,c) {
                            (F::StatusLine, c) if core.status_line.handle_event(UiEvent::Key(c))==UiEventResult::Handled => {},
                            (F::Library, c) if core.library.borrow_mut().handle_event(UiEvent::Key(c))==UiEventResult::Handled => {},
                            (F::Playlist, c) if core.playlist.borrow_mut().handle_event(UiEvent::Key(c))==UiEventResult::Handled => {},
                            (F::Player, c) if core.player.handle_event(UiEvent::Key(c))==UiEventResult::Handled => {},

                            (F::Library, C::Char(']')) => core.switch_focus(F::Playlist),
                            (F::Playlist, C::Char('[')) => core.switch_focus(F::Library),
                            (F::Playlist, C::Char(']')) => core.switch_focus(F::Player),
                            (F::Player, C::Char('[')) => core.switch_focus(F::Playlist),

                            (F::Library, C::Enter) => {
                                match core.library.borrow().find_by_id(&core.library.borrow().selected_id()).unwrap() {
                                    song::SourceItem::Song(id, _) => core.playlist.borrow_mut().play_song(*id),
                                    song::SourceItem::Title(..) => {},
                                }
                            },

                            (F::Player, C::Char(' ')) => core.playlist.borrow_mut().pause_or_resume(),
                            (F::Player, C::Char('l')) => core.playlist.borrow_mut().next_song(),

                            (F::StatusLine, C::Backspace) => {
                                core.status_line = ui::StatusLine::NothingButHappy;
                                core.switch_focus(F::Library);
                            }
                            (F::StatusLine, C::Enter) => {
                                if let ui::StatusLine::Find(ref s) = core.status_line {
                                    core.library.borrow_mut().set_find(if s.is_empty() {None} else {Some(s.clone())});
                                    core.switch_focus(F::Library);
                                }
                                core.status_line = ui::StatusLine::NothingButHappy;
                            }

                            (_, C::Char('?')) => {
                                core.focus = F::StatusLine;
                                core.library.borrow_mut().handle_event(UiEvent::FocusLost);
                                core.playlist.borrow_mut().handle_event(UiEvent::FocusLost);
                                core.status_line = ui::StatusLine::Find("".to_owned());
                            }
                            (_, C::Char('q')) => break,
                            _ => {},
                        }
                    }
                    _ => {}
                }
            }
            _ = tokio::time::sleep(Duration::from_secs_f64(core.frame_delay)) => {}
        }
    }
    Ok(())
}
