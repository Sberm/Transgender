/*═══════════════════════════════════════════════════════════════════════╗
║                          ©  Howard Chu                                 ║
║                                                                        ║
║ Permission to use, copy, modify, and/or distribute this software for   ║
║ any purpose with or without fee is hereby granted, provided that the   ║
║ above copyright notice and this permission notice appear in all copies ║
╚═══════════════════════════════════════════════════════════════════════*/

extern crate libc;

use self::libc::{
    c_ushort, ioctl, tcgetattr, tcsetattr, termios, ECHO, ICANON, ISIG, STDIN_FILENO,
    STDOUT_FILENO, TCSAFLUSH, TIOCGWINSZ,
};
use crate::ops::{code, consts, Theme};
use std::env::var;
use std::fs::File;
use std::io::{self, stdin, BufRead, Read, Write};
use std::mem;
use std::path::Path;
use std::str::from_utf8;
use std::thread::sleep;
use std::time::Duration;

#[inline(always)]
pub fn hide_cursor() {
    print!("\x1b[?25l"); // hide cursor
}

#[inline(always)]
pub fn show_cursor() {
    print!("\x1b[?25h"); // show cursor
}

pub fn set_search_cursor() {
    let (h, _) = term_size();
    print!("\x1b[{}H", h);
    show_cursor();
    let _ = io::stdout().flush();
}

pub fn reset_search_cursor() {
    print!("\x1b[1H");
    hide_cursor();
    let _ = io::stdout().flush();
}

#[allow(dead_code)]
struct TermSize {
    height: c_ushort,
    width: c_ushort,
    a: c_ushort,
    b: c_ushort,
}

/// Get the height and width of the current terminal window
///
/// returns
///  tuple of (height, width)
pub fn term_size() -> (usize, usize) {
    unsafe {
        let mut sz: TermSize = mem::zeroed();
        ioctl(STDOUT_FILENO, TIOCGWINSZ.into(), &mut sz as *mut _);
        (sz.height as usize, sz.width as usize)
    }
}

#[allow(dead_code)]
pub fn slp(i: u64) {
    sleep(Duration::from_secs(i));
}

pub fn raw_input() {
    unsafe {
        let mut termios_: termios = mem::zeroed();
        tcgetattr(STDIN_FILENO, &mut termios_);
        termios_.c_lflag &= !(ECHO | ICANON | ISIG);
        tcsetattr(STDIN_FILENO, TCSAFLUSH, &termios_);
    }
}

pub fn canonical_input() {
    unsafe {
        let mut termios_: termios = mem::zeroed();
        tcgetattr(STDIN_FILENO, &mut termios_);
        termios_.c_lflag |= ECHO | ICANON | ISIG;
        tcsetattr(STDIN_FILENO, TCSAFLUSH, &termios_);
    }
}

pub fn enter_albuf() {
    raw_input();
    hide_cursor();
    print!("\x1b[?1049h"); // use alternate buffer
    let _ = io::stdout().flush();
}

pub fn exit_albuf() {
    canonical_input();
    show_cursor();
    print!("\x1b[?1049l"); // switch back to normal screen buffer
    let _ = io::stdout().flush();
}

/// Read a single ascii byte input
///
/// returns
///  ascii byte
fn read_input() -> isize {
    let mut stdin_handle = stdin().lock();
    let mut byte = [0_u8];
    stdin_handle
        .read_exact(&mut byte)
        .expect("Failed to read single byte");
    byte[0] as isize
}

pub fn process_input() -> u8 {
    let mut input = read_input();

    // arrow keys
    if input == 27 {
        input = read_input();
        if input == 91 {
            input = read_input();
            if input == 65 {
                return code::UP;
            } else if input == 66 {
                return code::DOWN;
            } else if input == 67 {
                return code::RIGHT;
            } else if input == 68 {
                return code::LEFT;
            } else {
                return code::NOOP;
            }
        } else {
            return code::NOOP;
        }
    }

    // gg
    if input == 103 {
        input = read_input();
        if input == 103 {
            return code::TOP;
        }
    }

    match input {
        107 => return code::UP,          // k
        106 => return code::DOWN,        // j
        104 => return code::LEFT,        // h
        108 => return code::RIGHT,       // l
        111 => return code::EXIT_CURSOR, // o
        10 => return code::EXIT_CURSOR,  // Enter
        105 => return code::EXIT,        // i
        113 => return code::QUIT,        // q
        47 => return code::SEARCH,       // /
        71 => return code::BOTTOM,       // G
        110 => return code::NEXT_MATCH,  // n
        78 => return code::PREV_MATCH,   // N
        _ => return code::NOOP,
    }
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

/// Print path to stderr (although stdin and stdout are switched in ts shell function) for cd to
/// consume.
pub fn print_path(str_: &str) {
    eprintln!("\n{}", str_);
}

pub fn get_theme() -> Theme {
    if let Ok(home_dir) = var(consts::HOME_VAR) {
        if let Ok(lines) = read_lines(&format!("{}/{}", home_dir, consts::CONFIG_FILE)) {
            for line in lines.flatten() {
                let trimmed = line.replace(" ", "");

                let kv = trimmed.split("=").collect::<Vec<&str>>();
                if kv.len() != 2 {
                    continue;
                }
                if kv[0].eq(consts::THEME_KEY) && kv[1].eq(consts::THEME_DARK) {
                    return Theme::DARK;
                }
            }
        }
    }
    return Theme::TRANS;
}

/// Read trans config file to get preferred editor
///
/// returns
///  editor name as a String
pub fn get_editor() -> String {
    if let Ok(home_dir) = var(consts::HOME_VAR) {
        if let Ok(lines) = read_lines(&format!("{}/{}", home_dir, consts::CONFIG_FILE)) {
            for line in lines.flatten() {
                let trimmed = line.replace(" ", "");

                let kv = trimmed.split("=").collect::<Vec<&str>>();
                if kv.len() != 2 {
                    continue;
                }
                if kv[0].eq(consts::EDITOR_KEY) {
                    return String::from(kv[1]);
                }
            }
        }
    }
    return String::from(consts::EDITOR);
}

///  Read a single utf8 char
///
/// returns
///  A tuple of char and bool
pub fn read_utf8() -> Option<(char, bool)> {
    let mut c_bytes = [0u8; 4];
    let mut bytes_cnt: usize = 0;

    stdin()
        .read(&mut c_bytes[0..1])
        .expect("Failed to read the UTF8 prefix");

    if c_bytes[0] & 0b10000000 == 0 {
        bytes_cnt = 1;
    } else if c_bytes[0] & 0b11000000 == 0b11000000 && c_bytes[0] & 0b00100000 == 0 {
        bytes_cnt = 2;
        stdin()
            .read(&mut c_bytes[1..2])
            .expect("Failed to read 1 byte for UTF8 char");
    } else if c_bytes[0] & 0b11100000 == 0b11100000 && c_bytes[0] & 0b00010000 == 0 {
        bytes_cnt = 3;
        stdin()
            .read(&mut c_bytes[1..3])
            .expect("Failed to read 2 bytes for UTF8 char");
    } else if c_bytes[0] & 0b11110000 == 0b11110000 && c_bytes[0] & 0b00001000 == 0 {
        bytes_cnt = 4;
        stdin()
            .read(&mut c_bytes[1..])
            .expect("Failed to read 3 bytes for UTF8 char");
    }

    let is_ascii: bool = if bytes_cnt == 1 { true } else { false };

    let s = String::from(
        from_utf8(&c_bytes[0..bytes_cnt]).expect("Failed to convert bytes string to &str"),
    );

    let rc = s
        .chars()
        .nth(0)
        .expect("Failed to get the first & only character");

    Some((rc, is_ascii))
}
