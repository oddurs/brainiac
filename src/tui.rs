//! Terminal browser: live ranked search, preview, and marking chunks to build a
//! pack from. Ctrl-chords for actions so every printable key stays typing.

use crate::{
    pack,
    rank::{self, Graph, Hit},
    store,
};
use anyhow::Result;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Clear, List, ListItem, ListState, Paragraph, Wrap};
use rusqlite::Connection;
use std::collections::BTreeSet;

pub enum Outcome {
    Quit,
    /// Emit a pack built from the marked chunks (or the query, if none marked).
    Pack {
        query: String,
        marked: Vec<i64>,
    },
}

struct App {
    query: String,
    hits: Vec<Hit>,
    state: ListState,
    marked: BTreeSet<i64>,
    preview: String,
    preview_scroll: u16,
    status: String,
    help: bool,
}

pub fn run(conn: &Connection, initial: &str) -> Result<Outcome> {
    let graph = rank::build_graph(conn)?;
    let mut app = App {
        query: initial.to_string(),
        hits: Vec::new(),
        state: ListState::default(),
        marked: BTreeSet::new(),
        preview: String::new(),
        preview_scroll: 0,
        status: String::new(),
        help: false,
    };
    refresh(conn, &graph, &mut app)?;

    let mut term = ratatui::init();
    let outcome = loop {
        term.draw(|f| draw(f, &mut app))?;
        let Event::Key(k) = event::read()? else {
            continue;
        };
        if k.kind != KeyEventKind::Press {
            continue;
        }
        match on_key(
            &mut app,
            k.modifiers.contains(KeyModifiers::CONTROL),
            k.code,
        ) {
            Action::None => {}
            Action::Preview => load_preview(conn, &mut app)?,
            Action::Research => refresh(conn, &graph, &mut app)?,
            Action::Quit => break Outcome::Quit,
            Action::Emit => {
                break Outcome::Pack {
                    query: app.query.clone(),
                    marked: app.marked.iter().copied().collect(),
                };
            }
        }
    };
    ratatui::restore();
    Ok(outcome)
}

/// What the event loop must do after a keypress. Keeping the mapping pure means
/// the keymap is testable without a terminal.
#[derive(Debug, PartialEq, Eq)]
enum Action {
    None,
    Preview,
    Research,
    Quit,
    Emit,
}

fn on_key(app: &mut App, ctrl: bool, code: KeyCode) -> Action {
    match (ctrl, code) {
        (true, KeyCode::Char('c')) | (false, KeyCode::Esc) => Action::Quit,
        (true, KeyCode::Char('o')) | (false, KeyCode::Enter) => Action::Emit,
        (true, KeyCode::Char('h')) => {
            app.help = !app.help;
            Action::None
        }
        (true, KeyCode::Char('n')) | (false, KeyCode::Down) => {
            move_by(app, 1);
            Action::Preview
        }
        (true, KeyCode::Char('p')) | (false, KeyCode::Up) => {
            move_by(app, -1);
            Action::Preview
        }
        (_, KeyCode::PageDown) => {
            app.preview_scroll = app.preview_scroll.saturating_add(10);
            Action::None
        }
        (_, KeyCode::PageUp) => {
            app.preview_scroll = app.preview_scroll.saturating_sub(10);
            Action::None
        }
        (false, KeyCode::Tab) => {
            if let Some(h) = current(app) {
                let id = h.chunk_id;
                if !app.marked.remove(&id) {
                    app.marked.insert(id);
                }
                app.status = format!("{} marked", app.marked.len());
            }
            move_by(app, 1);
            Action::Preview
        }
        (true, KeyCode::Char('u')) => {
            app.query.clear();
            Action::Research
        }
        (false, KeyCode::Backspace) => {
            app.query.pop();
            Action::Research
        }
        (false, KeyCode::Char(c)) => {
            app.query.push(c);
            Action::Research
        }
        _ => Action::None,
    }
}

fn current(app: &App) -> Option<&Hit> {
    app.state.selected().and_then(|i| app.hits.get(i))
}

fn move_by(app: &mut App, d: isize) {
    if app.hits.is_empty() {
        return;
    }
    let i = app.state.selected().unwrap_or(0) as isize + d;
    let i = i.clamp(0, app.hits.len() as isize - 1) as usize;
    app.state.select(Some(i));
    app.preview_scroll = 0;
}

fn refresh(conn: &Connection, graph: &Graph, app: &mut App) -> Result<()> {
    let t = std::time::Instant::now();
    app.hits = if app.query.trim().len() < 2 {
        Vec::new()
    } else {
        rank::search_with_graph(conn, graph, &app.query, 200)?
    };
    app.state
        .select(if app.hits.is_empty() { None } else { Some(0) });
    app.preview_scroll = 0;
    app.status = format!("{} hits · {}ms", app.hits.len(), t.elapsed().as_millis());
    load_preview(conn, app)
}

fn load_preview(conn: &Connection, app: &mut App) -> Result<()> {
    app.preview = match current(app) {
        Some(h) => store::chunk_by_id(conn, h.chunk_id)?
            .map(|c| {
                c.body
                    .lines()
                    .enumerate()
                    .map(|(i, l)| format!("{:>5} │ {l}", c.start_line as usize + i))
                    .collect::<Vec<_>>()
                    .join("\n")
            })
            .unwrap_or_default(),
        None => String::new(),
    };
    Ok(())
}

fn draw(f: &mut Frame, app: &mut App) {
    let v = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(3),
        Constraint::Length(1),
    ])
    .split(f.area());

    let input = Paragraph::new(Line::from(vec![
        Span::styled("› ", Style::new().fg(Color::Cyan).bold()),
        Span::raw(app.query.as_str()),
        Span::styled("▌", Style::new().fg(Color::Cyan)),
    ]))
    .block(
        Block::bordered()
            .title(" brainiac ")
            .title_style(Style::new().bold()),
    );
    f.render_widget(input, v[0]);

    let h =
        Layout::horizontal([Constraint::Percentage(45), Constraint::Percentage(55)]).split(v[1]);

    let items: Vec<ListItem> = app
        .hits
        .iter()
        .map(|hit| {
            let mark = if app.marked.contains(&hit.chunk_id) {
                "●"
            } else {
                " "
            };
            ListItem::new(Line::from(vec![
                Span::styled(mark, Style::new().fg(Color::Green)),
                Span::styled(
                    format!(" {:<3}", hit.signals),
                    Style::new().fg(Color::DarkGray),
                ),
                Span::styled(hit.name.clone(), Style::new().fg(Color::Yellow)),
                Span::styled(
                    format!("  {}:{}", hit.path, hit.start_line),
                    Style::new().fg(Color::Gray),
                ),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(Block::bordered().title(" results "))
        .highlight_style(Style::new().bg(Color::Indexed(236)).bold())
        .highlight_symbol("");
    f.render_stateful_widget(list, h[0], &mut app.state);

    let title = current(app)
        .map(|hit| format!(" {}:{}-{} ", hit.path, hit.start_line, hit.end_line))
        .unwrap_or_else(|| " preview ".into());
    let prev = Paragraph::new(app.preview.as_str())
        .block(Block::bordered().title(title))
        .scroll((app.preview_scroll, 0));
    f.render_widget(prev, h[1]);

    let bar = Line::from(vec![
        Span::styled(
            format!(" {} ", app.status),
            Style::new().fg(Color::DarkGray),
        ),
        Span::styled(
            format!("· {} marked ", app.marked.len()),
            Style::new().fg(Color::Green),
        ),
        Span::styled(
            "· tab mark · enter pack · ^h help · esc quit",
            Style::new().fg(Color::DarkGray),
        ),
    ]);
    f.render_widget(Paragraph::new(bar), v[2]);

    if app.help {
        let area = centered(f.area(), 56, 12);
        f.render_widget(Clear, area);
        f.render_widget(
            Paragraph::new(HELP)
                .block(Block::bordered().title(" keys "))
                .wrap(Wrap { trim: false }),
            area,
        );
    }
}

const HELP: &str = "\
 type            live ranked search
 ↑ ↓ / ^p ^n     move selection
 tab             mark / unmark the hit
 pgup pgdn       scroll preview
 enter / ^o      emit a context pack and exit
 ^u              clear the query
 ^h              toggle this help
 esc / ^c        quit

 signals: L lexical  S symbol  G graph";

fn centered(area: Rect, w: u16, h: u16) -> Rect {
    let w = w.min(area.width);
    let h = h.min(area.height);
    Rect {
        x: area.x + (area.width - w) / 2,
        y: area.y + (area.height - h) / 2,
        width: w,
        height: h,
    }
}

/// Render a pack from explicitly marked chunks, bypassing ranking.
pub fn pack_marked(conn: &Connection, marked: &[i64]) -> Result<String> {
    let mut out = String::from("# Context pack — selected\n\n");
    for id in marked {
        let Some(c) = store::chunk_by_id(conn, *id)? else {
            continue;
        };
        let Some(f) = store::file_by_id(conn, c.file_id)? else {
            continue;
        };
        out.push_str(&format!(
            "### {}:{}-{} — {}\n```{}\n{}\n```\n\n",
            f.path,
            c.start_line,
            c.end_line,
            c.name,
            crate::lang::Lang::from_name(&f.lang).fence(),
            c.body
        ));
    }
    out.push_str(&format!("<!-- ~{} tokens -->\n", pack::est_tokens(&out)));
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};

    fn app_with(hits: Vec<Hit>) -> App {
        let mut state = ListState::default();
        state.select(hits.first().map(|_| 0));
        App {
            query: "config".into(),
            hits,
            state,
            marked: BTreeSet::new(),
            preview: "    1 │ fn load_config() {}".into(),
            preview_scroll: 0,
            status: "2 hits · 3ms".into(),
            help: false,
        }
    }

    fn hit(name: &str, path: &str) -> Hit {
        Hit {
            bhash: name.into(),
            chunk_id: 1,
            file_id: 1,
            path: path.into(),
            name: name.into(),
            kind: "function".into(),
            start_line: 10,
            end_line: 20,
            score: 0.5,
            signals: "LSG".into(),
        }
    }

    fn rendered(app: &mut App) -> String {
        let mut term = Terminal::new(TestBackend::new(100, 24)).unwrap();
        term.draw(|f| draw(f, app)).unwrap();
        term.backend()
            .buffer()
            .content()
            .iter()
            .map(|c| c.symbol())
            .collect::<String>()
    }

    #[test]
    fn draws_query_results_and_preview() {
        let mut app = app_with(vec![hit("load_config", "src/lib.rs")]);
        let out = rendered(&mut app);
        assert!(out.contains("brainiac"), "{out}");
        assert!(out.contains("config"));
        assert!(out.contains("load_config"));
        assert!(out.contains("src/lib.rs"));
        assert!(out.contains("2 hits"));
    }

    #[test]
    fn marked_rows_are_flagged_and_counted() {
        let mut app = app_with(vec![hit("a", "src/a.rs"), hit("b", "src/b.rs")]);
        app.marked.insert(1);
        let out = rendered(&mut app);
        assert!(out.contains('●'), "no mark glyph: {out}");
        assert!(out.contains("1 marked"));
    }

    #[test]
    fn help_overlay_renders_on_top() {
        let mut app = app_with(vec![hit("a", "src/a.rs")]);
        app.help = true;
        let out = rendered(&mut app);
        assert!(out.contains("live ranked search"), "{out}");
    }

    #[test]
    fn an_empty_result_set_does_not_panic() {
        let mut app = app_with(Vec::new());
        let out = rendered(&mut app);
        assert!(out.contains("preview"), "{out}");
    }

    #[test]
    fn typing_edits_the_query_and_asks_for_a_new_search() {
        let mut app = app_with(vec![hit("a", "src/a.rs")]);
        app.query.clear();
        for c in "rank".chars() {
            assert_eq!(on_key(&mut app, false, KeyCode::Char(c)), Action::Research);
        }
        assert_eq!(app.query, "rank");
        assert_eq!(
            on_key(&mut app, false, KeyCode::Backspace),
            Action::Research
        );
        assert_eq!(app.query, "ran");
        assert_eq!(on_key(&mut app, true, KeyCode::Char('u')), Action::Research);
        assert!(app.query.is_empty());
    }

    #[test]
    fn tab_marks_the_hit_and_advances() {
        let mut app = app_with(vec![hit("a", "src/a.rs"), hit("b", "src/b.rs")]);
        assert_eq!(on_key(&mut app, false, KeyCode::Tab), Action::Preview);
        assert!(app.marked.contains(&1));
        assert_eq!(app.state.selected(), Some(1));
        // Marking the same chunk again unmarks it.
        app.state.select(Some(0));
        on_key(&mut app, false, KeyCode::Tab);
        assert!(app.marked.is_empty());
    }

    #[test]
    fn quit_and_emit_have_two_bindings_each() {
        let mut app = app_with(vec![hit("a", "src/a.rs")]);
        assert_eq!(on_key(&mut app, false, KeyCode::Esc), Action::Quit);
        assert_eq!(on_key(&mut app, true, KeyCode::Char('c')), Action::Quit);
        assert_eq!(on_key(&mut app, false, KeyCode::Enter), Action::Emit);
        assert_eq!(on_key(&mut app, true, KeyCode::Char('o')), Action::Emit);
    }

    #[test]
    fn ctrl_h_toggles_help_without_touching_the_query() {
        let mut app = app_with(vec![hit("a", "src/a.rs")]);
        let before = app.query.clone();
        assert_eq!(on_key(&mut app, true, KeyCode::Char('h')), Action::None);
        assert!(app.help);
        on_key(&mut app, true, KeyCode::Char('h'));
        assert!(!app.help);
        assert_eq!(app.query, before);
    }

    #[test]
    fn selection_stays_in_bounds() {
        let mut app = app_with(vec![hit("a", "src/a.rs"), hit("b", "src/b.rs")]);
        for _ in 0..10 {
            move_by(&mut app, 1);
        }
        assert_eq!(app.state.selected(), Some(1));
        for _ in 0..10 {
            move_by(&mut app, -1);
        }
        assert_eq!(app.state.selected(), Some(0));
    }
}
