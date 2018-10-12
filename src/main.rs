extern crate libc;
extern crate nix;

use std::os::unix::io::AsRawFd;

use libc::TIOCGWINSZ;
use nix::pty::Winsize;
enum KeyAction {
    Null,
    CtrlC,
    CtrlD,
    CtrlF,
    CtrlH,
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    ArrowDown,
}

#[derive(Debug)]
struct EditorRow {
    idx: i32,
    size: i32,
    size_rendered: i32,
    content: Vec<char>,
    rendered_content: Vec<char>,
}
#[derive(Debug)]
struct Cursor(i32, i32);
#[derive(Debug)]
struct EditorConfig {
    cursor: Cursor,
    row_offset: i32,
    col_offset: i32,
    row_screen: i32,
    col_screen: i32,
    num_rows: i32,
    raw_mode: i32,
    rows: Option<Vec<EditorRow>>,
    dirty: i32,
    file_name: Option<String>,
}

impl EditorConfig {
    pub fn new() -> EditorConfig {
        let mut b: Winsize = Winsize {
            ws_col: 0,
            ws_row: 0,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };
        let a = unsafe { libc::ioctl(std::io::stdout().as_raw_fd(), TIOCGWINSZ, &mut b) };
        if a == -1 || b.ws_col == 0 {
            panic!("No idea of console size");
        }

        EditorConfig {
            cursor: Cursor(0, 0),
            row_offset: -2,
            col_offset: 0,
            row_screen: b.ws_row as i32,
            col_screen: b.ws_col as i32,
            num_rows: 0,
            raw_mode: 0,
            rows: None,
            file_name: None,
            dirty: 0,
        }
    }
}

//ioctl_read!(read_winsize,std::io::stdin().as_raw_fd(),TIOCGWINSZ,Winsize);
fn main() {
    println!("{:?}", EditorConfig::new());
}
