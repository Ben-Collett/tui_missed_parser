mod clipboard;
mod merge;
mod parse;

#[cfg(test)]
#[path = "tests/main.rs"]
mod tests;

use std::fs;
use std::io;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::{Frame, DefaultTerminal};

use merge::merge_all;
use parse::{parse_line, Entry};

const DEFAULT_DIR: &str = "Documents/logs";
const CURSOR_BG: Color = Color::Indexed(238);
const CURRENT_TRIGGER: Color = Color::LightMagenta;
const ALT_TRIGGER: Color = Color::DarkGray;

fn main() {
    let dir = parse_args();
    let files = match discover(&dir) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("failed to read {}: {e}", dir.display());
            std::process::exit(1);
        }
    };
    let app = App {
        dir,
        screen: Screen::Files(FilesScreen::new(files)),
    };
    if let Err(e) = run(app) {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn parse_args() -> PathBuf {
    let mut dir: Option<PathBuf> = None;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        if a == "--dir" || a == "-d" {
            if let Some(v) = args.next() {
                dir = Some(expand_home(&v));
            }
        } else if let Some(v) = a.strip_prefix("--dir=") {
            dir = Some(expand_home(v));
        }
    }
    dir.unwrap_or_else(|| {
        let home = std::env::var("HOME").unwrap_or_default();
        PathBuf::from(home).join(DEFAULT_DIR)
    })
}

fn expand_home(p: &str) -> PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        if p == "~" {
            return PathBuf::from(home);
        }
        if let Some(rest) = p.strip_prefix("~/") {
            return PathBuf::from(home).join(rest);
        }
    }
    PathBuf::from(p)
}

struct FileInfo {
    name: String,
    path: PathBuf,
    entries: usize,
    skipped: usize,
}

fn discover(dir: &Path) -> io::Result<Vec<FileInfo>> {
    let mut files = Vec::new();
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let Ok(entry) = entry else { continue };
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("log") || !path.is_file() {
                continue;
            }
            let (entries, skipped) = parse_log_file(&path);
            files.push(FileInfo {
                name: entry.file_name().to_string_lossy().into_owned(),
                path,
                entries,
                skipped,
            });
        }
    }
    files.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(files)
}

fn parse_log_file(path: &Path) -> (usize, usize) {
    let Ok(text) = fs::read_to_string(path) else {
        return (0, 0);
    };
    let mut entries = 0;
    let mut skipped = 0;
    for line in text.lines() {
        if parse_line(line).is_some() {
            entries += 1;
        } else {
            skipped += 1;
        }
    }
    (entries, skipped)
}

fn read_entries(path: &Path) -> Vec<Entry> {
    let Ok(text) = fs::read_to_string(path) else {
        return Vec::new();
    };
    text.lines().filter_map(parse_line).collect()
}

struct FilesScreen {
    files: Vec<FileInfo>,
    selected: Vec<bool>,
    cursor: usize,
    offset: usize,
    status: Option<String>,
}

impl FilesScreen {
    fn new(files: Vec<FileInfo>) -> Self {
        let status = if files.is_empty() {
            Some("no .log files found".to_string())
        } else {
            None
        };
        let selected = vec![false; files.len()];
        Self {
            files,
            selected,
            cursor: 0,
            offset: 0,
            status,
        }
    }
}

struct WordsScreen {
    words: Vec<WordEntry>,
    selected: Vec<bool>,
    trigger: Vec<usize>,
    cursor: usize,
    offset: usize,
    status: Option<String>,
    back: Option<FilesScreen>,
}

impl WordsScreen {
    fn new(words: Vec<WordEntry>, back: Option<FilesScreen>) -> Self {
        let n = words.len();
        Self {
            words,
            selected: vec![false; n],
            trigger: vec![0; n],
            cursor: 0,
            offset: 0,
            status: None,
            back,
        }
    }
}

use merge::WordEntry;

enum Screen {
    Files(FilesScreen),
    Words(WordsScreen),
}

struct App {
    dir: PathBuf,
    screen: Screen,
}

enum Nav {
    Continue,
    Quit,
    Back,
    Advance,
    Launch,
}

fn run(mut app: App) -> io::Result<()> {
    let mut term = ratatui::init();
    let res = event_loop(&mut term, &mut app);
    ratatui::restore();
    res
}

fn event_loop(term: &mut DefaultTerminal, app: &mut App) -> io::Result<()> {
    loop {
        term.draw(|f| render(f, app))?;
        let event = event::read()?;
        let Event::Key(ke) = event else { continue };
        if ke.kind != KeyEventKind::Press {
            continue;
        }
        let nav = match &mut app.screen {
            Screen::Files(fs) => handle_files_key(fs, ke.code),
            Screen::Words(ws) => handle_words_key(ws, ke.code),
        };
        match nav {
            Nav::Continue => {}
            Nav::Quit => return Ok(()),
            Nav::Back => {
                let back = match &mut app.screen {
                    Screen::Words(ws) => ws.back.take(),
                    Screen::Files(_) => None,
                };
                if let Some(fs) = back {
                    app.screen = Screen::Files(fs);
                }
            }
            Nav::Advance => {
                let has_selection = matches!(&app.screen, Screen::Files(fs) if fs.selected.iter().any(|&s| s));
                if !has_selection {
                    match &mut app.screen {
                        Screen::Files(fs) => {
                            fs.status = Some("no files selected - press space to mark files".to_string())
                        }
                        Screen::Words(_) => {}
                    }
                } else {
                    let entries = {
                        let Screen::Files(fs) = &app.screen else {
                            unreachable!()
                        };
                        let mut entries = Vec::new();
                        for (info, &sel) in fs.files.iter().zip(&fs.selected) {
                            if sel {
                                entries.extend(read_entries(&info.path));
                            }
                        }
                        entries
                    };
                    let words = merge_all(&entries);
                    let back = std::mem::replace(
                        &mut app.screen,
                        Screen::Files(FilesScreen::new(Vec::new())),
                    );
                    let Screen::Files(fs) = back else {
                        unreachable!()
                    };
                    app.screen = Screen::Words(WordsScreen::new(words, Some(fs)));
                }
            }
            Nav::Launch => {
                let pairs = match &app.screen {
                    Screen::Words(ws) => collect_pairs(ws),
                    Screen::Files(_) => unreachable!(),
                };
                finish(&pairs)?;
            }
        }
    }
}

fn handle_files_key(fs: &mut FilesScreen, code: KeyCode) -> Nav {
    match code {
        KeyCode::Char('q') | KeyCode::Esc => Nav::Quit,
        KeyCode::Char('j') | KeyCode::Down => {
            if fs.cursor + 1 < fs.files.len() {
                fs.cursor += 1;
            }
            Nav::Continue
        }
        KeyCode::Char('k') | KeyCode::Up => {
            fs.cursor = fs.cursor.saturating_sub(1);
            Nav::Continue
        }
        KeyCode::Char('g') => {
            if !fs.files.is_empty() {
                fs.cursor = fs.files.len() - 1;
            }
            Nav::Continue
        }
        KeyCode::Char('G') => {
            fs.cursor = 0;
            fs.offset = 0;
            Nav::Continue
        }
        KeyCode::Char(' ') => {
            if let Some(s) = fs.selected.get_mut(fs.cursor) {
                *s = !*s;
            }
            Nav::Continue
        }
        KeyCode::Enter => Nav::Advance,
        _ => Nav::Continue,
    }
}

fn handle_words_key(ws: &mut WordsScreen, code: KeyCode) -> Nav {
    match code {
        KeyCode::Char('q') => Nav::Quit,
        KeyCode::Esc => Nav::Back,
        KeyCode::Char('h') | KeyCode::Left => {
            if let Some(ts) = ws.trigger.get_mut(ws.cursor) {
                let n = ws.words[ws.cursor].triggers.len();
                if n > 0 {
                    *ts = (*ts + n - 1) % n;
                }
            }
            Nav::Continue
        }
        KeyCode::Char('l') | KeyCode::Right => {
            if let Some(ts) = ws.trigger.get_mut(ws.cursor) {
                let n = ws.words[ws.cursor].triggers.len();
                if n > 0 {
                    *ts = (*ts + 1) % n;
                }
            }
            Nav::Continue
        }
        KeyCode::Char('j') | KeyCode::Down => {
            if ws.cursor + 1 < ws.words.len() {
                ws.cursor += 1;
            }
            Nav::Continue
        }
        KeyCode::Char('k') | KeyCode::Up => {
            ws.cursor = ws.cursor.saturating_sub(1);
            Nav::Continue
        }
        KeyCode::Char('g') => {
            if !ws.words.is_empty() {
                ws.cursor = ws.words.len() - 1;
            }
            Nav::Continue
        }
        KeyCode::Char('G') => {
            ws.cursor = 0;
            ws.offset = 0;
            Nav::Continue
        }
        KeyCode::Char(' ') => {
            if let Some(s) = ws.selected.get_mut(ws.cursor) {
                *s = !*s;
            }
            Nav::Continue
        }
        KeyCode::Enter => {
            if ws.selected.iter().all(|&s| !s) {
                ws.status = Some("no words selected - press space to mark words".to_string());
                Nav::Continue
            } else {
                Nav::Launch
            }
        }
        _ => Nav::Continue,
    }
}

fn collect_pairs(ws: &WordsScreen) -> Vec<(String, String)> {
    let mut pairs = Vec::new();
    for (i, w) in ws.words.iter().enumerate() {
        if ws.selected.get(i).copied().unwrap_or(false) {
            if let Some(t) = w.triggers.get(ws.trigger[i]) {
                pairs.push((w.output.clone(), t.display.clone()));
            }
        }
    }
    pairs
}

fn format_buffer(pairs: &[(String, String)]) -> String {
    let mut sorted = pairs.to_vec();
    sorted.sort_by(|a, b| {
        a.0.to_lowercase()
            .cmp(&b.0.to_lowercase())
            .then_with(|| a.0.cmp(&b.0))
    });
    let width = sorted.iter().map(|p| p.0.chars().count()).max().unwrap_or(0);
    let mut body = String::new();
    for (out, trig) in &sorted {
        body.push_str(&format!("{out:<width$} {trig}\n"));
    }
    body.trim_end().to_string()
}

fn finish(pairs: &[(String, String)]) -> io::Result<()> {
    let sorted = {
        let mut s = pairs.to_vec();
        s.sort_by(|a, b| {
            a.0.to_lowercase()
                .cmp(&b.0.to_lowercase())
                .then_with(|| a.0.cmp(&b.0))
        });
        s
    };
    let body = format!("{}\n", format_buffer(pairs));
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let path = PathBuf::from(format!("/tmp/missed_{stamp}.txt"));
    fs::write(&path, body)?;
    let joined = sorted.iter().map(|p| p.0.as_str()).collect::<Vec<_>>().join(" ");
    let _ = clipboard::copy_to_clipboard(&joined);
    ratatui::restore();
    let editor = std::env::var("VISUAL")
        .ok()
        .or_else(|| std::env::var("EDITOR").ok())
        .unwrap_or_else(|| String::from("nvim"));
    Err(Command::new("sh")
        .arg("-c")
        .arg(format!("exec {editor} \"$1\""))
        .arg("sh")
        .arg(path)
        .exec())
}

fn render(f: &mut Frame<'_>, app: &mut App) {
    let [main, status, help] = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(f.area());

    let (title, lines, help_text) = match &mut app.screen {
        Screen::Files(fs) => render_files(fs, main, &app.dir),
        Screen::Words(ws) => render_words(ws, main),
    };

    f.render_widget(
        Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title(title)),
        main,
    );

    let status_text = match &app.screen {
        Screen::Files(fs) => fs.status.clone().unwrap_or_default(),
        Screen::Words(ws) => ws.status.clone().unwrap_or_default(),
    };
    if status_text.is_empty() {
        f.render_widget(Paragraph::new(""), status);
    } else {
        f.render_widget(
            Paragraph::new(Line::styled(status_text, Style::new().fg(Color::Yellow))),
            status,
        );
    }

    f.render_widget(
        Paragraph::new(Line::styled(help_text, Style::new().fg(Color::DarkGray))),
        help,
    );
}

fn render_files(fs: &mut FilesScreen, area: Rect, dir: &Path) -> (String, Vec<Line<'static>>, String) {
    let title = format!("select log files - {}", dir.display());
    let help = "space select | j/k or up/down move | g bottom | G top | enter parse | q/esc quit".to_string();
    let visible = area.height.saturating_sub(2) as usize;
    fs.offset = clamp_offset(fs.offset, fs.cursor, visible, fs.files.len());

    let mut lines = Vec::new();
    if fs.files.is_empty() {
        lines.push(Line::styled(
            "no .log files found",
            Style::new().fg(Color::Yellow),
        ));
        return (title, lines, help);
    }
    for i in fs.offset..fs.files.len().min(fs.offset + visible) {
        let info = &fs.files[i];
        let mut spans = vec![
            Span::styled(
                if fs.selected[i] { "[x]" } else { "[ ]" },
                if fs.selected[i] {
                    Style::new().fg(Color::Green)
                } else {
                    Style::new().fg(Color::DarkGray)
                },
            ),
            Span::raw("  "),
            Span::raw(info.name.clone()),
            Span::raw("  "),
            Span::styled(
                format!("{} entries", info.entries),
                Style::new().fg(Color::DarkGray),
            ),
        ];
        if info.skipped > 0 {
            spans.push(Span::styled(
                format!(" ({} skipped)", info.skipped),
                Style::new().fg(Color::Yellow),
            ));
        }
        let mut line = Line::from(spans);
        if i == fs.cursor {
            line = line.style(Style::new().bg(CURSOR_BG));
        }
        lines.push(line);
    }
    (title, lines, help)
}

fn render_words(ws: &mut WordsScreen, area: Rect) -> (String, Vec<Line<'static>>, String) {
    let title = "select words and triggers".to_string();
    let help = "space select | h/l or left/right trigger | j/k or up/down move | g bottom | G top | enter done | esc back | q quit".to_string();
    let visible = area.height.saturating_sub(2) as usize;
    ws.offset = clamp_offset(ws.offset, ws.cursor, visible, ws.words.len());

    let mut lines = Vec::new();
    if ws.words.is_empty() {
        lines.push(Line::styled(
            "no entries parsed from selected files",
            Style::new().fg(Color::Yellow),
        ));
        return (title, lines, help);
    }
    let max_out = ws.words.iter().map(|w| w.output.chars().count()).max().unwrap_or(0);
    for i in ws.offset..ws.words.len().min(ws.offset + visible) {
        let w = &ws.words[i];
        let mut spans = vec![
            Span::styled(
                if ws.selected[i] { "[x]" } else { "[ ]" },
                if ws.selected[i] {
                    Style::new().fg(Color::Green)
                } else {
                    Style::new().fg(Color::DarkGray)
                },
            ),
            Span::raw("  "),
            Span::raw(format!("{:<width$}", w.output, width = max_out)),
            Span::raw(" "),
        ];
        for (j, t) in w.triggers.iter().enumerate() {
            let style = if j == ws.trigger[i] {
                Style::new().fg(CURRENT_TRIGGER)
            } else {
                Style::new().fg(ALT_TRIGGER)
            };
            spans.push(Span::styled(t.display.clone(), style));
            spans.push(Span::raw(" "));
        }
        let cnt = format!("cnt {}", w.total);
        let used: usize = spans.iter().map(|s| s.content.len()).sum();
        let inner: usize = area.width.saturating_sub(2) as usize;
        let pad = inner.saturating_sub(used + cnt.len() + 1);
        if pad > 0 {
            spans.push(Span::raw(" ".repeat(pad)));
        }
        spans.push(Span::styled(cnt, Style::new().fg(Color::DarkGray)));
        let mut line = Line::from(spans);
        if i == ws.cursor {
            line = line.style(Style::new().bg(CURSOR_BG));
        }
        lines.push(line);
    }
    (title, lines, help)
}

fn clamp_offset(offset: usize, cursor: usize, visible: usize, len: usize) -> usize {
    if visible == 0 {
        return 0;
    }
    let mut o = offset.min(cursor);
    if cursor >= o + visible {
        o = cursor + 1 - visible;
    }
    if len <= visible {
        o = 0;
    } else {
        o = o.min(len - visible);
    }
    o
}

use ratatui::layout::Rect;