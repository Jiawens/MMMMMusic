use crate::song::{Source, SourceItem};
use crate::ui::{ScrollStatus, UiComponent, UiEvent, UiEventResult};
use ratatui::prelude::*;
use ratatui::widgets::Row;
use std::borrow::Cow;
use std::cell::RefCell;
use uuid::Uuid;

pub struct Library {
    items: Vec<(Uuid, Source)>,
    focused: bool,
    find: Option<String>,
    viewpoint: RefCell<ScrollStatus>,
}

impl Library {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            focused: false,
            find: None,
            viewpoint: RefCell::new(ScrollStatus {
                steps: 0,
                selected: 0,
                offset: 0,
            }),
        }
    }
    pub fn add_source(&mut self, source: Source) {
        self.items.push((Uuid::new_v4(), source));
    }
    pub fn find_by_id(&self, id: &Uuid) -> Option<SourceItem> {
        self.items
            .iter()
            .flat_map(|(x, y)| y.iter(x))
            .find(|x| match x {
                SourceItem::Title(y, _) => *id == **y,
                SourceItem::Song(y, _) => *id == **y,
            })
    }
    pub fn selected_id(&self) -> Uuid {
        self.items
            .iter()
            .flat_map(|(x, y)| y.iter(x))
            .nth(self.viewpoint.borrow().selected)
            .map(|x| match x {
                SourceItem::Title(y, _) => *y,
                SourceItem::Song(y, _) => *y,
            })
            .unwrap()
    }
    pub fn set_find(&mut self, find: Option<String>) {
        self.find = find;
    }
    fn next(&mut self) {
        self.viewpoint.borrow_mut().steps += 1;
    }
    fn prev(&mut self) {
        self.viewpoint.borrow_mut().steps -= 1;
    }
}

impl UiComponent for Library {
    fn handle_event(&mut self, event: UiEvent) -> UiEventResult {
        use crossterm::event::KeyCode as C;
        match event {
            UiEvent::Key(key) => match key {
                C::Char('j') => {
                    self.next();
                    UiEventResult::Handled
                }
                C::Char('k') => {
                    self.prev();
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
            UiEvent::Tick => UiEventResult::PassThrough,
        }
    }
    fn render(&self, frame: &mut Frame, area: Rect) {
        use ratatui::layout::Constraint as C;
        let block = ratatui::widgets::Block::new()
            .borders(ratatui::widgets::Borders::all())
            .border_type(ratatui::widgets::BorderType::Rounded)
            .title("Library")
            .title_style(Style::default().fg(if self.focused {
                Color::Blue
            } else {
                Color::Reset
            }))
            .title_alignment(Alignment::Center);
        let width = block.inner(area).width;
        let height = (block.inner(area).height - 1) as usize;

        let len = self.items.iter().flat_map(|(x, y)| y.iter(x)).count();
        let selected = self.viewpoint.borrow().selected;
        let steps = self.viewpoint.borrow().steps;
        if let Some(ref str) = self.find {
            if steps > 0 {
                if let Some(z) = self
                    .items
                    .iter()
                    .flat_map(|(x, y)| y.iter(x))
                    .skip(selected)
                    .enumerate()
                    .skip(1)
                    .find_map(|(i, x)| {
                        match x {
                            SourceItem::Title(_, s) => s.contains(str),
                            SourceItem::Song(_, s) => s.get_title().unwrap_or("").contains(str),
                        }
                        .then_some(i)
                    })
                {
                    self.viewpoint.borrow_mut().steps = z as isize;
                }
            }
        }
        self.viewpoint.borrow_mut().calculate_steps(len, height);

        let table = ratatui::widgets::Table::new(
            self.items
                .iter()
                .flat_map(|(x, y)| y.iter(x))
                .enumerate()
                .skip(self.viewpoint.borrow().selected - self.viewpoint.borrow().offset)
                .take(height)
                .map(|(i, x)| {
                    match x {
                        SourceItem::Title(_, s) => Row::new(["=====", s, ""]).underlined(),
                        SourceItem::Song(_, s) => Row::new([
                            Cow::Owned(format!("{:5}", i)),
                            Cow::Owned(format!(
                                "{} - {}",
                                s.get_artist().unwrap_or("NO ARTIST"),
                                s.get_title().unwrap_or("NO TITLE")
                            )),
                            {
                                let duration = match s.get_duration() {
                                    Ok(x) => Some(x.as_secs()),
                                    Err(_) => None,
                                };
                                match duration {
                                    Some(x) => Cow::Owned(format!("{:02}:{:02}", x / 60, x % 60)),
                                    None => Cow::Borrowed("--:--"),
                                }
                            },
                        ]),
                    }
                    .fg({
                        let found = if self.find.is_some() {
                            let str = match x {
                                SourceItem::Title(_, s) => Cow::Borrowed(s),
                                SourceItem::Song(_, s) => Cow::Owned(format!(
                                    "{} {}",
                                    s.get_title().unwrap_or(""),
                                    s.get_artist().unwrap_or("")
                                )),
                            };
                            str.contains(self.find.as_ref().unwrap())
                        } else {
                            false
                        };
                        if found && i == self.viewpoint.borrow().selected {
                            Color::LightMagenta
                        } else if found {
                            Color::LightRed
                        } else if i == self.viewpoint.borrow().selected {
                            Color::Blue
                        } else {
                            Color::Reset
                        }
                    })
                }),
            [C::Length(5), C::Length(width - 15), C::Length(8)],
        )
        .block(block)
        .header(Row::new(vec!["  #", "Artist - Title", "Duration"]).underlined());
        frame.render_widget(table, area);
    }
}
