#[macro_use]
extern crate nom;

pub mod command_prompt;
pub mod context;
pub mod screen;
pub use crate::screen::Frame;
pub use crate::screen::State;

pub mod status_bar {
    use std::io::Write;

    use termion::{color, cursor};

    use super::screen::Screen;

    pub fn render<T: Write>(screen: &mut Screen<T>, path: &str) {
        let message_right = format!(
            "{}|o:{}|sy:{}|sx:{}|w:{}",
            &screen.state, screen.offset, screen.scroll_y, screen.scroll_x, screen.bytes_per_row
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

        write!(
            &mut screen.out,
            "{}{}",
            cursor::Goto(1, status_bar_position.y),
            bar_full,
        )
        .unwrap();
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

    use super::line::Line;
    use super::screen::Screen;
    use termion::{clear, cursor};

    pub fn render<T: Write>(screen: &mut Screen<T>, data: &[u8]) {
        use std::cmp;

        let scroll = screen.scroll_y;
        let bytes_per_row = screen.bytes_per_row;
        let main_panel_height = screen.data_frame_height();
        let mut rows = data.chunks(bytes_per_row).skip(scroll);

        let mut line = Line::new(screen.data_frame_width() as usize);

        for i in 0..main_panel_height {
            if let Some(row) = rows.next() {
                // construct and print a view into the row, which satisfies two conditions:
                //  * contains only as many bytes as can fit onto a line
                //  * is offset according to the scroll x property
                //
                // We constrain the end index to the length of the row to avoid causing a panic by
                // indexing too far, this is necessary in a number of cases: lines shorter than
                // max-width, scrolling far enough for blank space to be visible, etc.
                // Line's ::format method will produce properly formatted strings for short rows.
                //
                // We constrain the start index by the end index to ensure if we're ever told to
                // scroll past the end of the row (probably shouldn't happen) we don't blow up.
                let end = cmp::min(
                    screen.scroll_x + max_bytes(screen.data_frame_width()),
                    row.len(),
                );
                let start = cmp::min(screen.scroll_x, end);
                let view = &row[start..end];

                write!(
                    screen.out,
                    "{}{}",
                    cursor::Goto(1, (i + 1) as u16),
                    line.format(view),
                )
                .unwrap();
            } else {
                write!(
                    screen.out,
                    "{}{}",
                    cursor::Goto(1, (i + 1) as u16),
                    clear::CurrentLine
                )
                .unwrap();
            }
        }
    }

    /// Calculate number of bytes which can be displayed per line
    fn max_bytes(line_length: u16) -> usize {
        if line_length > 0 {
            (line_length as usize + 1) / 3
        } else {
            0
        }
    }

    #[cfg(test)]
    mod tests {
        use super::max_bytes;

        #[test]
        fn max_bytes_when_line_length_is_less_than_2() {
            assert_eq!(max_bytes(1), 0);
            assert_eq!(max_bytes(0), 0);
        }

        #[test]
        fn max_bytes_when_line_length_accounting_for_padding() {
            assert_eq!(max_bytes(2), 1);
            assert_eq!(max_bytes(3), 1);
            assert_eq!(max_bytes(4), 1);
            assert_eq!(max_bytes(5), 2);
            assert_eq!(max_bytes(6), 2);
            assert_eq!(max_bytes(7), 2);
            assert_eq!(max_bytes(8), 3);
        }
    }
}

pub mod line {
    const LOOKUP: [char; 16] = [
        '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F',
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
            let formatted_length = if bytes.len() == 0 {
                0
            } else {
                bytes.len() * 2 + bytes.len() - 1
            };

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
        fn format_works_when_given_an_empty_slice() {
            let mut line = Line::new(0);

            assert_eq!(line.format(&[]), "");
        }

        #[test]
        #[should_panic]
        fn format_panics_if_given_more_bytes_than_there_is_line_space() {
            Line::new(4).format(&[111, 222, 000]);
        }
    }
}
