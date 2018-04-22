extern crate env_logger;
extern crate hi;
#[macro_use]
extern crate log;
extern crate termion;

use std::env;
use std::error::Error as StdError;
use std::fs;
use std::io::Read;
use std::io::{stdin, stdout, Write};

use termion::event::{Event, Key};
use termion::input::TermRead;
use termion::raw::IntoRawMode;

use hi::command_prompt::{CommandMachineEvent, CommandPrompt};
use hi::screen::Screen;
use hi::{byte_display, status_bar, Frame, State};

fn run() -> Result<(), Box<StdError>> {
    env_logger::init()?;
    let path = env::args().nth(1).expect("Usage: hi FILE");
    let mut file = fs::File::open(&path)?;

    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;

    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode()?;
    write!(
        stdout,
        "{}{}",
        termion::clear::All,
        termion::cursor::Goto(1, 1)
    )?;

    write!(stdout, "{}", termion::cursor::Hide,)?;

    let (width, height) = termion::terminal_size()?;
    let mut screen = Screen::new(&bytes, Frame { width, height });

    stdout.flush()?;

    byte_display::render(&mut stdout, screen.data, &screen);
    status_bar::render(&mut stdout, &screen, &path);
    stdout.flush()?;

    let mut command_machine = CommandPrompt::new();

    for evt in stdin.events() {
        screen.clear_dirty_flags();
        let evt = evt?;
        let mut command_bar_focus = false;

        match screen.state {
            State::Wait => match evt {
                Event::Key(Key::Char('q')) => {
                    write!(stdout, "{}", termion::cursor::Goto(1, 1))?;
                    break;
                }
                Event::Key(Key::Char('h')) => screen.scroll_left(),
                Event::Key(Key::Char('l')) => screen.scroll_right(),
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
                    use hi::command_prompt::Command::{ScrollX, ScrollY, SetOffset, SetWidth};
                    command_machine = command_machine.step(x);
                    match command_machine.last_event {
                        CommandMachineEvent::Reset | CommandMachineEvent::UnknownCommand(..) => {
                            screen.reset_prompt()
                        }
                        CommandMachineEvent::Update => screen.update_prompt(),
                        CommandMachineEvent::Execute(SetWidth(n)) => screen.set_width(n),
                        CommandMachineEvent::Execute(SetOffset(n)) => screen.set_offset(n),
                        CommandMachineEvent::Execute(ScrollX(n)) => screen.set_scroll_x(n),
                        CommandMachineEvent::Execute(ScrollY(n)) => screen.set_scroll_y(n),
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
            byte_display::render(&mut stdout, data, &screen);
        }

        if screen.status_bar_dirty {
            status_bar::render(&mut stdout, &screen, &path);
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
                    )?;
                }
                Update => {
                    write!(
                        stdout,
                        "{}{}{}:{}",
                        termion::cursor::Show,
                        termion::cursor::Goto(1, screen.frame.height),
                        termion::clear::CurrentLine,
                        command_machine.text,
                    )?;
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
            )?;
        }

        stdout.flush()?;
    }

    write!(stdout, "{}", termion::cursor::Show)?;
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
