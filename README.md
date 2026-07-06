# ⚡ GitVisual TUI

`GitVisual` is a high-performance, terminal-based Git repository history and diff viewer built in **Rust** using **Ratatui** and **Crossterm**. It offers developers a blazing-fast, keyboard-driven interface to explore local git commits, branches, tags, and diff changes, with the ability to switch directories and open different git repositories on the fly.

---

## 🚀 Key Features

*   **⚡ Blazing Fast Log Exploration**: Streamlined commits list featuring short hashes, active reference tracking (branches/tags), and clear commit summaries.
*   **🔍 Interactive Diff Viewer**: Instantaneous diff generation for any selected commit, complete with syntax-highlighted additions (`+` in green) and deletions (`-` in red).
*   **📂 Hot-Switch Directories**: Toggle an in-app file browser popup to traverse directories and dynamically reload any local Git repository.
*   **⌨️ Full Keyboard Navigation**: Highly responsive Vim-like keybindings (`j`/`k` or arrows) and instant context switching.
*   **🛠️ Lightweight & Zero Configuration**: Compiled to a single binary with zero external dependencies besides local `libgit2`.

---

## 🛠️ Tech Stack & Architecture

- **Core Engine**: Rust (Edition 2024)
- **UI Framework**: [Ratatui v0.29.0](https://github.com/ratatui-org/ratatui)
- **Terminal Control**: [Crossterm v0.28.1](https://github.com/crossterm-rs/crossterm)
- **Git Integration**: [git2-rs v0.19.0](https://github.com/rust-lang/git2-rs) (Safe bindings to `libgit2`)

---

## ⌨️ Control Manual

| Key | Action |
| :--- | :--- |
| **`Tab`** | Switch focus between Commits History and Diff View panels |
| **`o`** | Toggle the File Browser / Repository Selector overlay |
| **`↑ / ↓`** (or **`k / j`**) | Navigate active commits list, scroll diffs, or browse folders |
| **`Enter`** | Enter directory (in folder browser mode) |
| **`Space / s`** | Open highlighted directory as the active Git repo (in folder browser mode) |
| **`Esc / o`** | Cancel and close folder browser popup |
| **`q / Esc`** | Quit GitVisual TUI |

---

## 📦 Setup & Installation

### Prerequisites
Make sure you have Rust/Cargo installed on your system. If not, install via:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Installation
Clone and build the binary:
```bash
git clone https://github.com/GitVisual/GitVisual.git
cd GitVisual
cargo build --release
```

### Execution
Run directly in the current directory:
```bash
cargo run
```
Or open a specific git repository by passing the path:
```bash
cargo run -- /path/to/target/git/repo
```

---

## 📄 License
Distributed under the **MIT License**. See [LICENSE](LICENSE) for more information.
