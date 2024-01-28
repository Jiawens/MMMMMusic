use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

pub struct Tui {
    inner: Terminal<CrosstermBackend<std::io::Stderr>>,
}

impl Tui {
    pub fn run() -> anyhow::Result<Self> {
        crossterm::terminal::enable_raw_mode()?;
        crossterm::execute!(std::io::stderr(), crossterm::terminal::EnterAlternateScreen)?;
        Ok(Self {
            inner: Terminal::new(CrosstermBackend::new(std::io::stderr()))?,
        })
    }
    pub fn setup_panic(&mut self) {
        let panic_fn = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |x| {
            crossterm::execute!(std::io::stderr(), crossterm::terminal::LeaveAlternateScreen)
                .unwrap();
            crossterm::terminal::disable_raw_mode().unwrap();
            panic_fn(x);
        }));
    }
}

impl std::ops::Deref for Tui {
    type Target = Terminal<CrosstermBackend<std::io::Stderr>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl std::ops::DerefMut for Tui {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        crossterm::execute!(std::io::stderr(), crossterm::terminal::LeaveAlternateScreen).unwrap();
        crossterm::terminal::disable_raw_mode().unwrap();
    }
}
