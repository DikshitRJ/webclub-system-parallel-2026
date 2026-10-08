mod addition;
mod sha1;

use std::io::{stdout,Write};
use crossterm::{cursor,execute,terminal::{self,Clear,ClearType},event::{self,Event,KeyCode}};

fn select_function()->usize {
    let options=["add","sha1"];
    let mut selected=0usize;
    terminal::enable_raw_mode().expect("raw mode fail");
    loop {
        let mut out=stdout();
        execute!(out, Clear(ClearType::All), cursor::MoveTo(0,0)).expect("clear fail");
        println!("use up/down arrows and press enter");
        println!();
        for (i, option) in options.iter().enumerate() {
            if i==selected {
                println!("> {}",option);
            } else {
                println!("  {}",option);
            }
        }
        out.flush().expect("flush fail");
        if let Event::Key(key)=event::read().expect("read key fail") {
            match key.code {
                KeyCode::Up => {
                    if selected>0 {
                        selected-=1;
                    }
                }
                KeyCode::Down => {
                    if selected+1<options.len() {
                        selected+=1;
                    }
                }
                KeyCode::Enter => break,
                _ => {}
            }
        }
    }
    terminal::disable_raw_mode().expect("raw mode fail");
    selected
}

fn main() {
    match select_function() {
        0 => addition::add(),
        1 => sha1::sha1(),
        _ => {}
    }
}