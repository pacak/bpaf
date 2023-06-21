use std::ops::{Index, IndexMut};
use vte::ansi::{Handler, LineClearMode, Processor, Timeout};

pub struct Term {
    vterm: VTerm,
    parser: Processor<TestSyncHandler>,
}

#[derive(Debug)]
struct VTerm {
    screen: Vec<char>,
    width: usize,
    height: usize,
    x: usize,
    y: usize,
}

impl Index<(usize, usize)> for VTerm {
    type Output = char;
    fn index(&self, (y, x): (usize, usize)) -> &Self::Output {
        &self.screen[self.cursor(y, x)]
    }
}

impl IndexMut<(usize, usize)> for VTerm {
    fn index_mut(&mut self, (y, x): (usize, usize)) -> &mut Self::Output {
        let cursor = self.cursor(y, x);
        &mut self.screen[cursor]
    }
}

impl Term {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            parser: Processor::new(),
            vterm: VTerm::new(width, height),
        }
    }

    pub fn process(&mut self, buf: &[u8]) {
        for b in buf {
            self.parser.advance(&mut self.vterm, *b);
        }
    }

    pub fn render(&mut self) -> String {
        self.vterm.render()
    }
}

impl Handler for VTerm {
    fn input(&mut self, c: char) {
        let cursor = self.cur();
        self[cursor] = c;
        self.x += 1;
        assert!(self.x <= self.width);
    }

    fn goto(&mut self, line: i32, col: usize) {
        self.y = line.try_into().unwrap();
        self.x = col;
    }

    fn goto_line(&mut self, line: i32) {
        self.y = line.try_into().unwrap();
    }

    fn goto_col(&mut self, col: usize) {
        self.x = col;
    }

    fn insert_blank(&mut self, _: usize) {
        todo!();
    }

    fn move_up(&mut self, value: usize) {
        self.y = self.y.saturating_sub(value);
    }

    fn move_down(&mut self, value: usize) {
        self.y += value;
    }

    fn identify_terminal(&mut self, _intermediate: Option<char>) {
        todo!();
    }

    fn device_status(&mut self, _: usize) {
        todo!();
    }

    fn move_forward(&mut self, col: usize) {
        self.x += col;
        assert!(self.x <= self.width);
    }

    fn move_backward(&mut self, col: usize) {
        self.x = self.x.saturating_sub(col);
    }

    fn move_down_and_cr(&mut self, _row: usize) {
        self.y += 1;
        self.x = 0;
    }

    fn move_up_and_cr(&mut self, _row: usize) {
        self.y = self.y.saturating_sub(1);
        self.x = 0;
    }

    fn put_tab(&mut self, _count: u16) {
        // do this for bash, for some reason it refuses
        // to send anything if echo is disabled, but with
        // it enabled we also get the command I type followed
        // by a tab...
        assert_eq!(self.y, 0);
        self.x = 0;
        self.y = 0;
    }

    fn backspace(&mut self) {
        self.x = self.x.saturating_sub(1);
    }

    fn carriage_return(&mut self) {
        self.x = 0;
    }

    fn linefeed(&mut self) {
        self.y += 1;
    }

    fn substitute(&mut self) {
        todo!();
    }

    fn newline(&mut self) {
        todo!();
    }

    fn set_horizontal_tabstop(&mut self) {
        todo!();
    }

    fn scroll_up(&mut self, _: usize) {
        todo!();
    }

    fn scroll_down(&mut self, _: usize) {
        todo!();
    }

    fn insert_blank_lines(&mut self, _: usize) {
        todo!();
    }

    fn delete_lines(&mut self, _: usize) {
        todo!();
    }

    fn erase_chars(&mut self, _: usize) {
        todo!();
    }

    fn delete_chars(&mut self, _: usize) {
        todo!();
    }

    fn move_backward_tabs(&mut self, _count: u16) {
        todo!();
    }

    fn move_forward_tabs(&mut self, _count: u16) {
        todo!();
    }

    fn save_cursor_position(&mut self) {
        todo!();
    }

    fn restore_cursor_position(&mut self) {
        todo!();
    }

    fn clear_line(&mut self, _mode: vte::ansi::LineClearMode) {
        let range = match _mode {
            LineClearMode::Right => self.x..self.width,
            LineClearMode::Left => todo!(),
            LineClearMode::All => todo!(),
        };
        let y = self.y;
        for x in range {
            self[(y, x)] = ' ';
        }
    }

    fn clear_screen(&mut self, _mode: vte::ansi::ClearMode) {
        println!("clear screen {:?} of {:?}", _mode, self.cur());
    }

    fn clear_tabs(&mut self, _mode: vte::ansi::TabulationClearMode) {
        todo!();
    }

    fn reset_state(&mut self) {
        todo!();
    }

    fn reverse_index(&mut self) {
        //println!("RI at {:?}", self.cur());
        assert!(self.y > 0);
        //        self.y -= 1;
        self.y = self.y.saturating_sub(1);
    }

    fn terminal_attribute(&mut self, _attr: vte::ansi::Attr) {}

    fn set_mode(&mut self, _mode: vte::ansi::Mode) {}

    fn unset_mode(&mut self, _mode: vte::ansi::Mode) {}

    fn set_scrolling_region(&mut self, _top: usize, _bottom: Option<usize>) {
        todo!();
    }

    fn set_keypad_application_mode(&mut self) {}

    fn unset_keypad_application_mode(&mut self) {}

    fn set_active_charset(&mut self, _: vte::ansi::CharsetIndex) {}

    fn configure_charset(&mut self, _: vte::ansi::CharsetIndex, _: vte::ansi::StandardCharset) {}

    fn set_color(&mut self, _: usize, _: vte::ansi::Rgb) {
        todo!();
    }

    fn dynamic_color_sequence(&mut self, _: String, _: usize, _: &str) {
        todo!();
    }

    fn reset_color(&mut self, _: usize) {
        todo!();
    }

    fn set_title(&mut self, _: Option<String>) {}

    fn set_cursor_style(&mut self, _: Option<vte::ansi::CursorStyle>) {}

    fn set_cursor_shape(&mut self, _shape: vte::ansi::CursorShape) {}

    fn bell(&mut self) {}

    fn clipboard_store(&mut self, _: u8, _: &[u8]) {}

    fn clipboard_load(&mut self, _: u8, _: &str) {}

    fn decaln(&mut self) {}

    fn push_title(&mut self) {}

    fn pop_title(&mut self) {}

    fn text_area_size_pixels(&mut self) {}

    fn text_area_size_chars(&mut self) {}

    fn set_hyperlink(&mut self, _: Option<vte::ansi::Hyperlink>) {}
}

impl VTerm {
    pub fn new(width: u16, height: u16) -> Self {
        let screen = vec![' '; usize::from(width) * usize::from(height)];
        VTerm {
            screen,
            width: width.into(),
            height: height.into(),
            x: 0,
            y: 0,
        }
    }

    fn cur(&self) -> (usize, usize) {
        (self.y, self.x)
    }

    fn cursor(&self, y: usize, x: usize) -> usize {
        self.width * y + x
    }

    pub fn render(&self) -> String {
        let mut res = String::new();
        for y in 0..self.height {
            for x in 0..self.width {
                res.push(self[(y, x)]);
            }
            res.truncate(res.trim_end().len());
            res.push('\n');
        }
        res.truncate(res.trim_end().len());
        res
    }
}
#[derive(Default)]
pub struct TestSyncHandler;

impl Timeout for TestSyncHandler {
    #[inline]
    fn set_timeout(&mut self, _: std::time::Duration) {
        unreachable!()
    }

    #[inline]
    fn clear_timeout(&mut self) {
        unreachable!()
    }

    #[inline]
    fn pending_timeout(&self) -> bool {
        false
    }
}
