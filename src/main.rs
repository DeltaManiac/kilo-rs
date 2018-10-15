extern crate libc;
extern crate nix;
extern crate termios;
use std::os::unix::io::AsRawFd;

use libc::TIOCGWINSZ;
use nix::pty::Winsize;
use std::fs::File;
use std::io::prelude::*;
use std::io;
use std::os::unix::io::RawFd;

use termios::*;
use std::io::BufReader;
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

#[derive(Debug,Default)]
struct EditorRow {
        idx: i32,
        size: i32,
        size_rendered: i32,
        content: Option<String>,
        rendered_content:Option<Vec<char>>,
}
#[derive(Debug,Default)]
struct Cursor(i32, i32);
#[derive(Debug,Default)]
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
                file_name:None,
                dirty: 0,
        }
    }
    
    
    fn open(&mut self,filename:String)->std::io::Result<()>{
            self.file_name = Some(filename);
            self.dirty = 0;
            let file = File::open((self.file_name.clone()).unwrap())?;
            let mut buf_reader = BufReader::new(file);
            let mut rows:Vec<EditorRow> = Vec::new();
            let mut contents = String::new();
            let mut counter = 0;
            while buf_reader.read_line(&mut contents)? != 0{
                if contents.pop().unwrap()=='\n' {
                    contents.push('\0');
            } 
            rows.push(EditorRow{
                    idx: counter,
                    size: contents.len() as i32,
                    size_rendered: 0,
                    content: Some(contents.clone()),
                    rendered_content:None,
            });
            contents.clear();
                counter+=1;
        };
        self.rows=Some(rows);
            Ok(())
    }
    
    
    fn enable_raw_mode(&mut self, fd: RawFd) -> io::Result<()> {
            let mut termios = try!(Termios::from_fd(fd));
            println!("Termio {:?}",&termios);
            //tcgetattr(fd,&mut termios);
        
        
            /* control modes = set 8 bit chars */
            termios.c_cflag |= CS8;
        
            /* local modes - choing off, canonical off, no extended functions,
* no signal chars (^Z,^C) */
            termios.c_lflag &= !(ECHO | ICANON| IEXTEN | ISIG);
        
        
            /* input modes: no break, no CR to NL, no parity check, no strip char,
* no start/stop output control. */
            termios.c_iflag &= !(BRKINT | ICRNL | INPCK | ISTRIP | IXON);
        
            /* output modes - disable post processing */
            termios.c_oflag &= !(OPOST);
        
            termios.c_cc[VTIME] = 1;
            try!(tcsetattr(fd, TCSAFLUSH, &termios));
        
            println!("Termio {:?}",&termios);
        
            self.raw_mode=1;
            Ok(())
    }
    
}

//ioctl_read!(read_winsize,std::io::stdin().as_raw_fd(),TIOCGWINSZ,Winsize);
fn main() {
        let args: Vec<String> = std::env::args().collect();
        let mut c = Editor::new();
        c.open(args[1].to_owned());
        c.enable_raw_mode(std::io::stdin().as_raw_fd());
        //println!("{:?}", c);
}
