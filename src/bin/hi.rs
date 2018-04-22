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

struct Context<'a> {
    file_path: &'a str,
}

enum HandlerStatus {
    Continue,
    Quit,
}

struct EventHandler<'a, 'b, T: 'a>
where
    T: Write,
{
    out: &'a mut T,
    prompt: CommandPrompt,
    screen: &'b mut Screen<'b>,
}

impl<'a, 'b, T: 'a> EventHandler<'a, 'b, T>
where
    T: Write,
{
    fn new(out: &'a mut T, screen: &'b mut Screen<'b>) -> Self {
        let prompt = CommandPrompt::new();
        Self {
            out,
            prompt,
            screen,
        }
    }

    fn call(
        &mut self,
        context: &Context,
        event: termion::event::Event,
    ) -> Result<HandlerStatus, Box<StdError>> {
        let screen = &mut self.screen;
        screen.clear_dirty_flags();
        let mut command_bar_focus = false;
        let mut stdout = &mut self.out;

        match screen.state {
            State::Wait => match event {
                Event::Key(Key::Char('q')) => {
                    write!(stdout, "{}", termion::cursor::Goto(1, 1))?;
                    return Ok(HandlerStatus::Quit);
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
            State::Prompt => match event {
                Event::Key(x) => {
                    use hi::command_prompt::Command::{ScrollX, ScrollY, SetOffset, SetWidth};

                    self.prompt = self.prompt.step(x);

                    match self.prompt.last_event {
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
            status_bar::render(&mut stdout, &screen, context.file_path);
        }

        use CommandMachineEvent::{Execute, Reset, Update};
        if screen.prompt_bar_dirty {
            match self.prompt.last_event {
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
                        self.prompt.text,
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
        Ok(HandlerStatus::Continue)
    }
}

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

    let context = Context { file_path: &path };

    {
        // constrain stdout mutable borrow to event handling loop
        let mut handler = EventHandler::new(&mut stdout, &mut screen);
        for event in stdin.events() {
            let event = event?;
            match handler.call(&context, event)? {
                HandlerStatus::Continue => {}
                HandlerStatus::Quit => break,
            };
        }
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
