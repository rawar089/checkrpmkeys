mod app;
mod data;
mod i18n;
mod ui;
mod cli;
use std::io::{self, Write};
use std::time::Duration;

use anyhow::Result;
use arboard::Clipboard;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers,
    },
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
    version,
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

    let records = load_data()?;

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

fn run_app<B>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()>
where
    B: Backend,
    B::Error: std::error::Error + Send + Sync + 'static,
{
    // On Wayland, the Clipboard owns the data source. Keep it alive for the
    // entire TUI session so copied details remain available.
    let mut clipboard = None;

    loop {
        terminal.draw(|f| ui::draw(f, app))?;

        if event::poll(Duration::from_millis(200))?
            &&  let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                if app.show_details
                    && key.code == KeyCode::Char('c')
                    && key.modifiers.contains(KeyModifiers::CONTROL)
                {
                    copy_selected_key_details(app, &mut clipboard)?;
                    continue;
                }
                match app.input_mode {
                    InputMode::Normal => handle_normal_key(app, key.code),
                    InputMode::Filtering => handle_filter_key(app, key.code),
                }
            }

        if app.should_quit {
            return Ok(());
        }
    }
}

fn copy_selected_key_details(app: &App, clipboard: &mut Option<Clipboard>) -> Result<()> {
    if let Some(details) = ui::selected_key_details_text(app) {
        if let Some(clipboard) = clipboard.as_mut() {
            if clipboard.set_text(details.clone()).is_ok() {
                return Ok(());
            }
        } else if let Ok(mut new_clipboard) = Clipboard::new()
            && new_clipboard.set_text(details.clone()).is_ok() {
            *clipboard = Some(new_clipboard);
            return Ok(());
        }

        // Try teminal fallback if Wayland/X11 clipboard do not work
        copy_with_osc52(&details)?;
    }
    Ok(())
}

/// Sends clipboard contents to the user's terminal. This works over SSH when
/// the local terminal supports OSC 52, without requiring a remote display.
fn copy_with_osc52(text: &str) -> io::Result<()> {
    let encoded = STANDARD.encode(text);
    let mut stdout = io::stdout().lock();
    write!(stdout, "\x1b]52;c;{encoded}\x1b\\")?;
    stdout.flush()
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
