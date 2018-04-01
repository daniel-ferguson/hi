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
