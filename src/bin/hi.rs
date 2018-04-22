#![feature(nll)]
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
use hi::context::Context;
use hi::screen::Screen;
use hi::{Frame, State};

enum HandlerStatus {
    Continue,
    Quit,
}

struct EventHandler<'a, 'b: 'a, T: 'a>
where
    T: Write,
{
    prompt: CommandPrompt,
    screen: &'a mut Screen<'b, T>,
}

impl<'a, 'b: 'a, T: 'a> EventHandler<'a, 'b, T>
where
    T: Write,
{
    fn new(screen: &'a mut Screen<'b, T>) -> Self {
        let prompt = CommandPrompt::new();
        Self { prompt, screen }
    }

    fn call(
        &mut self,
        context: &Context,
        event: termion::event::Event,
    ) -> Result<HandlerStatus, Box<StdError>> {
        let screen = &mut self.screen;

        match screen.state {
            State::Wait => match event {
                Event::Key(Key::Char('q')) => return Ok(HandlerStatus::Quit),
                Event::Key(Key::Char('h')) => screen.scroll_left(),
                Event::Key(Key::Char('l')) => screen.scroll_right(),
                Event::Key(Key::Char('j')) => screen.down(),
                Event::Key(Key::Char('k')) => screen.up(),
                Event::Key(Key::Char(':')) => screen.prompt(),
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

        screen.render(context, &self.prompt)?;
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
    let stdout = stdout().into_raw_mode()?;
    let (width, height) = termion::terminal_size()?;
    let mut screen = Screen::new(&bytes, Frame { width, height }, stdout);
    let context = Context { file_path: &path };

    screen.render(&context, &CommandPrompt::new())?;

    let mut handler = EventHandler::new(&mut screen);
    for event in stdin.events() {
        let event = event?;
        match handler.call(&context, event)? {
            HandlerStatus::Continue => {}
            HandlerStatus::Quit => break,
        };
    }

    screen.reset()?;

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
