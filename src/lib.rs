extern crate termion;

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

pub struct Properties {
    pub scroll: usize,
    pub cursor: Cursor,
}


pub mod status_bar {
    use std::io::Write;

    use termion::{color, cursor};

    use super::{Frame, State};

    pub fn render<T: Write>(io: &mut T, frame: &Frame, path: &str, state: &State) {
        let message_right = format!("{}", state);
        let bar = unsafe { String::from_utf8_unchecked(vec![0x20; frame.width as usize]) };

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
}

pub mod byte_display {
    use std::io::Write;

    use termion::{clear, cursor};

    use super::Properties;

    pub fn render<T: Write>(
        io: &mut T,
        props: &mut Properties,
        data: &mut [u8],
        bytes_per_row: usize,
        main_panel_height: u16,
    ) {
        let scroll = props.scroll;

        let mut rows = data.chunks(bytes_per_row).skip(scroll);

        for i in 0..main_panel_height {
            if let Some(row) = rows.next() {
                write!(
                    io,
                    "{}{}{}",
                    cursor::Goto(1, (i + 1) as u16),
                    clear::CurrentLine,
                    format_row(row, bytes_per_row),
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

    fn format_row(row: &[u8], max_row_length: usize) -> String {
        let remainder = max_row_length - row.len();
        let mut s = row.iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(" ");

        for _ in 0..remainder {
            s.push_str("   ");
        }

        s
    }
}
