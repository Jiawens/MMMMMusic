MMMMMusic
===

This is a simple local music player in tui.

Currently, I only tested it on macOS, but it should work on Linux.

## Configurations

There's no runtime configuration file.

To make changes, edit the `src/config.rs` file and recompile the program.

- `(UN)FOCUSED_FRAME_DELAY` determines the number of milliseconds it will wait before updating the UI.
- `fn sources` add sources to the library. Sources are sets of songs, either in a directory or an individual file.

## Keybindings

The ui consists of four components: `Library`, `Playlist`, `Player`, `Status Line`.

- `Library` shows all the songs in your library.
- `Playlist` shows the songs in the current playlist, highlighting the playing song using LightRed.
- `Player` shows the progress of the current song.

### Library

- `j`/`k` to move the cursor.
- `Enter` to add the selected song to the playlist.

### Playlist

- `j`/`k` to move the cursor.

### Player

- `Space` to play or pause.
- `l` to skip the current song.

### Others

- `[`/`]` to switch focus between `Library`, `Playlist`, and `Player`.
- `q` to quit.

## Search

You can search the library.

To enter search mode, press `?`, type your query, and hit `Enter`. The search results will be highlighted in LightRed.

Press `j` to jump to the next result.

To exit search mode, press `?` followed by `Enter`.