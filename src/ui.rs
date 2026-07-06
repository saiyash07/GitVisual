use crate::app::{ActivePane, App};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

pub fn render(app: &mut App, frame: &mut Frame) {
    // 1. Create a 3-part vertical layout: Header, Main body, Footer/Status Bar
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header title
            Constraint::Min(10),   // Main area (split left/right)
            Constraint::Length(1), // Footer status bar
        ])
        .split(frame.area());

    // 2. Render Header
    let header_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan))
        .title(" GitVisual TUI ");
    let header_text = vec![Line::from(vec![
        Span::styled("⚡ Interactive Git History & Diff Viewer ", Style::default().add_modifier(Modifier::BOLD)),
        Span::styled(format!("(Repo: {})", app.git_repo.repo.path().display()), Style::default().fg(Color::DarkGray)),
    ])];
    let header = Paragraph::new(header_text).block(header_block);
    frame.render_widget(header, chunks[0]);

    // 3. Render Main Area (Split Left/Right)
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(45), // Left: Commits
            Constraint::Percentage(55), // Right: Diff
        ])
        .split(chunks[1]);

    // 3a. Commits List (Left)
    let commits_border_style = if app.active_pane == ActivePane::Commits {
        Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let commits_border_type = if app.active_pane == ActivePane::Commits {
        BorderType::Double
    } else {
        BorderType::Rounded
    };
    
    let commits_block = Block::default()
        .borders(Borders::ALL)
        .border_type(commits_border_type)
        .border_style(commits_border_style)
        .title(" Commit History ");

    let commit_items: Vec<ListItem> = app
        .commits
        .iter()
        .enumerate()
        .map(|(idx, commit)| {
            let mut spans = Vec::new();
            
            // Highlight selected item node
            if idx == app.selected_commit_idx {
                spans.push(Span::styled("● ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)));
            } else {
                spans.push(Span::styled("○ ", Style::default().fg(Color::DarkGray)));
            }

            // Short hash
            spans.push(Span::styled(format!("{} ", commit.short_id), Style::default().fg(Color::Yellow)));

            // References (branches / tags)
            if !commit.refs.is_empty() {
                let refs_str = format!("({}) ", commit.refs.join(", "));
                spans.push(Span::styled(refs_str, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)));
            }

            // Summary message
            spans.push(Span::styled(commit.summary.clone(), Style::default().fg(Color::White)));

            // Metadata: Author & Time (only show if not too cramped, but we'll put it in dark gray)
            spans.push(Span::styled(
                format!(" - {} [{}]", commit.author, commit.time),
                Style::default().fg(Color::DarkGray),
            ));

            let mut line = Line::from(spans);
            if idx == app.selected_commit_idx {
                line = line.style(Style::default().bg(Color::Rgb(40, 40, 40)));
            }
            ListItem::new(line)
        })
        .collect();

    let mut list_state = ratatui::widgets::ListState::default();
    list_state.select(Some(app.selected_commit_idx));

    let commits_list = List::new(commit_items)
        .block(commits_block)
        .highlight_symbol("> ")
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));
    
    frame.render_stateful_widget(commits_list, main_chunks[0], &mut list_state);

    // 3b. Diff View (Right)
    let diff_border_style = if app.active_pane == ActivePane::Diff {
        Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let diff_border_type = if app.active_pane == ActivePane::Diff {
        BorderType::Double
    } else {
        BorderType::Rounded
    };

    let diff_block = Block::default()
        .borders(Borders::ALL)
        .border_type(diff_border_type)
        .border_style(diff_border_style)
        .title(" Commit Details & Diff ");

    let diff_lines: Vec<Line> = app
        .diff_content
        .lines()
        .map(|line| {
            if line.starts_with('+') {
                Line::from(Span::styled(line, Style::default().fg(Color::Green)))
            } else if line.starts_with('-') {
                Line::from(Span::styled(line, Style::default().fg(Color::Red)))
            } else if line.starts_with("File:") {
                Line::from(Span::styled(line, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)))
            } else {
                Line::from(Span::styled(line, Style::default().fg(Color::Gray)))
            }
        })
        .collect();

    let diff_paragraph = Paragraph::new(diff_lines)
        .block(diff_block)
        .wrap(Wrap { trim: false })
        .scroll((app.diff_scroll, 0));
    frame.render_widget(diff_paragraph, main_chunks[1]);

    // 4. Render Footer (Status Bar / Keybindings)
    let footer_text = Line::from(vec![
        Span::styled("Tab", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::styled(" Switch Pane | ", Style::default().fg(Color::White)),
        Span::styled("o", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::styled(" Switch Folder | ", Style::default().fg(Color::White)),
        Span::styled("↑↓ / j/k", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::styled(" Move Commit/Scroll | ", Style::default().fg(Color::White)),
        Span::styled("q / Esc", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::styled(" Quit", Style::default().fg(Color::White)),
    ]);
    let footer = Paragraph::new(footer_text).style(Style::default().bg(Color::Rgb(30, 30, 30)));
    frame.render_widget(footer, chunks[2]);

    // 5. Render Folder Selector Overlay if active
    if app.show_folder_selector {
        let popup_area = centered_rect(70, 70, frame.area());
        frame.render_widget(Clear, popup_area);

        let popup_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .border_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .title(" Switch Directory / Open Git Repository ");
            
        let popup_inner = popup_block.inner(popup_area);
        frame.render_widget(popup_block, popup_area);

        let popup_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // Path text
                Constraint::Min(3),    // Directory list
                Constraint::Length(1), // Error info
                Constraint::Length(2), // Help text
            ])
            .split(popup_inner);

        // Path header
        let path_text = vec![Line::from(vec![
            Span::styled("Current Directory: ", Style::default().fg(Color::DarkGray)),
            Span::styled(app.folder_selector_path.to_string_lossy().to_string(), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ])];
        let path_paragraph = Paragraph::new(path_text);
        frame.render_widget(path_paragraph, popup_chunks[0]);

        // Directory list
        let dir_items: Vec<ListItem> = app
            .folder_entries
            .iter()
            .enumerate()
            .map(|(idx, path)| {
                let name = if let Some(parent) = app.folder_selector_path.parent() {
                    if idx == 0 && path == parent {
                        ".. (parent directory)".to_string()
                    } else {
                        path.file_name().unwrap_or_default().to_string_lossy().to_string()
                    }
                } else {
                    path.file_name().unwrap_or_default().to_string_lossy().to_string()
                };

                let mut spans = Vec::new();
                if idx == app.folder_selected_idx {
                    spans.push(Span::styled("➤ ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
                    spans.push(Span::styled(name, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)));
                } else {
                    spans.push(Span::styled("  ", Style::default()));
                    spans.push(Span::styled(name, Style::default().fg(Color::Gray)));
                }

                let mut line = Line::from(spans);
                if idx == app.folder_selected_idx {
                    line = line.style(Style::default().bg(Color::Rgb(50, 50, 50)));
                }
                ListItem::new(line)
            })
            .collect();

        let mut list_state = ratatui::widgets::ListState::default();
        list_state.select(Some(app.folder_selected_idx));

        let dir_list = List::new(dir_items)
            .block(Block::default().borders(Borders::NONE));
        frame.render_stateful_widget(dir_list, popup_chunks[1], &mut list_state);

        // Error message or notice
        if let Some(ref err) = app.error_message {
            let error_text = Paragraph::new(Span::styled(format!("⚠️ {}", err), Style::default().fg(Color::Red)));
            frame.render_widget(error_text, popup_chunks[2]);
        } else {
            // Check if current highlighted folder is a Git repo or contains .git
            let highlighted_path = if !app.folder_entries.is_empty() {
                Some(&app.folder_entries[app.folder_selected_idx])
            } else {
                None
            };
            let is_git = highlighted_path.map(|p| p.join(".git").exists()).unwrap_or(false)
                || app.folder_selector_path.join(".git").exists();
            
            let status_span = if is_git {
                Span::styled("✓ Selected folder is a Git Repository", Style::default().fg(Color::Green))
            } else {
                Span::styled("ℹ Highlighted folder is not a Git Repository yet", Style::default().fg(Color::DarkGray))
            };
            frame.render_widget(Paragraph::new(status_span), popup_chunks[2]);
        }

        // Help footer
        let help_text = Line::from(vec![
            Span::styled("Enter", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(" Navigate | ", Style::default().fg(Color::White)),
            Span::styled("Space / s", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled(" Open Repo | ", Style::default().fg(Color::White)),
            Span::styled("Esc / o", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled(" Cancel", Style::default().fg(Color::White)),
        ]);
        frame.render_widget(Paragraph::new(help_text), popup_chunks[3]);
    }
}

// Centered rect helper function
fn centered_rect(percent_x: u16, percent_y: u16, r: ratatui::layout::Rect) -> ratatui::layout::Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
