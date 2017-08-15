#[macro_use]
extern crate log;
#[macro_use]
extern crate nom;
extern crate termion;

pub mod command_prompt;

pub struct Cursor {
    pub x: u16,
    pub y: u16,
}

#[derive(Debug)]
pub enum State {
    Wait,
    Prompt,
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            State::Wait => write!(f, "State::Wait"),
            State::Prompt => write!(f, "State::Prompt"),
        }
    }
}

pub struct Frame {
    pub width: u16,
    pub height: u16,
}

pub mod status_bar {
    use std::io::Write;

    use termion::{color, cursor};

    use super::{Frame, State};

    pub fn render<T: Write>(
        io: &mut T,
        frame: &Frame,
        path: &str,
        state: &State,
        offset: usize,
        scroll: usize,
        bytes_per_row: usize,
    ) {
        let message_right = format!("{}|o:{}|s:{}|w:{}", state, offset, scroll, bytes_per_row,);
        let bar = line_of_spaces(frame.width as usize);

        let bar_full = format!(
            "{}{}{}{}{}{}{}{}{}",
            color::Bg(color::Black),
            color::Fg(color::White),
            bar,
            cursor::Goto(1, frame.height - 1),
            path,
            cursor::Goto((bar.len() - message_right.len()) as u16, frame.height - 1),
            message_right,
            color::Bg(color::Reset),
            color::Fg(color::Reset),
        );

        write!(io, "{}{}", cursor::Goto(1, frame.height - 1), bar_full,).unwrap();
    }

    fn line_of_spaces(len: usize) -> String {
        unsafe { String::from_utf8_unchecked(vec![0x20; len]) }
    }

    #[cfg(test)]
    mod tests {
        use super::line_of_spaces;

        #[test]
        fn it_returns_a_string_of_spaces() {
            assert_eq!(line_of_spaces(0), "");
            assert_eq!(line_of_spaces(4), "    ");
        }
    }
}

pub mod byte_display {
    use std::io::Write;

    use termion::{clear, cursor};

    pub fn render<T: Write>(
        io: &mut T,
        scroll: usize,
        data: &[u8],
        bytes_per_row: usize,
        main_panel_height: u16,
    ) {
        let mut rows = data.chunks(bytes_per_row).skip(scroll);

        for i in 0..main_panel_height {
            if let Some(row) = rows.next() {
                write!(
                    io,
                    "{}{}{}",
                    cursor::Goto(1, (i + 1) as u16),
                    clear::CurrentLine,
                    format_row(row),
                ).unwrap();
            } else {
                write!(
                    io,
                    "{}{}",
                    cursor::Goto(1, (i + 1) as u16),
                    clear::CurrentLine
                ).unwrap();
            }
        }
    }

    fn format_row(row: &[u8]) -> String {
        row.iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(" ")
    }

    #[cfg(test)]
    mod tests {
        use super::format_row;

        #[test]
        fn it_renders_bytes_as_hex_strings() {
            let res = format_row(&[0, 1, 15, 255]);
            assert_eq!(res, "00 01 0F FF");
        }
    }
}
