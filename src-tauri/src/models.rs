use serde::{Deserialize, Serialize};

/// A single action item / checkbox inside a conversation note.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ActionItem {
    pub text: String,
    pub done: bool,
}

/// One logged 1:1 conversation (one `## YYYY-MM-DD — Title` section in a person file).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Conversation {
    pub date: String, // ISO date, e.g. "2026-06-16"
    pub title: String,
    pub body: String,
    pub actions: Vec<ActionItem>,
}

/// YAML frontmatter at the top of a person's markdown file.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Frontmatter {
    pub name: String,
    #[serde(default)]
    pub role: String,
    #[serde(default)]
    pub bio: String,
    #[serde(default)]
    pub color: String,
    #[serde(default)]
    pub group: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Folder {
    pub name: String,
    #[serde(default)]
    pub color: String,
    #[serde(default)]
    pub exclude_checkin: bool,
}

/// A person, assembled from one `people/<slug>.md` file.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Person {
    pub slug: String,
    pub name: String,
    pub role: String,
    pub bio: String,
    pub color: String,
    pub group: String,
    /// Date of the most recent conversation, if any.
    pub last_met: Option<String>,
    pub conversations: Vec<Conversation>,
}

/// A personal task from `tasks.md`.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    pub title: String,
    #[serde(default)]
    pub description: String, // rich HTML body; stored as `@desc:` line after the task line
    #[serde(default)]
    pub person: String, // links to a Person.name; "" if unlinked
    #[serde(default)]
    pub due: String, // free text or ISO date
    #[serde(default)]
    pub due_time: String, // optional HH:MM time
    #[serde(default)]
    pub priority: String, // "high" | "med" | "low"
    pub column: String,   // "todo" | "doing" | "done"
    pub done: bool,
    #[serde(default)]
    pub completed_at: String,
    #[serde(default)]
    pub archived: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct AppSettings {
    #[serde(default = "default_true")]
    pub auto_archive_done: bool,
    #[serde(default = "default_archive_days")]
    pub auto_archive_days: u32,
    #[serde(default = "default_stale_1on1_days")]
    pub stale_1on1_days: u32,
    #[serde(default = "default_true")]
    pub spell_check: bool,
}

fn default_true() -> bool {
    true
}

fn default_archive_days() -> u32 {
    7
}

fn default_stale_1on1_days() -> u32 {
    14
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            auto_archive_done: true,
            auto_archive_days: default_archive_days(),
            stale_1on1_days: default_stale_1on1_days(),
            spell_check: true,
        }
    }
}
