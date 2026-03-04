use colored::*;
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::thread;
use std::time::Duration;

#[derive(Clone, Copy)]
pub enum ProgressStyle {
    Hash,
    Equals,
    Block,
    #[allow(dead_code)]
    Gradient,
}

static COMPACT_MODE: AtomicBool = AtomicBool::new(false);
static PROGRESS_BAR_WIDTH: AtomicUsize = AtomicUsize::new(20);

pub struct ProgressBar {
    style: ProgressStyle,
}

impl ProgressBar {
    pub fn set_compact_mode(compact: bool) {
        COMPACT_MODE.store(compact, Ordering::Relaxed);
        let width = if compact { 10 } else { 20 };
        PROGRESS_BAR_WIDTH.store(width, Ordering::Relaxed);
    }

    pub fn is_compact() -> bool {
        COMPACT_MODE.load(Ordering::Relaxed)
    }

    pub fn new(style: ProgressStyle) -> Self {
        Self { style }
    }

    pub fn render(&self, progress: f32) -> String {
        let width = PROGRESS_BAR_WIDTH.load(Ordering::Relaxed);
        let filled = ((progress * width as f32) as usize).min(width);
        let empty = width - filled;

        let (fill_char, empty_char) = match self.style {
            ProgressStyle::Hash => ('#', '.'),
            ProgressStyle::Equals => ('=', ' '),
            ProgressStyle::Block => ('█', '░'),
            ProgressStyle::Gradient => {
                if filled > empty {
                    ('▓', '░')
                } else {
                    ('▒', '░')
                }
            }
        };

        format!(
            "[{}{}] {:3.0}%",
            fill_char.to_string().repeat(filled).bright_green(),
            empty_char.to_string().repeat(empty).dimmed(),
            progress * 100.0
        )
    }

    pub fn animate(
        &self,
        message: &str,
        duration_ms: u64,
        exit_check: &dyn Fn() -> bool,
    ) -> io::Result<()> {
        let steps = 50;
        let delay = duration_ms / steps;
        let separate_line = COMPACT_MODE.load(Ordering::Relaxed);

        if separate_line {
            // Compact mode: message on separate line
            println!("{}", message.bright_white());
        } else {
            // Normal mode: message inline
            print!("{}", message.bright_white());
            io::stdout().flush()?;
        }

        for i in 0..=steps {
            if exit_check() {
                return Err(io::Error::new(io::ErrorKind::Interrupted, "User interrupt"));
            }

            let progress = i as f32 / steps as f32;

            if separate_line {
                print!("\r{}", self.render(progress));
            } else {
                print!("\r{} {}", message.bright_white(), self.render(progress));
            }
            io::stdout().flush()?;
            thread::sleep(Duration::from_millis(delay));
        }
        println!();
        Ok(())
    }
}
