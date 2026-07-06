use crate::git::{CommitInfo, GitRepo};
use anyhow::Result;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivePane {
    Commits,
    Diff,
}

pub struct App {
    pub git_repo: GitRepo,
    pub commits: Vec<CommitInfo>,
    pub selected_commit_idx: usize,
    pub diff_content: String,
    pub diff_scroll: u16,
    pub active_pane: ActivePane,
    pub should_quit: bool,
    pub error_message: Option<String>,
    pub show_folder_selector: bool,
    pub folder_selector_path: PathBuf,
    pub folder_entries: Vec<PathBuf>,
    pub folder_selected_idx: usize,
}

impl App {
    pub fn new(repo_path: &str) -> Result<Self> {
        let git_repo = GitRepo::open(repo_path)?;
        let mut app = Self {
            git_repo,
            commits: Vec::new(),
            selected_commit_idx: 0,
            diff_content: String::new(),
            diff_scroll: 0,
            active_pane: ActivePane::Commits,
            should_quit: false,
            error_message: None,
            show_folder_selector: false,
            folder_selector_path: PathBuf::new(),
            folder_entries: Vec::new(),
            folder_selected_idx: 0,
        };
        app.refresh_commits()?;
        Ok(app)
    }

    pub fn refresh_commits(&mut self) -> Result<()> {
        match self.git_repo.get_commits() {
            Ok(commits) => {
                self.commits = commits;
                if self.commits.is_empty() {
                    self.diff_content = "Repository has no commits.".to_string();
                } else {
                    self.selected_commit_idx = self.selected_commit_idx.min(self.commits.len() - 1);
                    self.update_diff()?;
                }
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to read commits: {}", e));
            }
        }
        Ok(())
    }

    pub fn update_diff(&mut self) -> Result<()> {
        if self.commits.is_empty() {
            self.diff_content = "No commits selected.".to_string();
            return Ok(());
        }
        let commit_id = self.commits[self.selected_commit_idx].id;
        match self.git_repo.get_diff(commit_id) {
            Ok(diff) => {
                self.diff_content = diff;
            }
            Err(e) => {
                self.diff_content = format!("Failed to generate diff: {}", e);
            }
        }
        self.diff_scroll = 0; // Reset scroll on change
        Ok(())
    }

    pub fn move_selection_up(&mut self) {
        if self.commits.is_empty() {
            return;
        }
        if self.selected_commit_idx > 0 {
            self.selected_commit_idx -= 1;
            let _ = self.update_diff();
        }
    }

    pub fn move_selection_down(&mut self) {
        if self.commits.is_empty() {
            return;
        }
        if self.selected_commit_idx < self.commits.len() - 1 {
            self.selected_commit_idx += 1;
            let _ = self.update_diff();
        }
    }

    pub fn scroll_diff_up(&mut self) {
        if self.diff_scroll > 0 {
            self.diff_scroll -= 1;
        }
    }

    pub fn scroll_diff_down(&mut self) {
        self.diff_scroll += 1;
    }

    pub fn toggle_pane(&mut self) {
        self.active_pane = match self.active_pane {
            ActivePane::Commits => ActivePane::Diff,
            ActivePane::Diff => ActivePane::Commits,
        };
    }

    pub fn toggle_folder_selector(&mut self) {
        self.show_folder_selector = !self.show_folder_selector;
        if self.show_folder_selector {
            let start_path = self.git_repo.repo.workdir()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| {
                    self.git_repo.repo.path().parent()
                        .map(|p| p.to_path_buf())
                        .unwrap_or_else(|| PathBuf::from("."))
                });
            // Try to resolve to absolute/canonical path
            self.folder_selector_path = std::fs::canonicalize(start_path)
                .unwrap_or_else(|_| PathBuf::from("."));
            let _ = self.load_folder_entries();
        }
    }

    pub fn load_folder_entries(&mut self) -> Result<()> {
        self.folder_entries.clear();
        self.folder_selected_idx = 0;

        // Add parent directory ".." option if parent exists
        if let Some(parent) = self.folder_selector_path.parent() {
            self.folder_entries.push(parent.to_path_buf());
        }

        // Read directory entries
        if let Ok(read_dir) = std::fs::read_dir(&self.folder_selector_path) {
            let mut entries: Vec<PathBuf> = read_dir
                .filter_map(|res| res.ok().map(|e| e.path()))
                .filter(|path| path.is_dir())
                .collect();
            // Sort entries alphabetically by file name
            entries.sort_by(|a, b| {
                let a_name = a.file_name().unwrap_or_default();
                let b_name = b.file_name().unwrap_or_default();
                a_name.cmp(b_name)
            });
            self.folder_entries.extend(entries);
        }

        Ok(())
    }

    pub fn folder_select_up(&mut self) {
        if self.folder_entries.is_empty() {
            return;
        }
        if self.folder_selected_idx > 0 {
            self.folder_selected_idx -= 1;
        }
    }

    pub fn folder_select_down(&mut self) {
        if self.folder_entries.is_empty() {
            return;
        }
        if self.folder_selected_idx < self.folder_entries.len() - 1 {
            self.folder_selected_idx += 1;
        }
    }

    pub fn folder_enter(&mut self) -> Result<()> {
        if self.folder_entries.is_empty() {
            return Ok(());
        }
        let target = self.folder_entries[self.folder_selected_idx].clone();
        if let Ok(canonical) = std::fs::canonicalize(target) {
            self.folder_selector_path = canonical;
        } else {
            self.folder_selector_path = self.folder_entries[self.folder_selected_idx].clone();
        }
        self.load_folder_entries()?;
        Ok(())
    }

    pub fn folder_open_repo(&mut self) -> Result<()> {
        let target_path = if !self.folder_entries.is_empty() {
            self.folder_entries[self.folder_selected_idx].clone()
        } else {
            self.folder_selector_path.clone()
        };

        match GitRepo::open(&target_path) {
            Ok(new_repo) => {
                self.git_repo = new_repo;
                self.selected_commit_idx = 0;
                self.refresh_commits()?;
                self.show_folder_selector = false;
                self.error_message = None;
            }
            Err(e) => {
                self.error_message = Some(format!("Not a git repository: {}", e));
            }
        }
        Ok(())
    }
}
