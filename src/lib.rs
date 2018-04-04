#[macro_use]
extern crate log;
#[macro_use]
extern crate nom;
extern crate termion;

pub mod command_prompt;
pub mod screen;
pub use screen::Frame;
pub use screen::State;

pub mod status_bar {
    use std::io::Write;

    use termion::{color, cursor};

    use super::screen::Screen;

    pub fn render<T: Write>(io: &mut T, screen: &Screen, path: &str) {
        let message_right = format!(
            "{}|o:{}|s:{}|w:{}",
            &screen.state, screen.offset, screen.scroll, screen.bytes_per_row
        );
        let bar = line_of_spaces(screen.status_bar_dimensions().width as usize);

        let status_bar_position = screen.status_bar_position();

        let bar_full = format!(
            "{}{}{}{}{}{}{}{}{}",
            color::Bg(color::Black),
            color::Fg(color::White),
            bar,
            cursor::Goto(status_bar_position.x, status_bar_position.y),
            path,
            cursor::Goto(
                (bar.len() - message_right.len()) as u16,
                status_bar_position.y
            ),
            message_right,
            color::Bg(color::Reset),
            color::Fg(color::Reset),
        );

        write!(io, "{}{}", cursor::Goto(1, status_bar_position.y), bar_full,).unwrap();
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

pub mod line {
    const LOOKUP: [char; 16] = [
        '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F'
    ];

    pub struct Line {
        length: usize,
        text: String,
    }

    impl Line {
        pub fn new(length: usize) -> Line {
            Line {
                length: length,
                text: String::with_capacity(length),
            }
        }

        pub fn format(&mut self, bytes: &[u8]) -> &str {
            let formatted_length = bytes.len() * 2 + bytes.len() - 1;
            assert!(formatted_length <= self.length);

            self.text.clear();

            // translate bytes into hex representations
            for (i, byte) in bytes.iter().enumerate() {
                self.text.push(LOOKUP[(byte >> 4) as usize]);
                self.text.push(LOOKUP[(byte & 0xF) as usize]);
                if i < bytes.len() - 1 {
                    self.text.push(' ');
                }
            }

            // pad string with spaces
            for _ in 0..(self.length - formatted_length) {
                self.text.push(' ');
            }

            &self.text
        }
    }

    #[cfg(test)]
    mod tests {
        use super::Line;

        #[test]
        fn format_represents_bytes_as_hex_values() {
            let mut line = Line::new(2);

            assert_eq!(line.format(&[129]), "81");
        }

        #[test]
        fn format_inserts_spaces_between_values() {
            let mut line = Line::new(5);

            assert_eq!(line.format(&[129, 0]), "81 00");
        }

        #[test]
        fn format_pads_line_with_spaces() {
            let mut line = Line::new(10);

            assert_eq!(line.format(&[129, 0]), "81 00     ");
        }

        #[test]
        #[should_panic]
        fn format_panics_if_given_more_bytes_than_there_is_line_space() {
            Line::new(4).format(&[111, 222, 000]);
        }
    }
}
