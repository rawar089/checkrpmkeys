use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table, Wrap},
    Frame,
};

use crate::app::{App, InputMode};
use crate::data::KeyStatus;

pub fn draw(f: &mut Frame, app: &mut App) {
    let area = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // title / summary
            Constraint::Min(5),    // table
            Constraint::Length(3), // filter / sort status
            Constraint::Length(1), // help line
        ])
        .split(area);

    draw_title(f, chunks[0], app);
    draw_table(f, chunks[1], app);
    draw_status_bar(f, chunks[2], app);
    draw_help(f, chunks[3], app);

    if app.show_help {
        draw_help_popup(f, area, app);
    } else if app.show_details {
        draw_details_popup(f, area, app);
    }
}

fn draw_title(f: &mut Frame, area: Rect, app: &App) {
    let (total, expired, invalid) = app.counts();
    let valid = total.saturating_sub(expired).saturating_sub(invalid);

    let line = Line::from(vec![
        Span::styled(
            app.i18n.app_title,
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::raw("   "),
        Span::styled(
            format!("{valid} {}", app.i18n.valid),
            Style::default().fg(Color::Green),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("{expired} {}", app.i18n.expired),
            Style::default().fg(Color::Red),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("{invalid} {}", app.i18n.invalid),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw(format!("   ({} {total})   [{}]", app.i18n.total, app.lang.code())),
    ]);

    let p = Paragraph::new(line)
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Left);
    f.render_widget(p, area);
}

fn draw_table(f: &mut Frame, area: Rect, app: &mut App) {
    let header = Row::new(
        [
            app.i18n.col_package,
            app.i18n.col_key_type,
            app.i18n.col_owner,
            app.i18n.col_expires,
            app.i18n.col_fingerprint,
            app.i18n.col_status,
        ]
        .map(|h| Cell::from(h).style(Style::default().add_modifier(Modifier::BOLD))),
    )
    .style(Style::default().bg(Color::DarkGray))
    .height(1)
    .bottom_margin(1);

    let rows: Vec<Row> = app
        .filtered_indices
        .iter()
        .map(|&idx| {
            let r = &app.all_records[idx];
            let (status_text, status_style) = match r.status() {
                KeyStatus::Expired => (
                    format!("\u{2717} {}", app.i18n.status_expired),
                    Style::default().fg(Color::Red),
                ),
                KeyStatus::Invalid => (
                    format!("\u{26A0} {}", app.i18n.status_invalid),
                    Style::default().fg(Color::Yellow),
                ),
                KeyStatus::Valid => (
                    format!("\u{2713} {}", app.i18n.status_valid),
                    Style::default().fg(Color::Green),
                ),
            };
            Row::new(vec![
                Cell::from(r.package_name.clone()),
                Cell::from(r.key_type.clone()),
                Cell::from(r.uid.clone()),
                Cell::from(r.expires.clone()),
                Cell::from(short_fingerprint(&r.fingerprint)),
                Cell::from(status_text).style(status_style),
            ])
        })
        .collect();

    let widths = [
        Constraint::Percentage(15),
        Constraint::Percentage(5),
        Constraint::Percentage(35),
        Constraint::Length(12),
        Constraint::Length(20),
        Constraint::Length(11),
    ];

    let title = format!(
        " {} ({}/{}) \u{2014} {} {} {} ",
        app.i18n.keys_title,
        app.filtered_indices.len(),
        app.all_records.len(),
        app.i18n.sorted_by,
        app.sort_column_label(),
        if app.sort_ascending { "\u{25B2}" } else { "\u{25BC}" }
    );

    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title(title))
        .row_highlight_style(
            Style::default()
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("\u{27A4} ");

    f.render_stateful_widget(table, area, &mut app.table_state);
}

fn draw_status_bar(f: &mut Frame, area: Rect, app: &App) {
    let content = match app.input_mode {
        InputMode::Filtering => Line::from(vec![
            Span::styled(app.i18n.filter_label, Style::default().fg(Color::Cyan)),
            Span::raw(app.filter_text.as_str()),
            Span::styled("\u{2588}", Style::default().fg(Color::Cyan)),
        ]),
        InputMode::Normal => {
            let filter_note = if app.filter_text.is_empty() {
                String::new()
            } else {
                format!(
                    "   {}: \"{}\" ({})",
                    app.i18n.active_filter, app.filter_text, app.i18n.clear_filter_hint
                )
            };
            Line::from(format!(
                "{}: {} {}{filter_note}",
                app.i18n.sorted_by,
                app.sort_column_label(),
                if app.sort_ascending { "\u{25B2}" } else { "\u{25BC}" }
            ))
        }
    };

    let p = Paragraph::new(content).block(Block::default().borders(Borders::ALL));
    f.render_widget(p, area);
}

fn draw_help(f: &mut Frame, area: Rect, app: &App) {
    let text = match app.input_mode {
        InputMode::Filtering => app.i18n.help_filtering,
        InputMode::Normal => app.i18n.help_normal,
    };
    let p = Paragraph::new(text).style(Style::default().fg(Color::DarkGray));
    f.render_widget(p, area);
}

fn draw_details_popup(f: &mut Frame, area: Rect, app: &App) {
    let Some(record) = app.selected_record() else {
        return;
    };

    let popup_area = centered_rect(70, 55, area);
    f.render_widget(Clear, popup_area);

    let status_span = match record.status() {
        KeyStatus::Expired => Span::styled(
            app.i18n.status_expired,
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ),
        KeyStatus::Invalid => Span::styled(
            app.i18n.status_invalid,
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        KeyStatus::Valid => Span::styled(
            app.i18n.status_valid,
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
    };

    // Label widths vary across languages (e.g. "Schl\u{00FC}sseltyp:" vs "Key Type:"),
    // so pad to the longest label in the active language rather than a
    // fixed English-sized column.
    let labels = [
        app.i18n.detail_package,
        app.i18n.detail_key_type,
        app.i18n.detail_owner,
        app.i18n.detail_expires,
        app.i18n.detail_status,
    ];
    let label_width = labels.iter().map(|l| l.chars().count()).max().unwrap_or(0) + 1;
    let pad = |label: &str| format!("{label:<label_width$}");

    let label_style = Style::default().add_modifier(Modifier::BOLD);
    let lines = vec![
        Line::from(vec![
            Span::styled(pad(app.i18n.detail_package), label_style),
            Span::raw(record.package_name.clone()),
        ]),
        Line::from(vec![
            Span::styled(pad(app.i18n.detail_key_type), label_style),
            Span::raw(record.key_type.clone()),
        ]),
        Line::from(vec![
            Span::styled(pad(app.i18n.detail_owner), label_style),
            Span::raw(record.uid.clone()),
        ]),
        Line::from(vec![
            Span::styled(pad(app.i18n.detail_expires), label_style),
            Span::raw(record.expires.clone()),
        ]),
        Line::from(vec![Span::styled(pad(app.i18n.detail_status), label_style), status_span]),
        Line::from(""),
        Line::from(Span::styled(app.i18n.detail_fingerprint, label_style)),
        Line::from(record.fingerprint.clone()),
    ];

    let title = format!(" {} ", app.i18n.details_title);
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let p = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });
    f.render_widget(p, popup_area);
}

/// Renders the keybindings help overlay, toggled with `h`.
fn draw_help_popup(f: &mut Frame, area: Rect, app: &App) {
    let popup_area = centered_rect(64, 70, area);
    f.render_widget(Clear, popup_area);

    let t = &app.i18n;
    let bindings: [(&str, &str); 11] = [
        ("j / k, \u{2191} / \u{2193}", t.kb_move),
        ("g / G", t.kb_top_bottom),
        ("PageUp / PageDown", t.kb_page),
        ("/", t.kb_filter),
        ("s", t.kb_sort_column),
        ("r", t.kb_reverse_sort),
        ("Enter", t.kb_details),
        ("c", t.kb_clear_filter),
        ("l", t.kb_language),
        ("h", t.kb_help),
        ("Esc", t.kb_close),
    ];

    let key_width = bindings.iter().map(|(k, _)| k.chars().count()).max().unwrap_or(0) + 2;
    let key_style = Style::default()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD);

    let mut lines: Vec<Line> = bindings
        .iter()
        .map(|(key, desc)| {
            Line::from(vec![
                Span::styled(format!("{key:<key_width$}"), key_style),
                Span::raw(*desc),
            ])
        })
        .collect();

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled(format!("{:<key_width$}", "q"), key_style),
        Span::raw(t.kb_quit),
    ]));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        t.filter_mode_note,
        Style::default()
            .fg(Color::DarkGray)
            .add_modifier(Modifier::ITALIC),
    )));

    let title = format!(" {} ", t.help_title);
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let p = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });
    f.render_widget(p, popup_area);
}

/// Centers a rectangle of `percent_x` x `percent_y` size within `r`.
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let vertical = Layout::default()
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
        .split(vertical[1])[1]
}

/// Truncates a long fingerprint for the table column; full value is shown
/// in the details popup.
fn short_fingerprint(fp: &str) -> String {
    if fp.len() <= 20 {
        fp.to_string()
    } else {
        format!("...{}", &fp[fp.len() - 16..])
    }
}
