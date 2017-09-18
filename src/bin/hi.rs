extern crate env_logger;
extern crate hi;
#[macro_use]
extern crate log;
extern crate termion;

use std::io::{stdin, stdout, Write};
use std::env;
use std::fs;
use std::io::Read;

use termion::event::{Event, Key};
use termion::input::TermRead;
use termion::raw::IntoRawMode;

use hi::{byte_display, status_bar, Cursor, Frame, State};
use hi::command_prompt::{CommandMachineEvent, CommandPrompt};

fn main() {
    env_logger::init().unwrap();
    let path = env::args().nth(1).expect("Usage: hi FILE");
    let mut file = fs::File::open(&path).expect("File not found");

    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).unwrap();

    let mut scroll = 0;
    let mut cursor = Cursor { x: 1, y: 1 };

    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();
    write!(
        stdout,
        "{}{}",
        termion::clear::All,
        termion::cursor::Goto(1, 1)
    ).unwrap();

    write!(stdout, "{}", termion::cursor::Hide,).unwrap();

    let (width, height) = termion::terminal_size().unwrap();
    let frame = Frame {
        width: width,
        height: height,
    };

    stdout.flush().unwrap();

    let mut offset: usize = 0;

    let mut state = State::Wait;

    let mut bytes_per_row = 32;
    byte_display::render(&mut stdout, scroll, &bytes, bytes_per_row, frame.height - 2);
    status_bar::render(
        &mut stdout,
        &frame,
        &path,
        &state,
        offset,
        scroll,
        bytes_per_row,
    );
    stdout.flush().unwrap();

    let mut command_machine = CommandPrompt::new();

    for evt in stdin.events() {
        // set panel height to frame height minus status bar and command bar
        let main_panel_height = frame.height - 1 - 1;
        let evt = evt.unwrap();
        let mut byte_display_dirty = false;
        let mut status_bar_dirty = false;

        match state {
            State::Wait => match evt {
                Event::Key(Key::Char('q')) => {
                    write!(stdout, "{}", termion::cursor::Goto(1, 1)).unwrap();
                    break;
                }
                Event::Key(Key::Char('h')) => if offset > 0 {
                    offset -= 1;

                    status_bar_dirty = true;
                    byte_display_dirty = true;
                },
                Event::Key(Key::Char('l')) => if offset < bytes.len() - 1 {
                    offset += 1;

                    status_bar_dirty = true;
                    byte_display_dirty = true;
                },
                Event::Key(Key::Char('j')) => {
                    let len = bytes.len();
                    let data = &bytes[offset..len];

                    if scroll < max_scroll(frame.height as usize - 2, &data, bytes_per_row) {
                        scroll += 1;
                    }

                    status_bar_dirty = true;
                    byte_display_dirty = true;
                }
                Event::Key(Key::Char('k')) => if scroll > 0 {
                    if scroll > 0 {
                        scroll -= 1;
                    }

                    status_bar_dirty = true;
                    byte_display_dirty = true;
                },
                Event::Key(Key::Char(':')) => {
                    state = State::Prompt;

                    let cursor = &mut cursor;
                    cursor.x = 1;
                    cursor.y = frame.height;

                    write!(
                        stdout,
                        "{}{}:",
                        termion::cursor::Goto(cursor.x, cursor.y),
                        termion::cursor::Show
                    ).unwrap();
                    cursor.x += 1;
                }
                Event::Key(Key::Ctrl('d')) | Event::Key(Key::PageDown) => {
                    let byte_display_height = frame.height as usize - 2;
                    let len = bytes.len();
                    let data = &bytes[offset..len];

                    if scroll + byte_display_height
                        < max_scroll(byte_display_height, &data, bytes_per_row)
                    {
                        scroll += byte_display_height;
                    } else {
                        scroll = max_scroll(byte_display_height, &data, bytes_per_row);
                    }

                    status_bar_dirty = true;
                    byte_display_dirty = true;
                }
                Event::Key(Key::Ctrl('u')) | Event::Key(Key::PageUp) => {
                    let byte_display_height = frame.height as usize - 2;

                    if byte_display_height > scroll {
                        scroll = 0;
                    } else {
                        scroll -= byte_display_height;
                    }

                    status_bar_dirty = true;
                    byte_display_dirty = true;
                }
                Event::Key(Key::Home) => {
                    scroll = 0;

                    status_bar_dirty = true;
                    byte_display_dirty = true;
                }
                Event::Key(Key::End) => {
                    let len = bytes.len();
                    let data = &bytes[offset..len];

                    scroll = max_scroll(frame.height as usize - 2, &data, bytes_per_row);

                    status_bar_dirty = true;
                    byte_display_dirty = true;
                }
                _ => {}
            },
            State::Prompt => match evt {
                Event::Key(x) => {
                    use hi::command_prompt::Command::{SetOffset, SetWidth};
                    command_machine = command_machine.step(x);
                    match command_machine.last_event {
                        CommandMachineEvent::Reset | CommandMachineEvent::UnknownCommand(..) => {
                            write!(
                                stdout,
                                "{}{}{}",
                                termion::cursor::Goto(1, frame.height),
                                termion::clear::CurrentLine,
                                termion::cursor::Hide
                            ).unwrap();
                            state = State::Wait;
                        }
                        CommandMachineEvent::Update => {
                            write!(
                                stdout,
                                "{}{}:{}",
                                termion::cursor::Goto(1, cursor.y),
                                termion::clear::CurrentLine,
                                command_machine.text,
                            ).unwrap();
                        }
                        CommandMachineEvent::Execute(SetWidth(n)) => {
                            let anchor = top_left_byte_index(offset, scroll, bytes_per_row);
                            bytes_per_row = n;

                            let mut s = scroll_for_anchor(anchor, offset, bytes_per_row);
                            let mut o = offset_for_anchor(anchor, offset, bytes_per_row);

                            s += o / bytes_per_row;
                            o = o % bytes_per_row;


                            scroll = s;
                            offset = o;

                            write!(
                                stdout,
                                "{}{}{}",
                                termion::cursor::Goto(1, frame.height),
                                termion::clear::CurrentLine,
                                termion::cursor::Hide
                            ).unwrap();
                            state = State::Wait;

                            status_bar_dirty = true;
                            byte_display_dirty = true;
                        }
                        CommandMachineEvent::Execute(SetOffset(n)) => {
                            offset = n;
                            write!(
                                stdout,
                                "{}{}",
                                termion::clear::CurrentLine,
                                termion::cursor::Hide
                            ).unwrap();
                            state = State::Wait;

                            status_bar_dirty = true;
                            byte_display_dirty = true;
                        }
                    }
                }
                e => {
                    let message = format!("{:?}", e);
                    info!("{}", message);
                }
            },
        }

        if status_bar_dirty {
            status_bar::render(
                &mut stdout,
                &frame,
                &path,
                &state,
                offset,
                scroll,
                bytes_per_row,
            );
        }

        if byte_display_dirty {
            let len = bytes.len();
            let data = &bytes[offset..len];
            byte_display::render(&mut stdout, scroll, data, bytes_per_row, main_panel_height);
        }

        stdout.flush().unwrap();
    }

    write!(stdout, "{}", termion::cursor::Show).unwrap();
}

fn top_left_byte_index(offset: usize, scroll: usize, bytes_per_row: usize) -> usize {
    offset + scroll * bytes_per_row
}

fn scroll_for_anchor(anchor: usize, offset: usize, bytes_per_row: usize) -> usize {
    (anchor - offset) / bytes_per_row
}

fn offset_for_anchor(anchor: usize, offset: usize, bytes_per_row: usize) -> usize {
    offset + (anchor - offset) % bytes_per_row
}

fn max_scroll(height: usize, data: &[u8], width: usize) -> usize {
    let lines = data.len() / width as usize;
    if lines > height {
        lines - height / 2
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod max_scroll {
        use super::max_scroll;

        #[test]
        fn it_allows_scrolling_half_a_screen_past_end_of_data() {
            // data displayed across more rows than height
            let height = 60;
            let data: &[u8; 80] = &[0; 80];
            let width = 1;
            assert_eq!(max_scroll(height, data, width), 20 + height / 2);
        }

        #[test]
        fn it_disables_scroll_when_data_fits_on_one_screen() {
            // data displayed across fewer rows than height
            let height = 60;
            let data: &[u8; 20] = &[0; 20];
            let width = 1;
            assert_eq!(max_scroll(height, data, width), 0);
        }
    }
}
