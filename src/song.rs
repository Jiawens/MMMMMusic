use std::cell::RefCell;
use std::time::Duration;
use uuid::Uuid;

#[derive(Clone)]
pub enum Song {
    File {
        path: String,
        metadata: RefCell<Option<SongMetadata>>,
    },
}

#[derive(Clone)]
pub struct SongMetadata {
    title: String,
    artist: String,
    duration: Duration,
}

impl Song {
    fn read_metadata(&self) -> anyhow::Result<SongMetadata> {
        match self {
            Self::File { path, .. } => {
                let tag = audiotags::Tag::new().read_from_path(path)?;
                Ok(SongMetadata {
                    title: tag
                        .title()
                        .map(|x| x.to_string())
                        .unwrap_or_else(|| path.split('/').last().unwrap().to_owned()),
                    artist: tag.artist().map(|x| x.to_string()).unwrap_or_default(),
                    duration: tag
                        .duration()
                        .map(|x| Duration::from_secs(x as u64))
                        .unwrap_or_default(),
                })
            }
        }
    }
    pub fn get_title(&self) -> anyhow::Result<&str> {
        match self {
            Self::File {
                path: _path,
                metadata,
            } => {
                if metadata.borrow().is_none() {
                    *metadata.borrow_mut() = Some(self.read_metadata()?);
                }
                Ok(unsafe { &metadata.try_borrow_unguarded()?.as_ref().unwrap().title })
            }
        }
    }
    pub fn get_artist(&self) -> anyhow::Result<&str> {
        match self {
            Self::File {
                path: _path,
                metadata,
            } => {
                if metadata.borrow().is_none() {
                    *metadata.borrow_mut() = Some(self.read_metadata()?);
                }
                Ok(unsafe { &metadata.try_borrow_unguarded()?.as_ref().unwrap().artist })
            }
        }
    }
    pub fn get_duration(&self) -> anyhow::Result<Duration> {
        match self {
            Self::File {
                path: _path,
                metadata,
            } => {
                if metadata.borrow().is_none() {
                    *metadata.borrow_mut() = Some(self.read_metadata()?);
                }
                Ok(metadata.borrow().as_ref().unwrap().duration)
            }
        }
    }
    pub fn decode(&self) -> anyhow::Result<impl rodio::Source<Item = f32> + Send + 'static> {
        use rodio::source::Source;
        match self {
            Self::File { path, .. } => {
                Ok(rodio::Decoder::new(std::fs::File::open(path)?)?.convert_samples())
            }
        }
    }
}

pub struct Source {
    title: String,
    items: Vec<(Uuid, Song)>,
}
pub enum SourceItem<'a> {
    Title(&'a Uuid, &'a String),
    Song(&'a Uuid, &'a Song),
}

impl Source {
    pub fn from_file(title: Option<String>, path: String) -> anyhow::Result<Self> {
        let song = Song::File {
            path,
            metadata: RefCell::new(None),
        };
        Ok(Self {
            title: title.unwrap_or_else(|| song.get_title().unwrap().to_owned()),
            items: vec![(Uuid::new_v4(), song)],
        })
    }
    pub fn from_directory(title: Option<String>, path: String) -> anyhow::Result<Self> {
        let len = std::fs::read_dir(&path)?.count();
        let mut items = Vec::with_capacity(len);
        for entry in std::fs::read_dir(&path)? {
            let entry = entry?;
            let path = entry.path();
            if path.ends_with(".DS_Store") {
                continue;
            }
            items.push((
                Uuid::new_v4(),
                Song::File {
                    path: path.to_str().unwrap().to_owned(),
                    metadata: RefCell::new(None),
                },
            ));
        }
        Ok(Self {
            title: title.unwrap_or_else(|| path.split('/').last().unwrap().to_owned()),
            items,
        })
    }
    pub fn iter<'a>(
        &'a self,
        title_uuid: &'a Uuid,
    ) -> impl Iterator<Item = SourceItem> + DoubleEndedIterator + '_ {
        std::iter::once(SourceItem::Title(title_uuid, &self.title))
            .chain(self.items.iter().map(|(x, y)| SourceItem::Song(x, y)))
    }
}
