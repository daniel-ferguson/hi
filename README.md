(H)ex (I)nspector
=====================

Very much work in progress, lightweight interactive hex inspector.

Guaranteed broken in terrible ways but also might be just good enough. Could eat your laundry.

## Building

You'll need a reasonably recent rust compiler, with that sorted run:

    cargo run --release [filename]

## Usage

Have a look at src/bin/hi.rs and guess at keyboard shortcuts, the key section
reproduced here for convenience:

    Event::Key(Key::Char('q')) => return Ok(HandlerStatus::Quit),
    Event::Key(Key::Char('h')) => screen.scroll_left(),
    Event::Key(Key::Char('l')) => screen.scroll_right(),
    Event::Key(Key::Char('j')) => screen.down(),
    Event::Key(Key::Char('k')) => screen.up(),
    Event::Key(Key::Char(':')) => screen.prompt(),
    Event::Key(Key::Char('f')) => screen.toggle_text_display_mode(),
    Event::Key(Key::Ctrl('d')) | Event::Key(Key::PageDown) => screen.page_down(),
    Event::Key(Key::Ctrl('u')) | Event::Key(Key::PageUp) => screen.page_up(),
    Event::Key(Key::Home) => screen.start(),
    Event::Key(Key::End) => screen.end(),

You can run enter command mode by pressing `:`. Once there type your command in and press `Enter`.
Supported commands are:

    o(ffset)  N  # set offset from beginning of file
    w(idth)   N  # set number of horizontal bytes to display
    scroll(x) N  # scroll to a certain row
    scroll(y) N  # scroll to a certain column
