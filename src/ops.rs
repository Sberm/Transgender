/*═══════════════════════════════════════════════════════════════════════╗
║                         (C)  Howard Chu                                ║
║                                                                        ║
║ Permission to use, copy, modify, and/or distribute this software for   ║
║ any purpose with or without fee is hereby granted, provided that the   ║
║ above copyright notice and this permission notice appear in all copies ║
╚═══════════════════════════════════════════════════════════════════════*/

/// All the constant
pub mod consts {
    pub const HOME_VAR: &str = "HOME";
    pub const CONFIG_FILE: &str = ".tsrc";
    pub const EDITOR_KEY: &str = "editor";
    pub const EDITOR: &str = "/bin/vi";
    pub const THEME_KEY: &str = "theme";
}

/// File browser operation code
pub enum Op {
    Noop,
    Up,
    Down,
    Left,
    Right,
    Exit,
    ExitCursor,
    Quit,
    Top,
    Bottom,
    Search,
    NextMatch,
    PrevMatch,
    PageUp,
    PageDown,
}

#[derive(Copy, Clone)]
pub enum Mode {
    NORMAL,
    SEARCH,
}
