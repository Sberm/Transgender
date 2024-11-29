/*═══════════════════════════════════════════════════════════════════════╗
║                         (C)  Howard Chu                                ║
║                                                                        ║
║ Permission to use, copy, modify, and/or distribute this software for   ║
║ any purpose with or without fee is hereby granted, provided that the   ║
║ above copyright notice and this permission notice appear in all copies ║
╚═══════════════════════════════════════════════════════════════════════*/

use crate::canvas;
use crate::ops::{code, consts, Mode};
use crate::util;
use regex_lite::RegexBuilder;
use std::fs::read_dir;
use std::iter::Rev;
use std::ops::Range;
use std::path::PathBuf;
use std::process::{exit, Command};
use std::vec::Vec;

/// Directory browser
pub struct Browser {
    cursor: usize,
    window_start: usize,
    current_dir: Vec<String>, // String instead of PathBuf for display purposes
    past_dir: Vec<PathBuf>,
    past_cursor: Vec<usize>,
    past_window_start: Vec<usize>,
    current_path: PathBuf,
    original_path: PathBuf,
    mode: Mode,
    search_txt: Vec<char>,
    has_search_input: bool,
    editor: String,
}

pub enum IterType {
    Forward(Range<usize>),
    Backward(Rev<Range<usize>>),
}

impl Iterator for IterType {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        match self {
            IterType::Forward(range) => range.next(),
            IterType::Backward(range) => range.next(),
        }
    }
}

impl Browser {
    /// Construct past directory stack according to the current path
    pub fn init(&mut self) {
        self.read_current_dir(&String::from("."));

        let mut srcdir = PathBuf::from(".")
            .canonicalize()
            .expect("Failed to canonicalize current directory");

        loop {
            self.past_dir.push(srcdir.clone());
            self.past_cursor.push(0);
            self.past_window_start.push(0);

            if !srcdir.pop() {
                break;
            }
        }

        self.past_dir.reverse();

        if self.past_dir.len() > 1 {
            self.current_path = self
                .past_dir
                .pop()
                .expect("Failed to pop the last element from past_dir")
                .clone();

            self.original_path = self.current_path.clone();

            self.past_cursor
                .pop()
                .expect("Failed to pop from past_cursor");

            self.past_window_start
                .pop()
                .expect("Failed to pop from past_window_start");
        }

        self.top();
    }

    /// Window display update loop
    pub fn start_loop(&mut self, canvas: &mut canvas::Canvas) {
        loop {
            let preview_dir = self.get_preview();
            canvas.draw(
                self.cursor,
                &self.current_dir,
                &preview_dir,
                self.window_start,
                &self.current_path,
                self.mode,
                &self.search_txt,
            );
            if matches!(self.mode, Mode::SEARCH) {
                self.search();
                continue;
            }
            match util::process_input() {
                code::UP => {
                    self.up();
                }
                code::DOWN => {
                    self.down();
                }
                code::LEFT => {
                    self.left();
                }
                code::RIGHT => {
                    self.right();
                }
                code::EXIT_CURSOR => {
                    self.exit_under_cursor();
                }
                code::EXIT => {
                    self.exit_cur_dir();
                }
                code::QUIT => {
                    self.quit();
                }
                code::TOP => {
                    self.top();
                }
                code::BOTTOM => {
                    self.bottom();
                }
                code::SEARCH => {
                    self.search_txt = Vec::new();
                    self.has_search_input = false;
                    self.mode = Mode::SEARCH;
                }
                code::NEXT_MATCH => {
                    self.next_match(
                        if self.cursor + 1 < self.current_dir.len() {
                            self.cursor + 1
                        } else {
                            0
                        },
                        false,
                    );
                }
                code::PREV_MATCH => {
                    self.next_match(
                        if self.cursor as isize - 1 >= 0 {
                            self.cursor - 1
                        } else {
                            self.current_dir.len() - 1
                        },
                        true,
                    );
                }
                _ => {
                    continue;
                }
            }
        }
    }

    ///  Get directory content preview window as a vector of strings
    fn get_preview(&self) -> Vec<String> {
        let empty: Vec<String> = Vec::new();

        if self.current_dir.len() == 0 {
            return empty;
        }

        let mut _dir = self.current_path.clone();
        _dir.push(&self.current_dir[self.cursor]);
        if _dir.is_dir() == false {
            return empty;
        }

        let dir = _dir.to_str().expect("Failed to construct preview path");

        let mut preview = read_dir(&dir)
            .expect(&format!("Failed to read files from {}", dir))
            .map(|e| {
                e.expect("Failed to get preview entry")
                    .file_name()
                    .into_string()
                    .expect("Failed to get preview file name")
            })
            .collect::<Vec<String>>();

        preview.sort_by(|d1, d2| d1.to_lowercase().cmp(&d2.to_lowercase()));

        preview
    }

    ///  Set cursor position, centered in the window
    fn set_cursor_pos_centered(&mut self, index: usize) {
        let (h, _) = util::term_size();

        self.cursor = index;
        self.window_start = if self.cursor as isize - h as isize / 2 > 0 {
            self.cursor - h / 2
        } else {
            0
        };
    }

    /// Next search match, can be a reversed search
    fn next_match(&mut self, start: usize, rev: bool) {
        if self.has_search_input == false {
            return;
        }

        let mut matched = false;
        let mut case_insensitive = true;

        if self.cursor >= self.current_dir.len() {
            return;
        }

        let mut search: String = self.search_txt.iter().collect::<String>();

        // Check if the case sensitive '\C' is present at the bottom of the search text
        let len = self.search_txt.iter().count();

        // TODO: support "/ok\\C" backslash escape
        if len > 2 {
            let last_two = self
                .search_txt
                .iter()
                .skip(len - 2)
                .take(2)
                .collect::<String>();
            if last_two.eq("\\C") {
                search = self.search_txt.iter().take(len - 2).collect::<String>();
                case_insensitive = false;
            }
            if last_two.eq("\\c") {
                search = self.search_txt.iter().take(len - 2).collect::<String>();
                case_insensitive = true;
            }
        }

        // Regex can be invalid while the user is typing
        let re = match RegexBuilder::new(&search)
            .case_insensitive(case_insensitive)
            .build()
        {
            Ok(re) => re,
            Err(_) => RegexBuilder::new("^$")
                .build()
                .expect("Failed to parse regex for ^$"),
        };

        let it1 = if rev == false {
            IterType::Forward(start..self.current_dir.len())
        } else {
            IterType::Backward((0..start + 1).rev())
        };

        for i in it1 {
            if re.is_match(&self.current_dir[i]) {
                self.set_cursor_pos_centered(i);
                matched = true;
                break;
            }
        }

        // start from 0
        if matched == false {
            let it2 = if rev == false {
                IterType::Forward(0..start)
            } else {
                IterType::Backward((start + 1..self.current_dir.len()).rev())
            };

            for i in it2 {
                if re.is_match(&self.current_dir[i]) {
                    self.set_cursor_pos_centered(i);
                    break;
                }
            }
        }
    }

    fn search(&mut self) {
        let (rc, is_ascii) = match util::read_utf8() {
            Some((rc, is_ascii)) => (rc, is_ascii),
            None => ('�', false),
        };

        // there should be no search input when trans first starts
        self.has_search_input = true;

        if is_ascii {
            // esc
            if rc as u8 == 27 {
                self.mode = Mode::NORMAL;
                return;
            }

            // backspace
            if rc as u8 == 127 {
                if self.search_txt.len() > 0 {
                    self.search_txt.pop().expect("search txt(pop) out of bound");
                }
                return;
            }

            // enter
            if rc as u8 == 10 {
                // search
                self.mode = Mode::NORMAL;
                return;
            }
        }

        self.search_txt.push(rc);
        self.next_match(self.cursor, false);
    }

    fn top(&mut self) {
        self.cursor = 0;
        self.window_start = 0;
    }

    fn bottom(&mut self) {
        self.set_cursor_pos_centered(self.current_dir.len() - 1);
    }

    fn up(&mut self) {
        if self.current_dir.is_empty() == true {
            return;
        }

        self.cursor = if self.cursor as isize - 1 >= 0 {
            self.cursor - 1
        } else {
            0
        };

        if self.cursor < self.window_start {
            self.window_start -= 1;
        }
    }

    fn down(&mut self) {
        if self.current_dir.is_empty() == true {
            return;
        }

        let max_len = self.current_dir.len();
        self.cursor = if self.cursor + 1 < max_len {
            self.cursor + 1
        } else {
            max_len - 1
        };

        let (h, _) = util::term_size();

        // So the cursor won't be covered by the bottom line (TODO: But trans still draws that line
        // in Canvas)
        let max_display_height = h - 1;

        if (self.cursor) as isize > (self.window_start + max_display_height - 1) as isize {
            self.window_start += 1;
        }
    }

    fn left(&mut self) {
        let child = self.current_path.clone();
        // for example, root dir '/'
        if child.file_name() == None {
            return;
        }

        // get the parent dir and read its content
        self.current_path = self
            .past_dir
            .pop()
            .expect("Failed to pop from past_dir in when exiting a directory");
        self.read_current_dir(&self.current_path.to_str().unwrap().to_string());

        self.cursor = self
            .past_cursor
            .pop()
            .expect("Failed to pop from past_cursor");

        self.window_start = self
            .past_window_start
            .pop()
            .expect("Failed to pop from past_window_start");

        let child_filename_str = child
            .file_name()
            .expect("Failed to get file name of current path to restore the directory")
            .to_str()
            .expect("Failed to do to_str()");

        // 0 is set in init()
        let mut index: usize = 0;

        // find the child dir in parent
        for (i, dir) in self.current_dir.iter().enumerate() {
            if dir.eq(child_filename_str) {
                self.cursor = i;
                index = i;
                break;
            }
        }

        self.set_cursor_pos_centered(index);
    }

    fn right(&mut self) {
        if self.current_dir.len() <= 0 {
            return;
        }

        let mut dir_under_cursor = self.current_path.clone();
        dir_under_cursor.push(&self.current_dir[self.cursor]);
        if dir_under_cursor.is_dir() == false {
            return;
        }

        self.past_dir.push(self.current_path.clone());
        self.past_cursor.push(self.cursor);
        self.past_window_start.push(self.window_start);
        self.current_path = dir_under_cursor.clone();
        self.read_current_dir(
            &dir_under_cursor
                .to_str()
                .expect("Failed to call to_str() for the dir under cursor")
                .to_string(),
        );
        self.top();
    }

    /// Read the file and directory names in the current directory
    fn read_current_dir(&mut self, path: &String) {
        self.current_dir = read_dir(path)
            .expect(&format!("Failed to read files from {}", path))
            .map(|_e| {
                let e = _e.expect(&format!("Failed to read a file from {}", path));
                match e.file_name().into_string() {
                    Ok(filename) => filename,
                    Err(_) => e.file_name().to_string_lossy().into_owned(),
                }
            })
            .collect::<Vec<String>>();

        self.current_dir
            .sort_by(|d1, d2| d1.to_lowercase().cmp(&d2.to_lowercase()));
    }

    /// Goto the directory in the left side window, while quitting trans
    fn exit_cur_dir(&self) {
        util::exit_albuf();
        util::print_path(&self.current_path.to_str().unwrap());
        exit(0);
    }

    /// Goto the directory under the cursor, while quitting trans
    ///  or
    /// Open the file under the cursor with a text editor
    fn exit_under_cursor(&self) {
        let mut dir = self.current_path.clone();
        dir.push(&self.current_dir[self.cursor]);

        if dir.is_dir() == false {
            // reduce color flicking (caused by bottom bar color)
            util::reduce_flick();

            if let Ok(_) = Command::new(&self.editor)
                .arg(dir.to_str().unwrap())
                .status()
            {
                // empty, successfully opened with user's desired editor
            } else {
                Command::new(consts::EDITOR)
                    .arg(dir.to_str().unwrap())
                    .status()
                    .expect(&format!(
                        "Failed to open {} with default editor {}",
                        dir.to_str().unwrap(),
                        consts::EDITOR
                    ));
            }
        } else {
            util::exit_albuf();
            util::print_path(dir.to_str().unwrap());

            exit(0);
        };

        // when an editor exits, it also exits the alternate buffer, and enables cursor, need to
        // stay in albuf and hide cursor in trans
        util::enter_albuf();
        util::hide_cursor();
    }

    fn quit(&self) {
        util::exit_albuf();
        util::print_path(&self.original_path.to_str().unwrap());
        exit(0);
    }
}

pub fn new() -> Browser {
    let mut browser = Browser {
        cursor: 0,
        window_start: 0,
        current_dir: Vec::new(),
        past_dir: Vec::new(),
        past_cursor: Vec::new(),
        past_window_start: Vec::new(),
        current_path: PathBuf::from(""),
        original_path: PathBuf::from(""),
        mode: Mode::NORMAL,
        search_txt: Vec::new(),
        has_search_input: false,
        editor: util::get_editor(),
    };

    browser.init();
    browser
}
