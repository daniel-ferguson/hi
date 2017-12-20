extern crate env_logger;
extern crate hi;
#[macro_use]
extern crate log;
extern crate termion;

use std::env;
use std::fs;
use std::io::Read;
use std::io::{stdin, stdout, Write};

use termion::event::{Event, Key};
use termion::input::TermRead;
use termion::raw::IntoRawMode;

use hi::command_prompt::{CommandMachineEvent, CommandPrompt};
use hi::screen::Screen;
use hi::{byte_display, status_bar, Frame, State};

fn main() {
    env_logger::init().unwrap();
    let path = env::args().nth(1).expect("Usage: hi FILE");
    let mut file = fs::File::open(&path).expect("File not found");

    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).unwrap();

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
    let mut screen = Screen::new(
        &bytes,
        Frame {
            width: width,
            height: height,
        },
    );

    stdout.flush().unwrap();

    byte_display::render(
        &mut stdout,
        screen.scroll,
        screen.data,
        screen.bytes_per_row,
        screen.data_frame_height(),
    );
    status_bar::render(
        &mut stdout,
        &screen.frame,
        &path,
        &screen.state,
        screen.offset,
        screen.scroll,
        screen.bytes_per_row,
    );
    stdout.flush().unwrap();

    let mut command_machine = CommandPrompt::new();

    for evt in stdin.events() {
        screen.clear_dirty_flags();
        let evt = evt.unwrap();
        let mut command_bar_focus = false;

        match screen.state {
            State::Wait => match evt {
                Event::Key(Key::Char('q')) => {
                    write!(stdout, "{}", termion::cursor::Goto(1, 1)).unwrap();
                    break;
                }
                Event::Key(Key::Char('h')) => screen.left(),
                Event::Key(Key::Char('l')) => screen.right(),
                Event::Key(Key::Char('j')) => screen.down(),
                Event::Key(Key::Char('k')) => screen.up(),
                Event::Key(Key::Char(':')) => {
                    screen.prompt();
                    command_bar_focus = true;
                }
                Event::Key(Key::Ctrl('d')) | Event::Key(Key::PageDown) => screen.page_down(),
                Event::Key(Key::Ctrl('u')) | Event::Key(Key::PageUp) => screen.page_up(),
                Event::Key(Key::Home) => screen.start(),
                Event::Key(Key::End) => screen.end(),
                _ => {}
            },
            State::Prompt => match evt {
                Event::Key(x) => {
                    use hi::command_prompt::Command::{SetOffset, SetWidth};
                    command_machine = command_machine.step(x);
                    match command_machine.last_event {
                        CommandMachineEvent::Reset | CommandMachineEvent::UnknownCommand(..) => {
                            screen.reset_prompt()
                        }
                        CommandMachineEvent::Update => screen.update_prompt(),
                        CommandMachineEvent::Execute(SetWidth(n)) => screen.set_width(n),
                        CommandMachineEvent::Execute(SetOffset(n)) => screen.set_offset(n),
                    }
                }
                e => {
                    let message = format!("{:?}", e);
                    info!("{}", message);
                }
            },
        }

        if screen.data_frame_dirty {
            let len = screen.data.len();
            let data = &screen.data[screen.offset..len];
            byte_display::render(
                &mut stdout,
                screen.scroll,
                data,
                screen.bytes_per_row,
                screen.data_frame_height(),
            );
        }

        if screen.status_bar_dirty {
            status_bar::render(
                &mut stdout,
                &screen.frame,
                &path,
                &screen.state,
                screen.offset,
                screen.scroll,
                screen.bytes_per_row,
            );
        }

        use CommandMachineEvent::{Execute, Reset, Update};
        if screen.prompt_bar_dirty {
            match command_machine.last_event {
                Reset | Execute(_) | CommandMachineEvent::UnknownCommand(_) => {
                    write!(
                        stdout,
                        "{}{}{}",
                        termion::cursor::Goto(1, screen.frame.height),
                        termion::clear::CurrentLine,
                        termion::cursor::Hide
                    ).unwrap();
                }
                Update => {
                    write!(
                        stdout,
                        "{}{}{}:{}",
                        termion::cursor::Show,
                        termion::cursor::Goto(1, screen.frame.height),
                        termion::clear::CurrentLine,
                        command_machine.text,
                    ).unwrap();
                }
            }
        }

        if command_bar_focus {
            write!(
                stdout,
                "{}{}{}:",
                termion::cursor::Show,
                termion::cursor::Goto(1, screen.frame.height),
                termion::clear::CurrentLine,
            ).unwrap();
        }

        stdout.flush().unwrap();
    }

    write!(stdout, "{}", termion::cursor::Show).unwrap();
}
