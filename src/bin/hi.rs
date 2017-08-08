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

    status_bar::render(&mut stdout, &frame, &path, &state);

    let mut bytes_per_row = 32;
    byte_display::render(
        &mut stdout,
        scroll,
        &mut bytes,
        bytes_per_row,
        frame.height - 2,
    );
    stdout.flush().unwrap();

    let mut command_machine = CommandPrompt::new();

    for evt in stdin.events() {
        // set panel height to frame height minus status bar and command bar
        let main_panel_height = frame.height - 1 - 1;
        let evt = evt.unwrap();

        match state {
            State::Wait => match evt {
                Event::Key(Key::Char('q')) => {
                    write!(stdout, "{}", termion::cursor::Goto(1, 1)).unwrap();
                    break;
                }
                Event::Key(Key::Char('h')) => if offset > 0 {
                    offset -= 1;
                    let len = bytes.len();
                    let data = &mut bytes[offset..len];
                    byte_display::render(
                        &mut stdout,
                        scroll,
                        data,
                        bytes_per_row,
                        main_panel_height,
                    );
                },
                Event::Key(Key::Char('l')) => if offset < bytes.len() - 1 {
                    offset += 1;
                    let len = bytes.len();
                    let data = &mut bytes[offset..len];
                    byte_display::render(
                        &mut stdout,
                        scroll,
                        data,
                        bytes_per_row,
                        main_panel_height,
                    );
                },
                Event::Key(Key::Char('j')) => {
                    let row_count = bytes.len() / bytes_per_row as usize + 1;
                    let limit_scroll: usize = row_count - (frame.height as usize - 2) + 40;
                    if scroll < limit_scroll {
                        scroll += 1;
                    }
                    let len = bytes.len();
                    let data = &mut bytes[offset..len];
                    byte_display::render(
                        &mut stdout,
                        scroll,
                        data,
                        bytes_per_row,
                        main_panel_height,
                    );
                }
                Event::Key(Key::Char('k')) => if scroll > 0 {
                    if scroll > 0 {
                        scroll -= 1;
                    }
                    let len = bytes.len();
                    let data = &mut bytes[offset..len];
                    byte_display::render(
                        &mut stdout,
                        scroll,
                        data,
                        bytes_per_row,
                        main_panel_height,
                    );
                },
                Event::Key(Key::Char(':')) => {
                    state = State::Prompt;

                    status_bar::render(&mut stdout, &frame, &path, &state);

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
                _ => {}
            },
            State::Prompt => match evt {
                Event::Key(x) => {
                    use hi::command_prompt::Command::SetWidth;
                    command_machine = command_machine.step(x);
                    match command_machine.last_event {
                        CommandMachineEvent::Reset | CommandMachineEvent::UnknownCommand(..) => {
                            cursor.x = command_machine.cursor as u16 + 2;
                            write!(
                                stdout,
                                "{}{}",
                                termion::clear::CurrentLine,
                                termion::cursor::Hide
                            ).unwrap();
                            state = State::Wait;
                        }
                        CommandMachineEvent::Update => {
                            cursor.x = command_machine.cursor as u16 + 2;
                            write!(
                                stdout,
                                "{}{}:{}{}",
                                termion::cursor::Goto(1, cursor.y),
                                termion::clear::CurrentLine,
                                command_machine.text,
                                termion::cursor::Goto(cursor.x, cursor.y),
                            ).unwrap();
                        }
                        CommandMachineEvent::Execute(SetWidth(n)) => {
                            bytes_per_row = n;
                            cursor.x = command_machine.cursor as u16 + 2;
                            write!(
                                stdout,
                                "{}{}",
                                termion::clear::CurrentLine,
                                termion::cursor::Hide
                            ).unwrap();
                            state = State::Wait;

                            let len = bytes.len();
                            let data = &mut bytes[offset..len];
                            byte_display::render(
                                &mut stdout,
                                scroll,
                                data,
                                bytes_per_row,
                                main_panel_height,
                            );
                        }
                    }
                }
                e => {
                    let message = format!("{:?}", e);
                    info!("{}", message);
                }
            },
        }
        stdout.flush().unwrap();
    }

    write!(stdout, "{}", termion::cursor::Show).unwrap();
}
