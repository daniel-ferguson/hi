use termion::event::Key;

pub mod parser;
pub use self::parser::{Command, CommandParseError as ParseError};

#[derive(Debug)]
pub enum CommandMachineEvent {
    Reset,
    Update,
    Execute(Command),
    UnknownCommand(String),
}

pub struct CommandPrompt {
    pub text: String,
    pub index: usize,
    pub last_event: CommandMachineEvent,
}

impl CommandPrompt {
    pub fn new() -> Self {
        Self {
            index: 0,
            text: String::new(),
            last_event: CommandMachineEvent::Reset,
        }
    }

    pub fn step(mut self, key: Key) -> Self {
        match key {
            Key::Char('\n') => {
                let last_event = match parser::parse_command(&self.text) {
                    Ok(command) => CommandMachineEvent::Execute(command),
                    Err(..) => CommandMachineEvent::UnknownCommand(self.text.to_owned()),
                };
                self.text.clear();
                Self {
                    index: 0,
                    text: self.text,
                    last_event: last_event,
                }
            }
            Key::Ctrl('c') => {
                self.text.clear();
                Self {
                    index: 0,
                    text: self.text,
                    last_event: CommandMachineEvent::Reset,
                }
            }
            Key::Char(x) => {
                self.text.push(x);
                Self {
                    index: self.index + 1,
                    text: self.text,
                    last_event: CommandMachineEvent::Update,
                }
            }
            Key::Backspace => if self.index > 0 {
                self.text.remove(self.index - 1);
                Self {
                    index: self.index - 1,
                    text: self.text,
                    last_event: CommandMachineEvent::Update,
                }
            } else {
                Self {
                    index: self.index,
                    text: self.text,
                    last_event: CommandMachineEvent::Update,
                }
            },
            _ => self,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::CommandPrompt;
    use termion::event::Key;

    #[test]
    fn it_builds_text_based_on_input() {
        let command = CommandPrompt::new();
        let command = command.step(Key::Char('h'));
        let command = command.step(Key::Char('e'));
        let command = command.step(Key::Char('l'));
        let command = command.step(Key::Char('l'));
        let command = command.step(Key::Char('o'));

        assert_eq!(command.text, "hello");
        assert_eq!(command.index, 5);
    }

    #[test]
    fn it_keeps_track_of_index() {
        let command = CommandPrompt::new();
        let command = command.step(Key::Char('h'));
        assert_eq!(command.index, 1);
        let command = command.step(Key::Char('i'));
        assert_eq!(command.index, 2);
    }

    #[test]
    fn it_deletes_text_when_backspacing() {
        let command = CommandPrompt::new();
        let command = command.step(Key::Char('h'));
        let command = command.step(Key::Char('i'));

        let command = command.step(Key::Backspace);
        assert_eq!(command.index, 1);
        assert_eq!(command.text, "h");

        let command = command.step(Key::Backspace);
        assert_eq!(command.index, 0);
        assert_eq!(command.text, "");
    }

    #[test]
    fn it_does_nothing_when_backspacing_past_the_start_of_text() {
        let command = CommandPrompt::new();
        let command = command.step(Key::Backspace);
        assert_eq!(command.index, 0);
        assert_eq!(command.text, "");
    }
}
