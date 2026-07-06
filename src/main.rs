mod app;
mod git;
mod ui;

use app::{ActivePane, App};
use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{io, time::Duration};

fn main() -> Result<()> {
    // Determine the git repository path
    let args: Vec<String> = std::env::args().collect();
    let repo_path = if args.len() > 1 {
        &args[1]
    } else {
        "."
    };

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = match App::new(repo_path) {
        Ok(app) => app,
        Err(e) => {
            // Restore terminal on error during init
            disable_raw_mode()?;
            execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
            eprintln!("Error: Failed to open git repository at '{}': {}", repo_path, e);
            std::process::exit(1);
        }
    };

    // Event loop
    loop {
        terminal.draw(|f| ui::render(&mut app, f))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if app.show_folder_selector {
                    match key.code {
                        KeyCode::Esc | KeyCode::Char('o') => {
                            app.show_folder_selector = false;
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            app.folder_select_up();
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            app.folder_select_down();
                        }
                        KeyCode::Enter => {
                            let _ = app.folder_enter();
                        }
                        KeyCode::Char(' ') | KeyCode::Char('s') => {
                            let _ = app.folder_open_repo();
                        }
                        _ => {}
                    }
                } else {
                    // Global hotkeys
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            app.should_quit = true;
                        }
                        KeyCode::Char('o') => {
                            app.toggle_folder_selector();
                        }
                        KeyCode::Tab => {
                            app.toggle_pane();
                        }
                        // Navigation controls based on active pane
                        KeyCode::Up | KeyCode::Char('k') => match app.active_pane {
                            ActivePane::Commits => app.move_selection_up(),
                            ActivePane::Diff => app.scroll_diff_up(),
                        },
                        KeyCode::Down | KeyCode::Char('j') => match app.active_pane {
                            ActivePane::Commits => app.move_selection_down(),
                            ActivePane::Diff => app.scroll_diff_down(),
                        },
                        _ => {}
                    }
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
