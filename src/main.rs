use std::io::{stdin, stdout, Write};
use std::thread;
use std::time::Duration;
use termion::{color, cursor, event::Key, raw::IntoRawMode, input::TermRead};
use termion::terminal_size;
use rand::prelude::*;
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
    
    // Create a single RNG at the start of your program
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

     let mut counter = 0;
     while *running.lock().unwrap() || counter % 10 == 0 {
         // Check if window was resized
if *resized.lock().unwrap() {
    let size = terminal_size()?;
    width = size.0;
    height = size.1;
    *resized.lock().unwrap() = false;

     // Update existing raindrops
     for drop in &mut raindrops {
         drop.x = rng.gen_range(1..=width);
         drop.y = rng.gen_range(1..=height);
         drop.style = rains.choose(&mut rng).unwrap().to_string();
     }
}

         // Create a buffer string
         let mut buffer_string = String::new();

         for drop in &mut raindrops {
             drop.fall(height);
             buffer_string.push_str(&format!(
                 "{}{}{}",
                 cursor::Goto(drop.x, drop.y),
                 color::White.fg_str(),
                 drop.style
             ));
         }

         write!(stdout, "{}", buffer_string)?;
         stdout.flush()?;

         thread::sleep(Duration::from_millis(SLEEP_DURATION));
         
         // Clear the screen
         write!(stdout, "{}", termion::clear::All)?;
         stdout.flush()?;
         
         counter += 1;
     }

     // Show the cursor again before exiting
     write!(stdout, "{}", termion::cursor::Show)?;

     Ok(())
}

