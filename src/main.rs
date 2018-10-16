extern crate libc;
extern crate nix;
extern crate termios;
use std::os::unix::io::AsRawFd;

use libc::TIOCGWINSZ;
use nix::pty::Winsize;
use std::fmt;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::BufReader;
use std::os::unix::io::RawFd;
use termios::*;
enum KeyAction {
    Null,
    CtrlC,
    CtrlD,
    CtrlF,
    CtrlH,
    TAB = 9,
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    ArrowDown,
}

#[derive(Debug, Default)]
struct EditorRow {
    idx: i32,
    size: i32,
    size_rendered: i32,
    content: Option<String>,
    rendered_content: Option<String>,
}
#[derive(Debug, Default)]
struct Cursor(i32, i32);
#[derive(Debug, Default)]
struct Editor {
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

impl Editor {
    pub fn new() -> Editor {
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

        Editor {
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
    fn update_row(&mut self, content: String) -> String {
        let mut tab = 0;
        let mut nonprint = 0;
        let mut j = 0;
        let mut idx = 0;
        for j in content.chars() {
            if j as u8 == KeyAction::TAB as u8 {
                tab += 1;
            }
        }
        let mut rendered_string = String::new();
        for j in content.chars() {
            if j as u8 == KeyAction::TAB as u8 {
                rendered_string.push(' ');
                idx += 1;
                while (idx + 1) % 8 != 0 {
                    rendered_string.push(' ');
                    idx += 1;
                }
            } else {
                rendered_string.push(j as char);
            }
        }
        rendered_string.push('\0');
        rendered_string.to_owned()
    }

    fn open(&mut self, filename: String) -> std::io::Result<()> {
        self.file_name = Some(filename);
        self.dirty = 0;
        let file = File::open((self.file_name.clone()).unwrap())?;
        let mut buf_reader = BufReader::new(file);
        let mut rows: Vec<EditorRow> = Vec::new();
        let mut contents = String::new();
        let mut counter = 0;
        while buf_reader.read_line(&mut contents)? != 0 {
            if contents.pop().unwrap() == '\n' {
                contents.push('\0');
            }
            let mut rendered_content = self.update_row(contents.clone());
            rows.push(EditorRow {
                idx: counter,
                size: contents.len() as i32,
                size_rendered: rendered_content.len() as i32,
                content: Some(contents.clone()),
                rendered_content: Some(rendered_content),
            });
            contents.clear();
            counter += 1;
        }
        self.num_rows = rows.len() as i32;
        self.rows = Some(rows);
        Ok(())
    }

    fn enable_raw_mode(&mut self, fd: RawFd) -> io::Result<()> {
        let mut termios = try!(Termios::from_fd(fd));
        // println!("Termio {:?}", &termios);
        //tcgetattr(fd,&mut termios);

        /* control modes = set 8 bit chars */
        termios.c_cflag |= CS8;

        /* local modes - choing off, canonical off, no extended functions,
         * no signal chars (^Z,^C) */
        termios.c_lflag &= !(ECHO | ICANON | IEXTEN | ISIG);

        /* input modes: no break, no CR to NL, no parity check, no strip char,
         * no start/stop output control. */
        termios.c_iflag &= !(BRKINT | ICRNL | INPCK | ISTRIP | IXON);

        /* output modes - disable post processing */
        termios.c_oflag &= !(OPOST);

        termios.c_cc[VTIME] = 1;
        try!(tcsetattr(fd, TCSAFLUSH, &termios));
        // println!("Termio {:?}", &termios);
        self.raw_mode = 1;
        Ok(())
    }

    fn refresh_screen(&mut self) {
        let mut output: String = String::new();
        /* Hide Cursor */
        output.push_str("\x1b[?25l");
        /* Go Home */
        output.push_str("\x1b[H");
        for y in 0..self.row_screen {
            let mut filerow = self.row_offset + y as i32;
            if filerow >= self.num_rows {
                if self.num_rows == 0 && y as i32 == (self.row_screen / 3) as i32 {
                    let welcom = "Kilo --editor vesion 1\x1b[0K\r\n".to_string();
                    let mut padddin = (self.col_screen - welcom.len() as i32) / 2 as i32;
                    if padddin != 0 {
                        output.push('~');
                        padddin -= 1;
                    }
                    while padddin >= 0 {
                        output.push_str(" ");
                        padddin -= 1;
                    }
                    output.push_str(&welcom);
                } else {
                    output.push_str("~\x1b[0K\r\n");
                }
                continue;
            }
            if filerow > -1 {
                let mut len =
                    self.rows.as_ref().unwrap()[filerow as usize].size_rendered - self.col_offset;
                if len > 0 {
                    if len > self.col_screen {
                        len = self.col_screen;
                    }
                }
                for j in 0..len {
                    output.push(
                        self.rows.as_ref().unwrap()[filerow as usize]
                            .rendered_content
                            .as_ref()
                            .unwrap()
                            .chars()
                            .nth(j as usize)
                            .unwrap(),
                    );
                }
            }

            output.push_str("\x1b[39m");
            output.push_str("\x1b[0K");
            output.push_str("\r\n");
        }

        //Status row
        output.push_str("\x1b[0K");
        output.push_str("\x1b[7m");
        let status_row_1 = format!(
            "{} - {} lines {}",
            self.file_name.as_ref().unwrap(),
            &self.num_rows,
            ""
        );
        //println!("{}",status_row_1 );
        let status_row_2 = format!("{}/{}", self.row_offset + self.cursor.1 + 1, &self.num_rows);
        //println!("{}",status_row_2 );
        output.push_str(&status_row_1);
        let mut len = status_row_1.len();
        if len > self.col_screen as usize {
            len = self.col_screen as usize;
        }
        while (len as i32 )< self.col_screen  {
            if self.col_screen - len as i32 == status_row_2.len() as i32 {
                output.push_str(&status_row_2);
                break;
            } else {
                output.push(' ');
                len += 1;
            }
        }
        output.push_str("\x1b[0m\r\n");

        /* Show Cursor*/
        output.push_str("\x1b[?25h");
        write!(std::io::stdout(), "{}", &output);
        //print!("{}", &output);
    }
}

//ioctl_read!(read_winsize,std::io::stdin().as_raw_fd(),TIOCGWINSZ,Winsize);
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut c = Editor::new();
    c.open(args[1].to_owned());
    //c.enable_raw_mode(std::io::stdin().as_raw_fd());
    c.refresh_screen();
    //println!("{:?}", c);
}
