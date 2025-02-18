/*═══════════════════════════════════════════════════════════════════════╗
║                         (C)  Howard Chu                                ║
║                                                                        ║
║ Permission to use, copy, modify, and/or distribute this software for   ║
║ any purpose with or without fee is hereby granted, provided that the   ║
║ above copyright notice and this permission notice appear in all copies ║
╚═══════════════════════════════════════════════════════════════════════*/

mod browser;
mod canvas;
mod ops;
mod theme;
mod util;
mod widechar_width;

use std::env;
use std::path::Path;
use std::process::exit;

fn main() {
    let mut path = String::from(".");
    let mut dest_file: Option<String> = None;

    let mut args = env::args();
    let print_help_message = || {
        let help_message = r###"
  transgender
    --dest         FILE file that transgender outputs the destination to
    -v, --version  Print current version
    -h, --help     Show this message
    --sh           Print transgender configuration shell script

    Use transgender <DIR> to start transgender in DIR directory

"###;
        print!("{}", help_message);
    };

    loop {
        let _s = args.next();
        if _s.is_none() {
            break;
        }

        let s = _s.expect("Failed to unwrap arguments");

        if s.eq("-v") || s.eq("--version") {
            println!("\n  Transgender.rs\n\n    Regex-powered trans\n");
            exit(0);
        } else if s.eq("--sh") {
            let script = r###"ts() {
  dest_file=$(mktemp -t "ts.XXXX")
  transgender $1 --dest "${dest_file}"
  dest_dir=$(tail -n 1 "${dest_file}")
  cd "${dest_dir}"
  rm -rf "${dest_file}"
}

if [[ "$0" =~ .*bash$ ]];
then
  complete -o dirnames ts
fi

if [[ "$0" == "zsh" ]];
then
  true
fi
"###;
            print!("{}", script);
            exit(0);
        } else if s.eq("--dest") {
            let _one_more = args.next();
            if _one_more.is_none() {
                println!("--dest: Please specify the file path with --dest FILE");
                print_help_message();
                exit(0);
            }
            dest_file = Some(_one_more.expect("Failed to unwrap destination file path"));
        } else if s.eq("-h") || s.eq("--help") {
            print_help_message();
            exit(0);
        } else {
            // the starting path with no argument name
            let p = Path::new(&s);
            if p.is_dir() {
                path = String::from(p.to_str().expect("Failed to convert an existed path"));
            }
        }
    }

    // multiple arguments with random order
    let mut canvas = canvas::new();
    let mut browser = browser::new(&path, dest_file);

    util::enter_albuf();
    browser.start_loop(&mut canvas);
}
