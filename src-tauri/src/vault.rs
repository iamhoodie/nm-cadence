use crate::models::*;
use chrono::{Local, NaiveDate};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Parsing
// ---------------------------------------------------------------------------

/// Split a markdown document into (frontmatter, body).
/// Frontmatter is the block between the first `---` fences.
pub fn parse_frontmatter(content: &str) -> (Option<Frontmatter>, String) {
    if let Some(stripped) = content.strip_prefix("---") {
        if let Some(end) = stripped.find("\n---") {
            let fm_str = stripped[..end].trim();
            let body = stripped[end + 4..].trim_start_matches('\n').to_string();
            let fm = serde_yaml::from_str::<Frontmatter>(fm_str).ok();
            return (fm, body);
        }
    }
    (None, content.to_string())
}

/// Split a `## ` heading like "2026-06-16 — Roadmap" into (date, title).
fn split_heading(h: &str) -> (String, String) {
    let h = h.trim();
    for sep in [" — ", " – ", " - "] {
        if let Some(idx) = h.find(sep) {
            return (
                h[..idx].trim().to_string(),
                h[idx + sep.len()..].trim().to_string(),
            );
        }
    }
    (h.to_string(), String::new())
}

/// Parse the body of a person file into conversations.
/// Each conversation starts at a `## ` heading; `- [ ]` / `- [x]` lines
/// become action items, everything else is body prose.
pub fn parse_conversations(body: &str) -> Vec<Conversation> {
    let mut out: Vec<Conversation> = Vec::new();
    let mut cur: Option<Conversation> = None;
    let mut body_lines: Vec<String> = Vec::new();

    let flush = |cur: &mut Option<Conversation>, body_lines: &mut Vec<String>, out: &mut Vec<Conversation>| {
        if let Some(mut c) = cur.take() {
            c.body = body_lines.join("\n").trim().to_string();
            out.push(c);
        }
        body_lines.clear();
    };

    for line in body.lines() {
        if let Some(h) = line.strip_prefix("## ") {
            flush(&mut cur, &mut body_lines, &mut out);
            let (date, title) = split_heading(h);
            cur = Some(Conversation { date, title, body: String::new(), actions: Vec::new() });
        } else if let Some(c) = cur.as_mut() {
            let t = line.trim_start();
            if let Some(rest) = t.strip_prefix("- [ ] ") {
                c.actions.push(ActionItem { text: rest.trim().to_string(), done: false });
            } else if let Some(rest) = t
                .strip_prefix("- [x] ")
                .or_else(|| t.strip_prefix("- [X] "))
            {
                c.actions.push(ActionItem { text: rest.trim().to_string(), done: true });
            } else {
                body_lines.push(line.to_string());
            }
        }
    }
    flush(&mut cur, &mut body_lines, &mut out);
    out
}

fn slug_from_path(path: &Path) -> String {
    path.file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default()
}

fn person_from_file(path: &Path) -> io::Result<Person> {
    let content = fs::read_to_string(path)?;
    let (fm, body) = parse_frontmatter(&content);
    let conversations = parse_conversations(&body);
    let last_met = conversations.first().map(|c| c.date.clone());
    let fm = fm.unwrap_or(Frontmatter {
        name: slug_from_path(path),
        role: String::new(),
        bio: String::new(),
        color: String::new(),
        group: String::new(),
        birthday: String::new(),
    });
    Ok(Person {
        slug: slug_from_path(path),
        name: fm.name,
        role: fm.role,
        bio: fm.bio,
        color: fm.color,
        group: fm.group,
        birthday: fm.birthday,
        last_met,
        conversations,
    })
}

// ---------------------------------------------------------------------------
// Vault operations
// ---------------------------------------------------------------------------

pub fn people_dir(vault: &Path) -> PathBuf {
    vault.join("people")
}

pub fn tasks_file(vault: &Path) -> PathBuf {
    vault.join("tasks.md")
}

pub fn legacy_settings_file(vault: &Path) -> PathBuf {
    vault.join("settings.json")
}

pub fn list_people(vault: &Path) -> io::Result<Vec<Person>> {
    let dir = people_dir(vault);
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut people = Vec::new();
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.extension().and_then(|e| e.to_str()) == Some("md") {
            if let Ok(p) = person_from_file(&path) {
                people.push(p);
            }
        }
    }
    people.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(people)
}

pub fn get_person(vault: &Path, slug: &str) -> io::Result<Person> {
    person_from_file(&people_dir(vault).join(format!("{slug}.md")))
}

/// Render a conversation back to markdown.
fn render_conversation(c: &Conversation) -> String {
    let mut s = format!("## {} — {}\n\n", c.date, c.title);
    if !c.body.trim().is_empty() {
        s.push_str(c.body.trim());
        s.push_str("\n\n");
    }
    for a in &c.actions {
        let mark = if a.done { "x" } else { " " };
        s.push_str(&format!("- [{}] {}\n", mark, a.text));
    }
    if !c.actions.is_empty() {
        s.push('\n');
    }
    s
}

/// Prepend a new conversation to a person's file (newest first).
pub fn append_conversation(vault: &Path, slug: &str, conv: &Conversation) -> io::Result<()> {
    let path = people_dir(vault).join(format!("{slug}.md"));
    let content = fs::read_to_string(&path)?;
    let (fm_raw, body) = split_raw_frontmatter(&content);
    let new_body = format!("{}\n{}", render_conversation(conv).trim_end(), body.trim_start());
    let out = match fm_raw {
        Some(fm) => format!("---\n{}\n---\n\n{}\n", fm.trim(), new_body.trim()),
        None => format!("{}\n", new_body.trim()),
    };
    fs::write(path, out)
}

/// Like parse_frontmatter but returns the raw YAML string (so we can rewrite the body untouched).
fn split_raw_frontmatter(content: &str) -> (Option<String>, String) {
    if let Some(stripped) = content.strip_prefix("---") {
        if let Some(end) = stripped.find("\n---") {
            let fm_str = stripped[..end].trim().to_string();
            let body = stripped[end + 4..].trim_start_matches('\n').to_string();
            return (Some(fm_str), body);
        }
    }
    (None, content.to_string())
}

// ---------------------------------------------------------------------------
// Tasks
// ---------------------------------------------------------------------------

const COLUMNS: [(&str, &str); 3] = [("To do", "todo"), ("In progress", "doing"), ("Done", "done")];

fn parse_task_meta(line: &str) -> (String, Task) {
    // line content after "- [ ] " / "- [x] "
    let mut person = String::new();
    let mut due = String::new();
    let mut due_time = String::new();
    let mut priority = String::new();
    let mut title_parts: Vec<&str> = Vec::new();
    for tok in line.split_whitespace() {
        if let Some(v) = tok.strip_prefix("@person:") {
            person = v.replace('_', " ");
        } else if let Some(v) = tok.strip_prefix("@due:") {
            due = v.to_string();
        } else if let Some(v) = tok.strip_prefix("@time:") {
            due_time = v.to_string();
        } else if let Some(v) = tok.strip_prefix("@priority:") {
            priority = v.to_string();
        } else if tok.starts_with("@done:") || tok.starts_with("@archived:") {
            continue;
        } else {
            title_parts.push(tok);
        }
    }
    (
        title_parts.join(" "),
        Task {
            title: String::new(),
            description: String::new(),
            person,
            due,
            due_time,
            priority,
            column: String::new(),
            done: false,
            completed_at: String::new(),
            archived: false,
        },
    )
}

pub fn list_tasks(vault: &Path, settings: &AppSettings) -> io::Result<Vec<Task>> {
    let path = tasks_file(vault);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(path)?;
    let mut tasks = Vec::new();
    let mut column = "todo".to_string();
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        if let Some(h) = line.strip_prefix("## ") {
            let h = h.trim();
            column = COLUMNS
                .iter()
                .find(|(label, _)| label.eq_ignore_ascii_case(h))
                .map(|(_, key)| key.to_string())
                .unwrap_or_else(|| h.to_lowercase());
            i += 1;
            continue;
        }
        let t = line.trim_start();
        let (done, rest) = if let Some(r) = t.strip_prefix("- [ ] ") {
            (false, r)
        } else if let Some(r) = t.strip_prefix("- [x] ").or_else(|| t.strip_prefix("- [X] ")) {
            (true, r)
        } else {
            i += 1;
            continue;
        };
        let (title, mut task) = parse_task_meta(rest);
        for tok in rest.split_whitespace() {
            if let Some(v) = tok.strip_prefix("@done:") {
                task.completed_at = v.to_string();
            } else if let Some(v) = tok.strip_prefix("@archived:") {
                task.archived = v.eq_ignore_ascii_case("true");
            }
        }
        task.title = title;
        task.column = column.clone();
        task.done = done;
        // peek at the next line for an optional description
        if i + 1 < lines.len() {
            if let Some(desc) = lines[i + 1].strip_prefix("@desc:") {
                task.description = desc.to_string();
                i += 1; // consume the description line
            }
        }
        tasks.push(task);
        i += 1;
    }
    let tasks = apply_auto_archive(tasks, &settings);
    write_tasks_if_changed(vault, &tasks)?;
    Ok(tasks)
}

fn render_tasks(tasks: &[Task]) -> String {
    let mut out = String::from("# Tasks\n\n");
    for (label, key) in COLUMNS {
        out.push_str(&format!("## {}\n\n", label));
        for t in tasks.iter().filter(|t| t.column == key) {
            let mark = if t.done { "x" } else { " " };
            let mut line = format!("- [{}] {}", mark, t.title);
            if !t.person.is_empty() {
                line.push_str(&format!(" @person:{}", t.person.replace(' ', "_")));
            }
            if !t.due.is_empty() {
                line.push_str(&format!(" @due:{}", t.due));
            }
            if !t.due_time.is_empty() {
                line.push_str(&format!(" @time:{}", t.due_time));
            }
            if !t.priority.is_empty() {
                line.push_str(&format!(" @priority:{}", t.priority));
            }
            if !t.completed_at.is_empty() {
                line.push_str(&format!(" @done:{}", t.completed_at));
            }
            if t.archived {
                line.push_str(" @archived:true");
            }
            out.push_str(&line);
            out.push('\n');
            if !t.description.is_empty() {
                let desc_inline = t.description.replace(['\n', '\r'], "");
                out.push_str(&format!("@desc:{}\n", desc_inline));
            }
        }
        out.push('\n');
    }
    out.trim_end().to_string() + "\n"
}

fn write_tasks_if_changed(vault: &Path, tasks: &[Task]) -> io::Result<()> {
    let next = render_tasks(tasks);
    let path = tasks_file(vault);
    if path.exists() {
        let current = fs::read_to_string(&path)?;
        if current == next {
            return Ok(());
        }
    }
    fs::write(path, next)
}

fn apply_auto_archive(mut tasks: Vec<Task>, settings: &AppSettings) -> Vec<Task> {
    if !settings.auto_archive_done {
        return tasks;
    }

    let today = Local::now().date_naive();
    let archive_after = settings.auto_archive_days.max(1) as i64;

    for task in &mut tasks {
        if task.column != "done" || task.archived || task.completed_at.is_empty() {
            continue;
        }

        if let Ok(done_date) = NaiveDate::parse_from_str(&task.completed_at, "%Y-%m-%d") {
            if (today - done_date).num_days() >= archive_after {
                task.archived = true;
            }
        }
    }

    tasks
}

pub fn save_tasks(vault: &Path, tasks: &[Task], settings: &AppSettings) -> io::Result<()> {
    let tasks = apply_auto_archive(tasks.to_vec(), &settings);
    write_tasks_if_changed(vault, &tasks)
}

/// Ensure the vault folder structure exists.
pub fn ensure_vault(vault: &Path) -> io::Result<()> {
    fs::create_dir_all(people_dir(vault))?;
    if !tasks_file(vault).exists() {
        save_tasks(vault, &[], &AppSettings::default())?;
    }
    Ok(())
}

pub fn read_legacy_app_settings(vault: &Path) -> io::Result<AppSettings> {
    let path = legacy_settings_file(vault);
    if !path.exists() {
        return Ok(AppSettings::default());
    }
    let content = fs::read_to_string(path)?;
    serde_json::from_str(&content).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
}

// ---------------------------------------------------------------------------
// Person CRUD
// ---------------------------------------------------------------------------

fn make_slug(name: &str) -> String {
    let s: String = name
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    s.split('-')
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

fn write_frontmatter(fm: &Frontmatter) -> io::Result<String> {
    serde_yaml::to_string(fm)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
        .map(|s| s.trim_end().to_string())
}

pub fn create_person(vault: &Path, fm: &Frontmatter) -> io::Result<Person> {
    let dir = people_dir(vault);
    fs::create_dir_all(&dir)?;
    let slug = make_slug(&fm.name);
    if slug.is_empty() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "name produces empty slug"));
    }
    let path = dir.join(format!("{slug}.md"));
    if path.exists() {
        return Err(io::Error::new(io::ErrorKind::AlreadyExists, "person already exists"));
    }
    let yaml = write_frontmatter(fm)?;
    fs::write(&path, format!("---\n{yaml}\n---\n"))?;
    person_from_file(&path)
}

pub fn delete_person(vault: &Path, slug: &str) -> io::Result<()> {
    fs::remove_file(people_dir(vault).join(format!("{slug}.md")))
}

pub fn update_person(vault: &Path, slug: &str, fm: &Frontmatter) -> io::Result<Person> {
    let path = people_dir(vault).join(format!("{slug}.md"));
    let content = fs::read_to_string(&path)?;
    let (_, body) = split_raw_frontmatter(&content);
    let yaml = write_frontmatter(fm)?;
    let body_trim = body.trim();
    let out = if body_trim.is_empty() {
        format!("---\n{yaml}\n---\n")
    } else {
        format!("---\n{yaml}\n---\n\n{body_trim}\n")
    };
    fs::write(&path, out)?;
    person_from_file(&path)
}

// ---------------------------------------------------------------------------
// Folders
// ---------------------------------------------------------------------------

fn folders_file(vault: &Path) -> PathBuf {
    vault.join("folders.json")
}

#[derive(serde::Deserialize)]
#[serde(untagged)]
enum StoredFolder {
    LegacyName(String),
    Full(Folder),
}

fn normalize_folder_color(color: &str) -> String {
    let trimmed = color.trim();
    if trimmed.is_empty() {
        "#6b7d9c".to_string()
    } else {
        trimmed.to_string()
    }
}

pub fn list_folders(vault: &Path) -> io::Result<Vec<Folder>> {
    let path = folders_file(vault);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(path)?;
    let folders: Vec<Folder> = serde_json::from_str::<Vec<StoredFolder>>(&content)
        .map(|items| {
            items
                .into_iter()
                .filter_map(|item| match item {
                    StoredFolder::LegacyName(name) => {
                        let name = name.trim().to_string();
                        if name.is_empty() {
                            None
                        } else {
                            Some(Folder {
                                name,
                                color: "#6b7d9c".to_string(),
                                exclude_checkin: false,
                            })
                        }
                    }
                    StoredFolder::Full(folder) => {
                        let name = folder.name.trim().to_string();
                        if name.is_empty() {
                            None
                        } else {
                            Some(Folder {
                                name,
                                color: normalize_folder_color(&folder.color),
                                exclude_checkin: folder.exclude_checkin,
                            })
                        }
                    }
                })
                .collect()
        })
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    Ok(folders)
}

fn write_folders(vault: &Path, folders: &[Folder]) -> io::Result<()> {
    let json = serde_json::to_string_pretty(folders)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    fs::write(folders_file(vault), json)
}

pub fn create_folder(vault: &Path, name: &str, color: &str, exclude_checkin: bool) -> io::Result<()> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Ok(());
    }
    let mut folders = list_folders(vault)?;
    if folders.iter().any(|folder| folder.name == name) {
        return Ok(());
    }
    folders.push(Folder {
        name,
        color: normalize_folder_color(color),
        exclude_checkin,
    });
    write_folders(vault, &folders)
}

pub fn delete_folder(vault: &Path, name: &str) -> io::Result<()> {
    let mut folders = list_folders(vault)?;
    folders.retain(|folder| folder.name != name);
    write_folders(vault, &folders)
}

pub fn update_folder(vault: &Path, name: &str, next_name: &str, color: &str, exclude_checkin: bool) -> io::Result<()> {
    let next_name = next_name.trim();
    if next_name.is_empty() {
        return Ok(());
    }

    let mut folders = list_folders(vault)?;
    if next_name != name && folders.iter().any(|folder| folder.name == next_name) {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "folder already exists",
        ));
    }

    if let Some(folder) = folders.iter_mut().find(|folder| folder.name == name) {
        folder.name = next_name.to_string();
        folder.color = normalize_folder_color(color);
        folder.exclude_checkin = exclude_checkin;
    }
    write_folders(vault, &folders)?;

    let people = list_people(vault)?;
    for person in people.into_iter().filter(|person| person.group == name) {
        let updated = Frontmatter {
            name: person.name,
            role: person.role,
            bio: person.bio,
            color: person.color,
            group: next_name.to_string(),
            birthday: person.birthday,
        };
        update_person(vault, &person.slug, &updated)?;
    }

    Ok(())
}

pub fn reorder_folders(vault: &Path, ordered_names: &[String]) -> io::Result<()> {
    let folders = list_folders(vault)?;
    let mut remaining = folders;
    let mut ordered: Vec<Folder> = Vec::new();

    for name in ordered_names {
        if let Some(index) = remaining.iter().position(|folder| folder.name == *name) {
            ordered.push(remaining.remove(index));
        }
    }

    ordered.extend(remaining);
    write_folders(vault, &ordered)
}

// ---------------------------------------------------------------------------
// Conversation update
// ---------------------------------------------------------------------------

pub fn update_conversation(
    vault: &Path,
    slug: &str,
    orig_date: &str,
    orig_title: &str,
    updated: &Conversation,
) -> io::Result<()> {
    let path = people_dir(vault).join(format!("{slug}.md"));
    let content = fs::read_to_string(&path)?;
    let (fm_raw, body) = split_raw_frontmatter(&content);
    let conversations = parse_conversations(&body);
    let new_body: String = conversations
        .iter()
        .map(|c| {
            if c.date == orig_date && c.title == orig_title {
                render_conversation(updated)
            } else {
                render_conversation(c)
            }
        })
        .collect();
    let out = match fm_raw {
        Some(fm) => {
            let body_trim = new_body.trim();
            if body_trim.is_empty() {
                format!("---\n{}\n---\n", fm.trim())
            } else {
                format!("---\n{}\n---\n\n{body_trim}\n", fm.trim())
            }
        }
        None => format!("{}\n", new_body.trim()),
    };
    fs::write(path, out)
}

// ---------------------------------------------------------------------------
// Conversation delete
// ---------------------------------------------------------------------------

pub fn delete_conversation(vault: &Path, slug: &str, date: &str, title: &str) -> io::Result<()> {
    let path = people_dir(vault).join(format!("{slug}.md"));
    let content = fs::read_to_string(&path)?;
    let (fm_raw, body) = split_raw_frontmatter(&content);
    let conversations = parse_conversations(&body);
    let remaining: Vec<&Conversation> = conversations
        .iter()
        .filter(|c| !(c.date == date && c.title == title))
        .collect();
    let new_body: String = remaining.iter().map(|c| render_conversation(c)).collect();
    let out = match fm_raw {
        Some(fm) => {
            let body_trim = new_body.trim();
            if body_trim.is_empty() {
                format!("---\n{}\n---\n", fm.trim())
            } else {
                format!("---\n{}\n---\n\n{body_trim}\n", fm.trim())
            }
        }
        None => format!("{}\n", new_body.trim()),
    };
    fs::write(path, out)
}
