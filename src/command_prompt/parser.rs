use nom;
use nom::digit;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
enum CommandName {
    Offset,
    ScrollX,
    ScrollY,
    Width,
}

#[derive(Debug)]
pub struct CommandParseError;

impl FromStr for CommandName {
    type Err = CommandParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "o" | "offset" => Ok(CommandName::Offset),
            "w" | "width" => Ok(CommandName::Width),
            "x" | "scrollx" => Ok(CommandName::ScrollX),
            "y" | "scrolly" => Ok(CommandName::ScrollY),
            _ => Err(CommandParseError),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Command {
    ScrollX(usize),
    ScrollY(usize),
    SetOffset(usize),
    SetWidth(usize),
}

named!(
    whitespace,
    alt!(tag!(" ") | tag!("\t") | tag!("\r") | tag!("\n"))
);

named!(numeric_string<&str>, map_res!(digit, ::std::str::from_utf8));

named!(
    usize_digit<usize>,
    map_res!(numeric_string, ::std::str::FromStr::from_str)
);

named!(command_name<&[u8], CommandName>,
       do_parse!(
           many0!(whitespace)                              >>
           n: map_res!(
               map_res!(nom::alpha, ::std::str::from_utf8),
               CommandName::from_str
           )                                               >>
           ({ n })
           )
       );

named!(command_offset<&[u8], Command>,
       do_parse!(
           many0!(whitespace)            >>
           arg1: usize_digit             >>
           many0!(whitespace)            >>
           eof!()                        >>
           ({ Command::SetOffset(arg1) })
           )
       );

named!(command_width<&[u8], Command>,
       do_parse!(
           many0!(whitespace)            >>
           arg1: usize_digit             >>
           many0!(whitespace)            >>
           eof!()                        >>
           ({ Command::SetWidth(arg1) })
           )
       );

named!(command_scroll_x<&[u8], Command>,
       do_parse!(
           many0!(whitespace)            >>
           arg1: usize_digit             >>
           many0!(whitespace)            >>
           eof!()                        >>
           ({ Command::ScrollX(arg1) })
           )
       );

named!(command_scroll_y<&[u8], Command>,
       do_parse!(
           many0!(whitespace)            >>
           arg1: usize_digit             >>
           many0!(whitespace)            >>
           eof!()                        >>
           ({ Command::ScrollY(arg1) })
           )
       );

named!(pub command<&[u8], Command>,
       switch!(command_name,
               CommandName::Offset => complete!(command_offset) |
               CommandName::ScrollX => complete!(command_scroll_x) |
               CommandName::ScrollY => complete!(command_scroll_y) |
               CommandName::Width => complete!(command_width)
               )
       );

pub fn parse_command(s: &str) -> Result<Command, CommandParseError> {
    match command(s.as_bytes()) {
        ::nom::IResult::Done(_, parsed) => Ok(parsed),
        ::nom::IResult::Error(..) | ::nom::IResult::Incomplete(..) => Err(CommandParseError),
    }
}

#[cfg(test)]
mod tests {
    // Wraps nom's IResult but ignores remaining data stored in Done
    // to make loose matching a little less effort when using the
    // assert_parse_ok macro
    #[derive(Debug, PartialEq)]
    enum ParseResult<T, E, I> {
        Done(T),
        Error(E),
        Incomplete(I),
    }

    macro_rules! assert_parse_ok {
        ($method:expr, $result:expr, [$($bytes:expr),*]) => {
            $(
                let res = match $method($bytes) {
                    ::nom::IResult::Done(_, parsed) => super::ParseResult::Done(parsed),
                    ::nom::IResult::Error(e) => super::ParseResult::Error(e),
                    ::nom::IResult::Incomplete(i) => super::ParseResult::Incomplete(i),
                };

                assert_eq!(res, super::ParseResult::Done($result));
            )*
        };
        ($method:expr, $result:expr, [$($bytes:expr),*,]) => {
            $(
                let res = match $method($bytes) {
                    ::nom::IResult::Done(_, parsed) => super::ParseResult::Done(parsed),
                    ::nom::IResult::Error(e) => super::ParseResult::Error(e),
                    ::nom::IResult::Incomplete(i) => super::ParseResult::Incomplete(i),
                };

                assert_eq!(res, super::ParseResult::Done($result));
            )*
        };
    }

    macro_rules! assert_parse_any_error {
        ($method:expr, [ $($bytes:expr),+ ]) => {
            $(
                let res = $method($bytes);
                assert!(
                    match res {
                        ::nom::IResult::Error(_) => true,
                        _ => false
                    }
                );
            )*
        };
        ($method:expr, [ $($bytes:expr),+, ]) => {
            $(
                let res = $method($bytes);
                assert!(
                    match res {
                        ::nom::IResult::Error(_) => true,
                        _ => false
                    }
                );
            )*
        }
    }

    use super::*;
    mod command {
        use super::{command, command_name, usize_digit};
        use super::{Command, CommandName};

        #[test]
        fn parsing_commands() {
            assert_parse_ok!(command, Command::SetOffset(32), [b"offset 32", b"o 32"]);
            assert_parse_ok!(command, Command::SetOffset(0), [b"offset 0", b"o  0"]);
            assert_parse_ok!(command, Command::SetWidth(32), [b"width 32", b"w 32"]);
            assert_parse_ok!(command, Command::SetWidth(0), [b"width 0", b"w  0"]);
            assert_parse_ok!(command, Command::ScrollX(0), [b"scrollx 0", b"x  0"]);
            assert_parse_ok!(command, Command::ScrollY(0), [b"scrolly 0", b"y  0"]);
            assert_parse_any_error!(command, [b"wdith 3", b"width", b"wid"]);
            assert_parse_any_error!(command, [b"offest 3", b"offset", b"offse"]);
        }

        #[test]
        fn parsing_command_names() {
            assert_parse_ok!(
                command_name,
                CommandName::Width,
                [
                    b"w",
                    b"width",
                    b" width",
                    b"\twidth",
                    b"\nwidth",
                    b"\r\nwidth",
                    b" \r \n width",
                    b"\tw",
                    b"\nw",
                    b"\r\nw",
                    b" \r \n w",
                ]
            );
            assert_parse_ok!(
                command_name,
                CommandName::Offset,
                [
                    b"o",
                    b"offset",
                    b" offset",
                    b"\toffset",
                    b"\noffset",
                    b"\r\noffset",
                    b" \r \n offset",
                    b"\to",
                    b"\no",
                    b"\r\no",
                    b" \r \n o",
                ]
            );

            assert_parse_any_error!(command_name, [b"unknown"]);
        }

        #[test]
        fn parsing_usize_digits() {
            assert_parse_ok!(usize_digit, 123usize, [b"123", b"123 "]);

            assert_parse_ok!(usize_digit, 0usize, [b"0", b"0 "]);
        }
    }
}
