extern crate libc;
extern crate nix;

use std::os::unix::io::AsRawFd;

use libc::TIOCGWINSZ;
use nix::pty::Winsize;
use std::fs::File;
use std::io::prelude::*;

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

#[derive(Debug)]
struct EditorRow {
        idx: i32,
        size: i32,
        size_rendered: i32,
        content: Option<String>,
        rendered_content:Option<Vec<char>>,
}
#[derive(Debug)]
struct Cursor(i32, i32);
#[derive(Debug)]
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
            let mut contents = String::new();
            while buf_reader.read_line(&mut contents)? != 0{
                println!("###{}",contents);
                contents.clear();
        };
        Ok(())
    }
    
    //Insert a row at the specified position
    fn insert_row(&mut self,position:i32, content:String){
            if position > self.num_rows {
                return;
        }
        
        let row:EditorRow = EditorRow{
                idx: position  ,
                size: content.len() as i32,
                size_rendered: 0,
                content: Some(content),
                rendered_content:None
        };
        
        if self.rows.is_none()  {
                let a = vec![row];
                self.rows = Some(a);
        } else {
            self.rows.unwrap().insert(position as usize, row);
            
        }
        
    }
}

//ioctl_read!(read_winsize,std::io::stdin().as_raw_fd(),TIOCGWINSZ,Winsize);
fn main() {
        let args: Vec<String> = std::env::args().collect();
        let mut c = Editor::new();
        c.open(args[1].to_owned());
        println!("{:?}", c);
}
