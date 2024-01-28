use crate::song::Source;

pub const FOCUSED_FRAME_DELAY: f64 = 1f64 / 10f64; // 10fps
pub const UNFOCUSED_FRAME_DELAY: f64 = 1f64; // 1fps

pub fn sources(library: &mut crate::ui::Library) -> anyhow::Result<()> {
    library.add_source(Source::from_file(
        Some("Example(file)".to_owned()),
        "/example/file.mp3".to_owned(),
    )?);
    // Ensure that the directory only contains music files and .DS_Store(will be ignored)
    library.add_source(Source::from_directory(
        Some("Example(directory)".to_owned()),
        "/example/dir".to_owned(),
    )?);
    Ok(())
}
