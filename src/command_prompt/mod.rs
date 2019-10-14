use termion::event::Key;

pub mod parser;
pub use self::parser::{Command, CommandParseError as ParseError};

#[derive(Clone, Debug, PartialEq)]
pub enum CommandMachineEvent<'a> {
    Reset,
    Update(&'a str),
    Execute(Command),
    UnknownCommand(String),
}

pub struct CommandPrompt {
    pub text: String,
    pub index: usize,
}

impl CommandPrompt {
    pub fn new() -> Self {
        Self {
            index: 0,
            text: String::new(),
        }
    }

    pub fn step(&mut self, key: Key) -> CommandMachineEvent {
        match key {
            Key::Char('\n') => {
                let result = match parser::parse_command(&self.text) {
                    Ok(command) => CommandMachineEvent::Execute(command),
                    Err(..) => CommandMachineEvent::UnknownCommand(self.text.to_owned()),
                };

                self.text.clear();
                self.index = 0;

                result
            }
            Key::Ctrl('c') => {
                self.text.clear();
                self.index = 0;
                CommandMachineEvent::Reset
            }
            Key::Char(x) => {
                self.text.push(x);
                self.index += 1;
                CommandMachineEvent::Update(&self.text)
            }
            Key::Backspace => {
                if self.index > 0 {
                    self.index -= 1;
                    self.text.remove(self.index);
                    CommandMachineEvent::Update(&self.text)
                } else {
                    CommandMachineEvent::Update(&self.text)
                }
            }
            _ => CommandMachineEvent::Update(&self.text),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Command, CommandMachineEvent, CommandPrompt};
    use termion::event::Key;

    #[test]
    fn it_builds_text_based_on_input() {
        let mut command = CommandPrompt::new();
        command.step(Key::Char('h'));
        command.step(Key::Char('e'));
        command.step(Key::Char('l'));
        command.step(Key::Char('l'));
        command.step(Key::Char('o'));

        assert_eq!(command.text, "hello");
        assert_eq!(command.index, 5);
    }

    #[test]
    fn it_keeps_track_of_index() {
        let mut command = CommandPrompt::new();
        command.step(Key::Char('h'));
        assert_eq!(command.index, 1);
        command.step(Key::Char('i'));
        assert_eq!(command.index, 2);
    }

    #[test]
    fn it_deletes_text_when_backspacing() {
        let mut command = CommandPrompt::new();
        command.step(Key::Char('h'));
        command.step(Key::Char('i'));

        command.step(Key::Backspace);
        assert_eq!(command.index, 1);
        assert_eq!(command.text, "h");

        command.step(Key::Backspace);
        assert_eq!(command.index, 0);
        assert_eq!(command.text, "");
    }

    #[test]
    fn it_does_nothing_when_backspacing_past_the_start_of_text() {
        let mut command = CommandPrompt::new();
        command.step(Key::Backspace);
        assert_eq!(command.index, 0);
        assert_eq!(command.text, "");
    }

    #[test]
    fn it_returns_an_execute_command_on_successful_match() {
        let mut command = CommandPrompt::new();
        command.step(Key::Char('y'));
        command.step(Key::Char(' '));
        command.step(Key::Char('3'));
        command.step(Key::Char('2'));
        let result = command.step(Key::Char('\n'));

        assert_eq!(result, CommandMachineEvent::Execute(Command::ScrollY(32)));
    }
}
