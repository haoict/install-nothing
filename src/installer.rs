use crate::cli::Stage;
use crate::log_generator::LogGenerator;
use crate::messages::{EASTER_EGGS, RETRY_MESSAGES, WARNINGS};
use crate::stages::selected_stages;
use crate::ui::{ProgressBar, Spinner};
use colored::*;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute,
    terminal::{self, ClearType},
};
use rand::Rng;
use std::io;
use std::thread;
use std::time::Duration;

pub struct Installer {
    rng: rand::rngs::ThreadRng,
    selected_stages: Vec<Stage>,
    compact: bool,
}

impl Installer {
    pub fn new(stages: Vec<Stage>, compact: bool) -> Self {
        Self {
            rng: rand::thread_rng(),
            selected_stages: stages,
            compact,
        }
    }

    fn check_exit(&self) -> bool {
        if event::poll(Duration::from_millis(0)).unwrap_or(false) {
            if let Ok(Event::Key(key_event)) = event::read() {
                if key_event.code == KeyCode::Char('c')
                    && key_event.modifiers.contains(event::KeyModifiers::CONTROL)
                {
                    return true;
                }
            }
        }
        false
    }

    fn print_header(&self) {
        let sep_len = if self.compact { 48 } else { 63 };
        let separator = "=".repeat(sep_len);
        let title = if self.compact {
            " UNIVERSAL SYSTEM INSTALLER v3.2.1 (Build 1999)"
        } else {
            "         UNIVERSAL SYSTEM INSTALLER v3.2.1 (Build 1999)"
        };
        println!("{}", separator.bright_cyan());
        println!("{}", title.bright_white().bold());
        println!("{}", separator.bright_cyan());
        println!();
        thread::sleep(Duration::from_millis(1500));
    }

    fn show_easter_egg(&mut self) -> io::Result<()> {
        if self.rng.gen_bool(0.15) {
            println!();
            let egg = EASTER_EGGS[self.rng.gen_range(0..EASTER_EGGS.len())];
            let mut spinner = Spinner::new();
            spinner.animate(egg, 1500, &|| self.check_exit())?;
            println!();
        }
        Ok(())
    }

    fn show_warning(&mut self) {
        if self.rng.gen_bool(0.2) {
            let warning = WARNINGS[self.rng.gen_range(0..WARNINGS.len())];
            println!("\n{}", warning.yellow());
            thread::sleep(Duration::from_millis(1000));
            println!("{}", "Continuing anyway...".dimmed());
            println!();
        }
    }

    fn show_retry(&mut self) -> io::Result<()> {
        if self.rng.gen_bool(0.1) {
            let message = RETRY_MESSAGES[self.rng.gen_range(0..RETRY_MESSAGES.len())];
            println!("\n{}", message.yellow());
            thread::sleep(Duration::from_millis(800));

            let mut spinner = Spinner::new();
            spinner.animate("Reconnecting to mirror.oldsoft.org", 1200, &|| {
                self.check_exit()
            })?;
            println!();
        }
        Ok(())
    }

    pub fn run(&mut self) -> io::Result<()> {
        // Set compact mode globally
        LogGenerator::set_compact_mode(self.compact);
        ProgressBar::set_compact_mode(self.compact);

        terminal::enable_raw_mode()?;
        execute!(
            io::stdout(),
            terminal::Clear(ClearType::All),
            cursor::MoveTo(0, 0)
        )?;
        terminal::disable_raw_mode()?;

        self.print_header();

        println!(
            "{}",
            "Initializing installation environment...".bright_white()
        );
        thread::sleep(Duration::from_millis(1000));

        let mut spinner = Spinner::new();
        spinner.animate("Detecting hardware configuration...", 1500, &|| {
            self.check_exit()
        })?;
        println!();

        let mut cycle = 0;
        loop {
            cycle += 1;

            if cycle > 1 {
                let cycle_sep_len = if self.compact { 38 } else { 63 };
                let separator = "═".repeat(cycle_sep_len);
                println!("\n{}", separator.bright_magenta());
                println!(
                    "{}",
                    format!("Beginning installation cycle #{}...", cycle)
                        .bright_magenta()
                        .bold()
                );
                println!("{}", separator.bright_magenta());
                thread::sleep(Duration::from_millis(1000));
            }

            let stages = selected_stages(&self.selected_stages);

            for stage in stages {
                if self.check_exit() {
                    return Err(io::Error::new(io::ErrorKind::Interrupted, "User interrupt"));
                }

                self.show_easter_egg()?;
                self.show_warning();
                self.show_retry()?;

                stage.run(&|| self.check_exit())?;

                thread::sleep(Duration::from_millis(self.rng.gen_range(300..800)));
            }

            println!(
                "\n{}",
                "Installation complete! Restarting installation process..."
                    .bright_green()
                    .bold()
            );
            thread::sleep(Duration::from_millis(2000));
        }
    }
}

impl Default for Installer {
    fn default() -> Self {
        Self::new(Stage::all(), false)
    }
}
