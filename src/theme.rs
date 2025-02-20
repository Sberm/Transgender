/*═══════════════════════════════════════════════════════════════════════╗
║                         (C)  Howard Chu                                ║
║                                                                        ║
║ Permission to use, copy, modify, and/or distribute this software for   ║
║ any purpose with or without fee is hereby granted, provided that the   ║
║ above copyright notice and this permission notice appear in all copies ║
╚═══════════════════════════════════════════════════════════════════════*/

#[derive(Default)]
pub struct Theme {
    // Highlighted filename when selected
    pub highlight: String,
    // Highlighted directory name when selected
    pub highlight_dir: String,
    // Highlight block background
    pub highlight_background: String,
    // Normal entry names
    pub normal: String,
    // Background of everything
    pub normal_background: String,
    // These two are optional
    pub bottom_bar: String,
    pub bottom_bar_background: String,
}

impl Theme {
    pub fn from(name: &str) -> Self {
        let theme_table = ThemeTable::new();
        let mut theme_i = 0;

        // default
        for (i, t) in theme_table.theme_entries.iter().enumerate() {
            if t.name == "lucius" {
                theme_i = i;
                break;
            }
        }

        for (i, t) in theme_table.theme_entries.iter().enumerate() {
            if t.name == name.to_lowercase() {
                theme_i = i;
                break;
            }
        }

        let t = &theme_table.theme_entries[theme_i];

        let mut theme = Theme {
            highlight: (*t).color[0].clone(),
            highlight_dir: (*t).color[1].clone(),
            highlight_background: (*t).color[2].clone(),
            normal: (*t).color[3].clone(),
            normal_background: (*t).color[4].clone(),
            bottom_bar: String::new(),
            bottom_bar_background: String::new(),
        };

        // optional extra colors for bottom bar
        if t.color.len() == 7 {
            theme.bottom_bar = t.color[5].clone();
            theme.bottom_bar_background = t.color[6].clone();
        }

        theme
    }
}

struct ThemeTable {
    theme_entries: Vec<ThemeEntry>,
}

struct ThemeEntry {
    name: String,
    color: Vec<String>,
}

macro_rules! __theme {
    ( $name: expr, $($x: expr),+ ) => {
        ThemeEntry {
            name: String::from($name),
            color: vec![
            $(
                String::from($x),
            )*],
        }
    };
}

impl ThemeTable {
    fn new() -> Self {
        ThemeTable {
            theme_entries: vec![
                __theme![
                    "trans",
                    "\x1b[0;37m",
                    "\x1b[38;5;175m",
                    "\x1b[48;5;24m",
                    "\x1b[0;37m",
                    "\x1b[48;5;31m",
                    "\x1b[38;5;0m",
                    "\x1b[48;5;175m"
                ],
                __theme![
                    "dark",
                    "\x1b[38;5;0m",
                    "\x1b[38;5;27m",
                    "\x1b[48;5;255m",
                    "\x1b[38;5;255m",
                    "\x1b[48;5;0m"
                ],
                __theme![
                    "lucius",
                    "\x1b[38;5;187m",
                    "\x1b[38;5;117m",
                    "\x1b[48;5;238m",
                    "\x1b[38;5;188m",
                    "\x1b[48;5;236m",
                    "\x1b[38;5;188m",
                    "\x1b[48;5;238m"
                ],
                __theme![
                    "acme",
                    "\x1b[38;5;233m",
                    "\x1b[38;5;39m",
                    "\x1b[48;5;186m",
                    "\x1b[38;5;233m",
                    "\x1b[48;5;230m",
                    "\x1b[38;5;233m",
                    "\x1b[48;5;195m"
                ],
                __theme![
                    "sakura",
                    "\x1b[38;5;253m",
                    "\x1b[38;5;52m",
                    "\x1b[48;5;175m",
                    "\x1b[38;5;253m",
                    "\x1b[48;5;168m",
                    "\x1b[38;5;52m",
                    "\x1b[48;5;175m"
                ],
                __theme![
                    "vscode",
                    "\x1b[38;5;176m",
                    "\x1b[38;5;43m",
                    "\x1b[48;5;236m",
                    "\x1b[38;5;75m",
                    "\x1b[48;5;235m",
                    "\x1b[38;5;117m",
                    "\x1b[48;5;236m"
                ],
                __theme![
                    "jesus",
                    "\x1b[38;5;94m",
                    "\x1b[38;5;236m",
                    "\x1b[48;5;180m",
                    "\x1b[38;5;187m",
                    "\x1b[48;5;137m",
                    "\x1b[38;5;236m",
                    "\x1b[48;5;180m"
                ],
            ],
        }
    }
}
