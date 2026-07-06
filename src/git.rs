use anyhow::Result;
use git2::{Repository, Oid};
use std::path::Path;

#[derive(Clone, Debug)]
pub struct CommitInfo {
    pub id: Oid,
    pub short_id: String,
    pub summary: String,
    pub author: String,
    pub time: String,
    pub parents: Vec<Oid>,
    pub refs: Vec<String>,
}

pub struct GitRepo {
    pub repo: Repository,
}

impl GitRepo {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let repo = Repository::discover(path)?;
        Ok(Self { repo })
    }

    pub fn get_commits(&self) -> Result<Vec<CommitInfo>> {
        let mut revwalk = self.repo.revwalk()?;
        // Configure revwalk to show commits from all branches/references
        revwalk.push_head().or_else(|_| {
            // If HEAD doesn't exist (empty repo), try pushing other refs
            if let Ok(references) = self.repo.references() {
                for reference in references.flatten() {
                    if reference.is_branch() || reference.is_tag() {
                        let _ = revwalk.push(reference.target().unwrap());
                    }
                }
            }
            Ok::<(), git2::Error>(())
        })?;
        
        // Use topological sorting and reverse to get a sensible graph structure (or time-based)
        revwalk.set_sorting(git2::Sort::TOPOLOGICAL | git2::Sort::TIME)?;

        // Map references to Oids for easy lookup
        let mut ref_map = std::collections::HashMap::new();
        if let Ok(references) = self.repo.references() {
            for reference in references.flatten() {
                if let Some(target) = reference.target() {
                    let name = reference.shorthand().unwrap_or("").to_string();
                    ref_map.entry(target).or_insert_with(Vec::new).push(name);
                }
            }
        }

        let mut commits = Vec::new();
        for id in revwalk {
            let id = id?;
            if let Ok(commit) = self.repo.find_commit(id) {
                let short_id = commit.as_object().short_id()?.as_str().unwrap_or("").to_string();
                let summary = commit.summary().unwrap_or("").to_string();
                let author = commit.author().name().unwrap_or("Unknown").to_string();
                
                // Format commit time
                let datetime = chrono::DateTime::from_timestamp(commit.time().seconds(), 0)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                    .unwrap_or_else(|| "Unknown".to_string());

                let parents = commit.parent_ids().collect();
                let refs = ref_map.get(&id).cloned().unwrap_or_default();

                commits.push(CommitInfo {
                    id,
                    short_id,
                    summary,
                    author,
                    time: datetime,
                    parents,
                    refs,
                });
            }
        }
        Ok(commits)
    }

    pub fn get_diff(&self, commit_id: Oid) -> Result<String> {
        let commit = self.repo.find_commit(commit_id)?;
        let commit_tree = commit.tree()?;
        
        let parent_tree = if commit.parent_count() > 0 {
            let parent = commit.parent(0)?;
            Some(parent.tree()?)
        } else {
            None
        };

        let diff = self.repo.diff_tree_to_tree(
            parent_tree.as_ref(),
            Some(&commit_tree),
            None,
        )?;

        let mut diff_text = String::new();
        // Custom formatting or standard git diff formatting
        let mut current_file = String::new();
        
        diff.print(git2::DiffFormat::Patch, |delta, _hunk, line| {
            let old_file = delta.old_file().path().and_then(|p| p.to_str()).unwrap_or("");
            let new_file = delta.new_file().path().and_then(|p| p.to_str()).unwrap_or("");
            
            if current_file != new_file {
                current_file = new_file.to_string();
                diff_text.push_str(&format!("File: {} -> {}\n", old_file, new_file));
            }

            let origin = line.origin();
            let line_str = std::str::from_utf8(line.content()).unwrap_or("");
            match origin {
                '+' | '>' => diff_text.push_str(&format!("+{}", line_str)),
                '-' | '<' => diff_text.push_str(&format!("-{}", line_str)),
                ' ' => diff_text.push_str(&format!(" {}", line_str)),
                _ => {}
            }
            true
        })?;

        if diff_text.is_empty() {
            Ok("No changes or merge commit (no diff generated).".to_string())
        } else {
            Ok(diff_text)
        }
    }
}
