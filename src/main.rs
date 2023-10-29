use std::io::{stdout, Write};
use std::thread;
use std::time::Duration;
use termion::color;
use termion::cursor;
use termion::raw::IntoRawMode;
use termion::event::Key;
use termion::input::TermRead;
use termion::terminal_size;
use std::io::stdin;
use rand::prelude::*;
use rand::Rng;
use std::sync::{Arc, Mutex};
use signal_hook::{iterator::Signals};
use libc::SIGWINCH;

const SLEEP_DURATION: u64 = 25;

struct Raindrop {
    x: u16,
    y: u16,
    style: String,
}

impl Raindrop {
    fn new(x: u16, y: u16, style: String) -> Self {
        Self { x, y, style }
    }

    fn fall(&mut self, height: u16) {
        if self.y < height {
            self.y += 1;
        } else {
            self.y = 1;  // Wrap back to 1 instead of 0
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut stdout = stdout().into_raw_mode()?;
    let stdin = stdin();

    let rains = vec!["|", "│", "┃", "┆", "┇", "┊", "┋", "╽", "╿"];
    let colors = vec![
        color::White,
        color::White,
        // More from 256 color mode
    ];
    let mut rng = rand::thread_rng();

    // Shared flag for threads
    let running = Arc::new(Mutex::new(true));
    let resized = Arc::new(Mutex::new(false));

    // Spawn a new thread to listen for key events
    let r = Arc::clone(&running);
    thread::spawn(move || {
        for c in stdin.keys() {
            match c.unwrap() {
                // Exit on Esc or Ctrl+C
                Key::Esc | Key::Ctrl('c') => {
                    let mut r = r.lock().unwrap();
                    *r = false;
                    break;
                },
                _ => (),
            }
        }
    });

    // Handle SIGWINCH
let mut signals = Signals::new(&[SIGWINCH])?;
let re = Arc::clone(&resized);
thread::spawn(move || {
    for _ in signals.forever() {
        let mut re = re.lock().unwrap();
        *re = true;
    }
});


    // Get terminal size
    let (mut width, mut height) = terminal_size()?;

     // Create raindrops
     let mut raindrops: Vec<Raindrop> = (0..width).map(|x| Raindrop::new(x, rng.gen_range(1..=height), rains.choose(&mut rng).unwrap().to_string())).collect();

     // Hide the cursor
     write!(stdout, "{}", termion::cursor::Hide)?;

     while *running.lock().unwrap() {
         // Check if window was resized
if *resized.lock().unwrap() {
    let size = terminal_size()?;
    width = size.0;
    height = size.1;
    *resized.lock().unwrap() = false;

    // Clear the screen
    write!(stdout, "{}", termion::clear::All)?;

    // Clear existing raindrops and create new ones
    raindrops.clear();
    raindrops = (0..width).map(|x| Raindrop::new(x, rng.gen_range(1..=height), rains.choose(&mut rng).unwrap().to_string())).collect();
}


         for drop in &mut raindrops {
             drop.fall(height);
             let color = colors.choose(&mut rng).unwrap();
             write!(
                 stdout,
                 "{}{}{}",
                 cursor::Goto(drop.x, drop.y),
                 color.fg_str(),
                 drop.style
             )?;
         }

         stdout.flush()?;

         thread::sleep(Duration::from_millis(SLEEP_DURATION));
         
         // Clear the screen
         write!(stdout, "{}", termion::clear::All)?;
         stdout.flush()?;
     }

     // Show the cursor again before exiting
     write!(stdout, "{}", termion::cursor::Show)?;

     Ok(())
}

