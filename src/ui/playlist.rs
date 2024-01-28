use crate::song::{Song, SourceItem};
use crate::ui::{Library, PlaylistPlaying, ScrollStatus, UiComponent, UiEvent, UiEventResult};
use ratatui::prelude::*;
use ratatui::widgets::Row;
use rodio::queue::SourcesQueueInput;
use rodio::OutputStreamHandle;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use uuid::Uuid;

pub struct Playlist {
    library: Option<Rc<RefCell<Library>>>,
    items: Vec<Uuid>,
    //ui
    focused: bool,
    viewpoint: RefCell<ScrollStatus>,
    //playback
    queue_tx: Arc<SourcesQueueInput<f32>>,
    playing: PlaylistPlaying,
    finished: Arc<AtomicBool>,
    paused: Arc<AtomicBool>,
    should_skip: Arc<AtomicBool>,
    pub progress_hundred_ms: Arc<AtomicU64>,
}

impl Playlist {
    pub fn new(stream: &OutputStreamHandle) -> anyhow::Result<Self> {
        let (i, o) = rodio::queue::queue(true);
        stream.play_raw(o)?;
        Ok(Self {
            library: None,
            items: Vec::new(),
            focused: false,
            viewpoint: RefCell::new(ScrollStatus {
                steps: 0,
                selected: 0,
                offset: 0,
            }),
            queue_tx: i,
            playing: PlaylistPlaying::None,
            finished: Arc::new(AtomicBool::new(false)),
            paused: Arc::new(AtomicBool::new(false)),
            should_skip: Arc::new(AtomicBool::new(false)),
            progress_hundred_ms: Arc::new(AtomicU64::new(0)),
        })
    }
    pub fn set_ref_to_library(&mut self, library: Rc<RefCell<Library>>) {
        self.library = Some(library);
    }
    fn next_item(&mut self) {
        self.viewpoint.borrow_mut().steps += 1;
    }
    fn prev_item(&mut self) {
        self.viewpoint.borrow_mut().steps -= 1;
    }
    pub fn playing(&self) -> Option<Song> {
        if let PlaylistPlaying::Index(i) = self.playing {
            if let SourceItem::Song(_, x) = self
                .library
                .as_ref()
                .unwrap()
                .borrow()
                .find_by_id(&self.items[i])
                .unwrap()
            {
                Some((*x).clone())
            } else {
                unreachable!()
            }
        } else {
            None
        }
    }
    pub fn pause_or_resume(&mut self) {
        self.paused.fetch_xor(true, Ordering::SeqCst);
    }
    pub fn next_song(&mut self) {
        self.should_skip.store(true, Ordering::SeqCst);
    }
    pub fn play_song(&mut self, id: Uuid) {
        use rodio::Source;
        self.items.push(id);
        match self.playing {
            PlaylistPlaying::None => {
                self.playing = PlaylistPlaying::Index(0);
            }
            PlaylistPlaying::Done => {
                self.playing = PlaylistPlaying::Index(self.items.len() - 1);
            }
            PlaylistPlaying::Index(_) => {}
        }
        let sync_signal = self.queue_tx.append_with_signal(
            if let SourceItem::Song(_, x) = self
                .library
                .as_ref()
                .unwrap()
                .borrow()
                .find_by_id(&id)
                .unwrap()
            {
                let finished = Arc::clone(&self.finished);
                let paused = self.paused.clone();
                let should_skip = self.should_skip.clone();
                let progress_hundred_ms = self.progress_hundred_ms.clone();
                x.decode()
                    .unwrap()
                    .pausable(false)
                    .skippable()
                    .periodic_access(std::time::Duration::from_millis(100), move |x| {
                        if should_skip.load(Ordering::Acquire) {
                            x.skip();
                            finished.store(true, Ordering::Release);
                            should_skip.store(false, Ordering::Release);
                            return;
                        }
                        x.inner_mut().set_paused(paused.load(Ordering::Acquire));
                        if !paused.load(Ordering::Acquire) {
                            progress_hundred_ms.fetch_add(1, Ordering::Release);
                        }
                    })
            } else {
                unreachable!()
            },
        );
        let finished = Arc::clone(&self.finished);
        tokio::task::spawn_blocking(move || {
            sync_signal.recv().unwrap();
            finished.store(true, Ordering::Release);
        });
    }
}

impl UiComponent for Playlist {
    fn handle_event(&mut self, event: UiEvent) -> UiEventResult {
        use crossterm::event::KeyCode as C;
        match event {
            UiEvent::Key(key) => match key {
                C::Char('j') => {
                    self.next_item();
                    UiEventResult::Handled
                }
                C::Char('k') => {
                    self.prev_item();
                    UiEventResult::Handled
                }
                _ => UiEventResult::PassThrough,
            },
            UiEvent::FocusGained => {
                self.focused = true;
                UiEventResult::Handled
            }
            UiEvent::FocusLost => {
                self.focused = false;
                UiEventResult::Handled
            }
            UiEvent::Tick => {
                if self.finished.load(Ordering::Acquire) {
                    self.progress_hundred_ms.store(0, Ordering::Release);
                    self.finished.store(false, Ordering::Release);
                    if let PlaylistPlaying::Index(i) = self.playing {
                        if i + 1 == self.items.len() {
                            self.playing = PlaylistPlaying::Done;
                        } else {
                            self.playing = PlaylistPlaying::Index(i + 1);
                        }
                    }
                }
                UiEventResult::PassThrough
            }
        }
    }
    fn render(&self, frame: &mut Frame, area: Rect) {
        let block = ratatui::widgets::Block::new()
            .borders(ratatui::widgets::Borders::all())
            .border_type(ratatui::widgets::BorderType::Rounded)
            .title("Playlist")
            .title_style(Style::default().fg(if self.focused {
                Color::Blue
            } else {
                Color::Reset
            }))
            .title_alignment(Alignment::Center);
        let height = block.inner(area).height as usize;

        let len = self.items.len();
        self.viewpoint.borrow_mut().calculate_steps(len, height);

        let table = ratatui::widgets::Table::new(
            self.items
                .iter()
                .enumerate()
                .skip(self.viewpoint.borrow().selected - self.viewpoint.borrow().offset)
                .take(height)
                .map(|(i, x)| {
                    Row::new([
                        if let SourceItem::Song(_, s) = self
                            .library
                            .as_ref()
                            .unwrap()
                            .borrow()
                            .find_by_id(x)
                            .unwrap()
                        {
                            s.get_title().unwrap_or("NO TITLE").to_owned()
                        } else {
                            unreachable!()
                        },
                    ])
                    .fg({
                        let selected = self.viewpoint.borrow().selected;
                        let playing = match self.playing {
                            PlaylistPlaying::None => usize::MIN,
                            PlaylistPlaying::Index(x) => x,
                            PlaylistPlaying::Done => usize::MAX,
                        };
                        if i == selected && i == playing {
                            Color::LightMagenta
                        } else if i == selected {
                            Color::Blue
                        } else if i == playing {
                            Color::LightRed
                        } else if i < playing {
                            Color::Gray
                        } else {
                            Color::Reset
                        }
                    })
                }),
            [Constraint::Percentage(100)],
        )
        .block(block);
        frame.render_widget(table, area);
    }
}
