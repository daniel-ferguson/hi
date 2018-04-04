use std::fmt;

#[derive(Debug)]
pub enum State {
    Wait,
    Prompt,
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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

pub struct Screen<'a> {
    pub state: State,
    pub frame: Frame,
    pub offset: usize,
    pub scroll: usize,
    pub bytes_per_row: usize,
    pub data_frame_dirty: bool,
    pub prompt_bar_dirty: bool,
    pub status_bar_dirty: bool,
    pub data: &'a [u8],
}

pub struct Point {
    pub x: u16,
    pub y: u16,
}

pub struct Dimension {
    pub width: u16,
    pub height: u16,
}

impl<'a> Screen<'a> {
    pub fn new(data: &'a [u8], frame: Frame) -> Screen {
        Screen {
            state: State::Wait,
            frame: frame,
            offset: 0,
            scroll: 0,
            bytes_per_row: 32,
            data: data,
            data_frame_dirty: false,
            prompt_bar_dirty: false,
            status_bar_dirty: false,
        }
    }

    fn status_bar_height() -> u16 {
        1
    }

    fn prompt_height() -> u16 {
        1
    }

    pub fn data_frame_height(&self) -> u16 {
        self.frame.height - Self::status_bar_height() - Self::prompt_height()
    }

    pub fn status_bar_position(&self) -> Point {
        Point {
            x: 1,
            y: self.frame.height - Self::prompt_height(),
        }
    }

    pub fn status_bar_dimensions(&self) -> Dimension {
        Dimension {
            width: self.frame.width,
            height: Self::status_bar_height(),
        }
    }

    pub fn left(&mut self) {
        if self.offset > 0 {
            self.data_frame_dirty = true;
            self.status_bar_dirty = true;

            self.offset -= 1;
        }
    }

    pub fn right(&mut self) {
        if self.offset <= self.data.len() {
            self.data_frame_dirty = true;
            self.status_bar_dirty = true;

            self.offset += 1;
        }
    }

    pub fn down(&mut self) {
        self.data_frame_dirty = true;
        self.status_bar_dirty = true;

        let data = &self.data[self.offset..self.data.len()];

        if self.scroll < max_scroll(self.data_frame_height() as usize, &data, self.bytes_per_row) {
            self.scroll += 1;
        }
    }

    pub fn up(&mut self) {
        self.data_frame_dirty = true;
        self.status_bar_dirty = true;

        if self.scroll > 0 {
            self.scroll -= 1;
        }
    }

    pub fn page_down(&mut self) {
        self.data_frame_dirty = true;
        self.status_bar_dirty = true;

        let len = self.data.len();
        let data = &self.data[self.offset..len];

        if (self.scroll + self.data_frame_height() as usize)
            < max_scroll(self.data_frame_height() as usize, &data, self.bytes_per_row)
        {
            self.scroll += self.data_frame_height() as usize;
        } else {
            self.scroll = max_scroll(self.data_frame_height() as usize, &data, self.bytes_per_row);
        }
    }

    pub fn page_up(&mut self) {
        self.data_frame_dirty = true;
        self.status_bar_dirty = true;

        if self.data_frame_height() as usize > self.scroll {
            self.scroll = 0;
        } else {
            self.scroll -= self.data_frame_height() as usize;
        }
    }

    pub fn start(&mut self) {
        self.data_frame_dirty = true;
        self.status_bar_dirty = true;

        self.scroll = 0;
    }

    pub fn end(&mut self) {
        self.data_frame_dirty = true;
        self.status_bar_dirty = true;

        let len = self.data.len();
        let data = &self.data[self.offset..len];

        self.scroll = max_scroll(self.data_frame_height() as usize, &data, self.bytes_per_row);

        self.scroll = 0;
    }

    pub fn prompt(&mut self) {
        self.prompt_bar_dirty = true;
        self.status_bar_dirty = true;
        self.state = State::Prompt;
    }

    pub fn reset_prompt(&mut self) {
        self.prompt_bar_dirty = true;
        self.status_bar_dirty = true;
        self.state = State::Wait;
    }

    pub fn update_prompt(&mut self) {
        self.prompt_bar_dirty = true;
        self.status_bar_dirty = true;
    }

    pub fn set_width(&mut self, width: usize) {
        self.data_frame_dirty = true;
        self.prompt_bar_dirty = true;
        self.status_bar_dirty = true;
        self.state = State::Wait;

        self.bytes_per_row = width;

        let anchor = top_left_byte_index(self.offset, self.scroll, self.bytes_per_row);

        let s = scroll_for_anchor(anchor, self.offset, self.bytes_per_row);
        let o = offset_for_anchor(anchor, self.offset, self.bytes_per_row);
        let (s, o) = balance_offset_and_scroll(s, o, self.bytes_per_row);

        self.scroll = s;
        self.offset = o;
    }

    pub fn set_offset(&mut self, offset: usize) {
        self.data_frame_dirty = true;
        self.prompt_bar_dirty = true;
        self.status_bar_dirty = true;
        self.state = State::Wait;

        self.offset = offset;
    }

    pub fn clear_dirty_flags(&mut self) {
        self.data_frame_dirty = false;
        self.prompt_bar_dirty = false;
        self.status_bar_dirty = false;
    }
}

fn max_scroll(height: usize, data: &[u8], width: usize) -> usize {
    let lines = data.len() / width as usize;
    if lines > height {
        lines - height / 2
    } else {
        0
    }
}

fn balance_offset_and_scroll(scroll: usize, offset: usize, bytes_per_row: usize) -> (usize, usize) {
    let scroll = scroll + offset / bytes_per_row;
    let offset = offset % bytes_per_row;
    (scroll, offset)
}

fn top_left_byte_index(offset: usize, scroll: usize, bytes_per_row: usize) -> usize {
    offset + scroll * bytes_per_row
}

fn scroll_for_anchor(anchor: usize, offset: usize, bytes_per_row: usize) -> usize {
    (anchor - offset) / bytes_per_row
}

fn offset_for_anchor(anchor: usize, offset: usize, bytes_per_row: usize) -> usize {
    offset + (anchor - offset) % bytes_per_row
}

#[cfg(test)]
mod tests {
    use super::*;

    mod max_scroll {
        use super::max_scroll;

        #[test]
        fn it_allows_scrolling_half_a_screen_past_end_of_data() {
            // data displayed across more rows than height
            let height = 60;
            let data: &[u8; 80] = &[0; 80];
            let width = 1;
            assert_eq!(max_scroll(height, data, width), 20 + height / 2);
        }

        #[test]
        fn it_disables_scroll_when_data_fits_on_one_screen() {
            // data displayed across fewer rows than height
            let height = 60;
            let data: &[u8; 20] = &[0; 20];
            let width = 1;
            assert_eq!(max_scroll(height, data, width), 0);
        }
    }
}