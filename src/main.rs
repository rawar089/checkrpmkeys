mod app;
mod data;
mod i18n;
mod ui;
mod cli;
use std::io;
use std::time::Duration;

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, backend::Backend, Terminal};

use app::{App, InputMode};
use data::load_data;
use cli::run_cli;
use i18n::Lang;
use clap::Parser;
use crate::data::GpgKeyRecord;

// Define the command-line argument schema using Clap
#[derive(Parser, Debug)]
#[command(
    name = "checkrpmkeys",
    author = "rawar089",
    version = "0.2",
    about = "Checks installed RPM repository GPG signing keys for expirations.",
    long_about = "Queries the local RPM database for 'gpg-pubkey' packages, decodes their internal cryptographic profiles using sequoia, and displays expiration states in  a tui."
)]

struct Args {
    /// Output the raw records array as pretty-printed JSON payload
    #[arg(short, long, conflicts_with = "generate", conflicts_with = "yaml")]
    json: bool,

    /// Generate a bash shell script containing 'rpm -e' removal targets for expired keys
    #[arg(short, long, conflicts_with = "json", conflicts_with = "yaml")]
    generate: bool,

    /// Output the raw records array as pretty-printed JSON payload
    #[arg(short, long, conflicts_with =  "json", conflicts_with = "generate")]
    yaml: bool,

}
fn main() -> Result<()> {
    let args = Args::parse();
    let optional_output = args.json || args.yaml || args.generate;

    let records = load_data();

    if optional_output  {
        run_cli(&args, records)
    } else {
        run_tui(records)
    }
}

fn run_tui(records: Vec<GpgKeyRecord>) -> Result<()> {
    // Auto-detect UI language from the environment (LC_ALL/LANG/...),
    // falling back to English.
    let lang = Lang::detect_from_env();

    let mut terminal = setup_terminal()?;
    let mut app = App::new(records, lang);
    let result = run_app(&mut terminal, &mut app);
    restore_terminal(&mut terminal)?;

    if let Err(err) = result {
        eprintln!("Error: {err}");
    }
    Ok(())
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    Ok(Terminal::new(backend)?)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|f| ui::draw(f, app))?;

        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                // On some platforms (Windows) key events fire on both press
                // and release; only act on press to avoid double input.
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match app.input_mode {
                    InputMode::Normal => handle_normal_key(app, key.code),
                    InputMode::Filtering => handle_filter_key(app, key.code),
                }
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

fn handle_normal_key(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Esc => {
            if app.show_help {
                app.show_help = false;
            } else if app.show_details {
                app.show_details = false;
            } else if !app.filter_text.is_empty() {
                app.filter_text.clear();
                app.apply_filter();
            } else {
                app.should_quit = true;
            }
        }
        KeyCode::Char('j') | KeyCode::Down => app.next(),
        KeyCode::Char('k') | KeyCode::Up => app.previous(),
        KeyCode::Char('g') => app.go_top(),
        KeyCode::Char('G') => app.go_bottom(),
        KeyCode::PageDown => app.page_down(10),
        KeyCode::PageUp => app.page_up(10),
        KeyCode::Char('/') => app.input_mode = InputMode::Filtering,
        KeyCode::Char('s') => app.cycle_sort_column(),
        KeyCode::Char('r') => app.toggle_sort_direction(),
        KeyCode::Char('l') => app.cycle_lang(),
        KeyCode::Char('h') => app.toggle_help(),
        KeyCode::Enter => app.toggle_details(),
        KeyCode::Char('c') => {
            app.filter_text.clear();
            app.apply_filter();
        }
        _ => {}
    }
}

fn handle_filter_key(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Enter | KeyCode::Esc => app.input_mode = InputMode::Normal,
        KeyCode::Char(c) => {
            app.filter_text.push(c);
            app.apply_filter();
        }
        KeyCode::Backspace => {
            app.filter_text.pop();
            app.apply_filter();
        }
        _ => {}
    }
}
