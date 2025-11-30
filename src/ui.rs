use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, DialogState, FocusedPanel, RenameMode};

// btop-inspired color scheme
const BORDER_COLOR: Color = Color::Cyan;
const BORDER_COLOR_FOCUSED: Color = Color::LightCyan;
const TITLE_COLOR: Color = Color::White;
const SELECTED_BG: Color = Color::Rgb(40, 44, 52);
const MARKER_COLOR: Color = Color::LightGreen;
const TEXT_COLOR: Color = Color::White;
const TEXT_DIM: Color = Color::DarkGray;
const INPUT_COLOR: Color = Color::Yellow;
const OLD_NAME_COLOR: Color = Color::Red;
const NEW_NAME_COLOR: Color = Color::LightGreen;
const ARROW_COLOR: Color = Color::DarkGray;
const DIR_COLOR: Color = Color::LightBlue;
const HELP_KEY_COLOR: Color = Color::Cyan;
const HELP_DESC_COLOR: Color = Color::DarkGray;
const DIALOG_BG: Color = Color::Rgb(30, 34, 42);
const SUCCESS_COLOR: Color = Color::LightGreen;
const ERROR_COLOR: Color = Color::LightRed;
const WARNING_COLOR: Color = Color::Yellow;
const MODE_COLOR: Color = Color::Magenta;

/// Main draw function
pub fn draw_ui(frame: &mut Frame, app: &App) {
    // Create main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(8),        // Files panel
            Constraint::Length(6),     // Operation panel
            Constraint::Min(5),        // Preview panel
            Constraint::Length(3),     // Help bar
        ])
        .split(frame.area());

    draw_files_panel(frame, app, chunks[0]);
    draw_operation_panel(frame, app, chunks[1]);
    draw_preview_panel(frame, app, chunks[2]);
    draw_help_bar(frame, app, chunks[3]);

    // Draw dialogs on top
    match app.dialog_state {
        DialogState::Confirm => draw_confirm_dialog(frame, app),
        DialogState::Help => draw_help_dialog(frame),
        DialogState::Success => draw_success_dialog(frame, app),
        DialogState::Error => draw_error_dialog(frame, app),
        DialogState::None => {}
    }
}

/// Draw the files panel
fn draw_files_panel(frame: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.focused_panel == FocusedPanel::Files;
    let border_color = if is_focused {
        BORDER_COLOR_FOCUSED
    } else {
        BORDER_COLOR
    };

    let sort_indicator = app.sort_order.short_indicator();
    let title = format!(" Dateien ({}) {} ", app.directory.display(), sort_indicator);
    let block = Block::default()
        .title(title)
        .title_style(Style::default().fg(TITLE_COLOR).bold())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .style(Style::default());

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    if app.files.is_empty() {
        let empty_msg = Paragraph::new("Keine Dateien gefunden")
            .style(Style::default().fg(TEXT_DIM));
        frame.render_widget(empty_msg, inner_area);
        return;
    }

    let items: Vec<ListItem> = app
        .files
        .iter()
        .enumerate()
        .map(|(i, file)| {
            let is_selected = app.selected_files.contains(&i);
            let is_current = i == app.selected_index;

            let marker = if is_selected { " * " } else { "   " };
            let marker_style = if is_selected {
                Style::default().fg(MARKER_COLOR).bold()
            } else {
                Style::default().fg(TEXT_DIM)
            };

            let name_style = if file.is_dir {
                Style::default().fg(DIR_COLOR)
            } else {
                Style::default().fg(TEXT_COLOR)
            };

            let suffix = if file.is_dir { "/" } else { "" };

            let line = Line::from(vec![
                Span::styled(marker, marker_style),
                Span::styled(format!("{}{}", file.name, suffix), name_style),
            ]);

            let mut item = ListItem::new(line);
            if is_current {
                item = item.style(
                    Style::default()
                        .bg(SELECTED_BG)
                        .add_modifier(Modifier::BOLD),
                );
            }
            item
        })
        .collect();

    // Calculate visible range for scrolling
    let visible_height = inner_area.height as usize;
    let total_items = items.len();
    let selected = app.selected_index;

    let start = if total_items <= visible_height {
        0
    } else if selected < visible_height / 2 {
        0
    } else if selected > total_items - visible_height / 2 {
        total_items.saturating_sub(visible_height)
    } else {
        selected.saturating_sub(visible_height / 2)
    };

    let visible_items: Vec<ListItem> = items.into_iter().skip(start).take(visible_height).collect();

    let list = List::new(visible_items);
    frame.render_widget(list, inner_area);
}

/// Draw the operation panel
fn draw_operation_panel(frame: &mut Frame, app: &App, area: Rect) {
    let is_search_focused = app.focused_panel == FocusedPanel::SearchField;
    let is_replace_focused = app.focused_panel == FocusedPanel::ReplaceField;
    let is_panel_focused = is_search_focused || is_replace_focused;

    let border_color = if is_panel_focused {
        BORDER_COLOR_FOCUSED
    } else {
        BORDER_COLOR
    };

    let block = Block::default()
        .title(" Operation ")
        .title_style(Style::default().fg(TITLE_COLOR).bold())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    // Split inner area for labels and input fields
    let inner_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Mode label
            Constraint::Length(1), // Search/Pattern or info
            Constraint::Length(1), // Replace or action toggle or empty
        ])
        .margin(1)
        .split(inner_area);

    // Mode label with current mode highlighted
    let mode_line = Line::from(vec![
        Span::styled("Modus: ", Style::default().fg(TEXT_DIM)),
        Span::styled(
            format!("[{}]", app.rename_mode.display_name()),
            Style::default().fg(MODE_COLOR).bold(),
        ),
        Span::styled("  (m: wechseln)", Style::default().fg(TEXT_DIM)),
    ]);
    frame.render_widget(Paragraph::new(mode_line), inner_chunks[0]);

    // Mode-specific content
    match app.rename_mode {
        RenameMode::SearchReplace => {
            draw_search_replace_fields(frame, app, is_search_focused, is_replace_focused, &inner_chunks, "Suche:", "Ersetze:");
        }
        RenameMode::Regex => {
            draw_search_replace_fields(frame, app, is_search_focused, is_replace_focused, &inner_chunks, "Regex:", "Ersetze:");
            // Show regex error if any
            if let Some(err) = &app.regex_error {
                let error_line = Line::from(Span::styled(
                    format!("Fehler: {}", err),
                    Style::default().fg(ERROR_COLOR),
                ));
                // This would need additional space, but for now we'll show in preview
                let _ = error_line;
            }
        }
        RenameMode::Numbering => {
            let label_style = if is_search_focused {
                Style::default().fg(INPUT_COLOR).bold()
            } else {
                Style::default().fg(TEXT_DIM)
            };

            let pattern_line = Line::from(vec![
                Span::styled("Muster:  ", label_style),
                Span::styled(&app.search_input, Style::default().fg(TEXT_COLOR)),
                if is_search_focused {
                    Span::styled("_", Style::default().fg(INPUT_COLOR).add_modifier(Modifier::SLOW_BLINK))
                } else {
                    Span::raw("")
                },
            ]);
            frame.render_widget(Paragraph::new(pattern_line), inner_chunks[1]);

            let hint_line = Line::from(Span::styled(
                "Nutze # fuer Ziffern: photo_### -> photo_001",
                Style::default().fg(TEXT_DIM).italic(),
            ));
            frame.render_widget(Paragraph::new(hint_line), inner_chunks[2]);
        }
        RenameMode::Prefix | RenameMode::Suffix => {
            let label = if app.rename_mode == RenameMode::Prefix { "Prefix:" } else { "Suffix:" };
            let label_style = if is_search_focused {
                Style::default().fg(INPUT_COLOR).bold()
            } else {
                Style::default().fg(TEXT_DIM)
            };

            let input_line = Line::from(vec![
                Span::styled(format!("{:9}", label), label_style),
                Span::styled(&app.search_input, Style::default().fg(TEXT_COLOR)),
                if is_search_focused {
                    Span::styled("_", Style::default().fg(INPUT_COLOR).add_modifier(Modifier::SLOW_BLINK))
                } else {
                    Span::raw("")
                },
            ]);
            frame.render_widget(Paragraph::new(input_line), inner_chunks[1]);

            let action_line = Line::from(vec![
                Span::styled("Aktion:  ", Style::default().fg(TEXT_DIM)),
                Span::styled(
                    format!("[{}]", app.prefix_action.display_name()),
                    Style::default().fg(INPUT_COLOR).bold(),
                ),
                Span::styled("  (t: wechseln)", Style::default().fg(TEXT_DIM)),
            ]);
            frame.render_widget(Paragraph::new(action_line), inner_chunks[2]);
        }
        RenameMode::Uppercase | RenameMode::Lowercase | RenameMode::TitleCase => {
            let info_text = match app.rename_mode {
                RenameMode::Uppercase => "Alle Dateinamen werden in GROSSBUCHSTABEN umgewandelt",
                RenameMode::Lowercase => "Alle Dateinamen werden in kleinbuchstaben umgewandelt",
                RenameMode::TitleCase => "Jedes Wort Beginnt Mit Grossbuchstaben",
                _ => "",
            };

            let info_line = Line::from(Span::styled(info_text, Style::default().fg(TEXT_DIM).italic()));
            frame.render_widget(Paragraph::new(info_line), inner_chunks[1]);

            let hint_line = Line::from(Span::styled(
                "Druecke Enter um die Vorschau anzuwenden",
                Style::default().fg(INPUT_COLOR),
            ));
            frame.render_widget(Paragraph::new(hint_line), inner_chunks[2]);
        }
    }
}

/// Helper to draw search/replace input fields
fn draw_search_replace_fields(
    frame: &mut Frame,
    app: &App,
    is_search_focused: bool,
    is_replace_focused: bool,
    chunks: &[Rect],
    search_label: &str,
    replace_label: &str,
) {
    let search_label_style = if is_search_focused {
        Style::default().fg(INPUT_COLOR).bold()
    } else {
        Style::default().fg(TEXT_DIM)
    };

    let search_line = Line::from(vec![
        Span::styled(format!("{:9}", search_label), search_label_style),
        Span::styled(&app.search_input, Style::default().fg(TEXT_COLOR)),
        if is_search_focused {
            Span::styled("_", Style::default().fg(INPUT_COLOR).add_modifier(Modifier::SLOW_BLINK))
        } else {
            Span::raw("")
        },
    ]);
    frame.render_widget(Paragraph::new(search_line), chunks[1]);

    let replace_label_style = if is_replace_focused {
        Style::default().fg(INPUT_COLOR).bold()
    } else {
        Style::default().fg(TEXT_DIM)
    };

    let replace_line = Line::from(vec![
        Span::styled(format!("{:9}", replace_label), replace_label_style),
        Span::styled(&app.replace_input, Style::default().fg(TEXT_COLOR)),
        if is_replace_focused {
            Span::styled("_", Style::default().fg(INPUT_COLOR).add_modifier(Modifier::SLOW_BLINK))
        } else {
            Span::raw("")
        },
    ]);
    frame.render_widget(Paragraph::new(replace_line), chunks[2]);
}

/// Draw the preview panel
fn draw_preview_panel(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Vorschau ")
        .title_style(Style::default().fg(TITLE_COLOR).bold())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(BORDER_COLOR));

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    // Show regex error if present
    if let Some(err) = &app.regex_error {
        let error_line = Paragraph::new(format!("Regex-Fehler: {}", err))
            .style(Style::default().fg(ERROR_COLOR));
        frame.render_widget(error_line, inner_area);
        return;
    }

    // For search/replace and regex mode, check if search is empty
    if app.rename_mode.uses_search_replace() && app.search_input.is_empty() {
        let hint = match app.rename_mode {
            RenameMode::Regex => "Gib ein Regex-Muster ein (z.B. IMG_(\\d+) -> photo_$1)",
            _ => "Gib einen Suchbegriff ein, um die Vorschau zu sehen",
        };
        let hint_para = Paragraph::new(hint).style(Style::default().fg(TEXT_DIM));
        frame.render_widget(hint_para, inner_area);
        return;
    }

    // For numbering/prefix/suffix, check if pattern is empty
    if matches!(app.rename_mode, RenameMode::Numbering | RenameMode::Prefix | RenameMode::Suffix) 
        && app.search_input.is_empty() 
    {
        let hint = match app.rename_mode {
            RenameMode::Numbering => "Gib ein Muster ein (z.B. photo_###)",
            RenameMode::Prefix => "Gib einen Prefix ein",
            RenameMode::Suffix => "Gib einen Suffix ein",
            _ => "",
        };
        let hint_para = Paragraph::new(hint).style(Style::default().fg(TEXT_DIM));
        frame.render_widget(hint_para, inner_area);
        return;
    }

    let changes: Vec<&_> = app.previews.iter().filter(|p| p.will_change).collect();

    if changes.is_empty() {
        let hint = Paragraph::new("Keine Aenderungen")
            .style(Style::default().fg(TEXT_DIM));
        frame.render_widget(hint, inner_area);
        return;
    }

    let items: Vec<ListItem> = changes
        .iter()
        .map(|preview| {
            let line = Line::from(vec![
                Span::styled(&preview.original_name, Style::default().fg(OLD_NAME_COLOR).add_modifier(Modifier::CROSSED_OUT)),
                Span::styled("  ->  ", Style::default().fg(ARROW_COLOR)),
                Span::styled(&preview.new_name, Style::default().fg(NEW_NAME_COLOR).bold()),
            ]);
            ListItem::new(line)
        })
        .collect();

    let list = List::new(items);
    frame.render_widget(list, inner_area);
}

/// Draw the help bar at the bottom
fn draw_help_bar(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Hilfe ")
        .title_style(Style::default().fg(TITLE_COLOR).bold())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(BORDER_COLOR));

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    let help_text = match app.focused_panel {
        FocusedPanel::Files => {
            let mut base = vec![
                ("j/k", "Nav"),
                ("Space", "Ausw"),
                ("a", "Alle"),
                ("m", "Modus"),
                ("s", "Sort"),
            ];
            // Add 't' hint for prefix/suffix modes
            if matches!(app.rename_mode, RenameMode::Prefix | RenameMode::Suffix) {
                base.push(("t", "Toggle"));
            }
            base.extend([
                ("Tab", "Feld"),
                ("Enter", "Run"),
                ("?", "Hilfe"),
                ("q", "Ende"),
            ]);
            base
        }
        FocusedPanel::SearchField | FocusedPanel::ReplaceField => {
            let mut base = vec![
                ("Tab", "Naechstes"),
                ("Enter", "Ausfuehren"),
                ("Esc", "Zurueck"),
            ];
            if matches!(app.rename_mode, RenameMode::Prefix | RenameMode::Suffix) {
                base.insert(1, ("t", "Toggle"));
            }
            base.push(("F1", "Hilfe"));
            base
        }
    };

    let spans: Vec<Span> = help_text
        .iter()
        .flat_map(|(key, desc)| {
            vec![
                Span::styled(format!(" {} ", key), Style::default().fg(HELP_KEY_COLOR).bold()),
                Span::styled(format!("{} ", desc), Style::default().fg(HELP_DESC_COLOR)),
            ]
        })
        .collect();

    let line = Line::from(spans);
    frame.render_widget(Paragraph::new(line), inner_area);
}

/// Draw the confirmation dialog
fn draw_confirm_dialog(frame: &mut Frame, app: &App) {
    let area = centered_rect(60, 40, frame.area());

    frame.render_widget(Clear, area);

    let change_count = app.previews.iter().filter(|p| p.will_change).count();

    let block = Block::default()
        .title(" Bestaetigung ")
        .title_style(Style::default().fg(WARNING_COLOR).bold())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(WARNING_COLOR))
        .style(Style::default().bg(DIALOG_BG));

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    let text = vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("{} Dateien werden umbenannt:", change_count),
            Style::default().fg(TEXT_COLOR).bold(),
        )),
        Line::from(""),
    ];

    // Show first few files to be renamed
    let mut lines = text;
    for preview in app.previews.iter().filter(|p| p.will_change).take(5) {
        lines.push(Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(&preview.original_name, Style::default().fg(OLD_NAME_COLOR)),
            Span::styled(" -> ", Style::default().fg(ARROW_COLOR)),
            Span::styled(&preview.new_name, Style::default().fg(NEW_NAME_COLOR)),
        ]));
    }

    if change_count > 5 {
        lines.push(Line::from(Span::styled(
            format!("  ... und {} weitere", change_count - 5),
            Style::default().fg(TEXT_DIM),
        )));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled(" [Enter/y] ", Style::default().fg(SUCCESS_COLOR).bold()),
        Span::styled("Bestaetigen  ", Style::default().fg(TEXT_DIM)),
        Span::styled(" [Esc/n] ", Style::default().fg(ERROR_COLOR).bold()),
        Span::styled("Abbrechen", Style::default().fg(TEXT_DIM)),
    ]));

    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
    frame.render_widget(paragraph, inner_area);
}

/// Draw the help dialog
fn draw_help_dialog(frame: &mut Frame) {
    let area = centered_rect(70, 90, frame.area());

    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" Tastenbelegung ")
        .title_style(Style::default().fg(TITLE_COLOR).bold())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(BORDER_COLOR_FOCUSED))
        .style(Style::default().bg(DIALOG_BG));

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    let help_sections = vec![
        ("", "--- Dateiliste ---"),
        ("j / Pfeil runter", "Naechste Datei"),
        ("k / Pfeil hoch", "Vorherige Datei"),
        ("Space", "Datei auswaehlen/abwaehlen"),
        ("a", "Alle Dateien auswaehlen/abwaehlen"),
        ("", ""),
        ("", "--- Modi & Sortierung ---"),
        ("m", "Modus wechseln"),
        ("s", "Sortierung wechseln"),
        ("t", "Aktion wechseln (Prefix/Suffix)"),
        ("", ""),
        ("", "--- Modi ---"),
        ("", "Suchen/Ersetzen, Regex, Nummerierung"),
        ("", "Prefix, Suffix, GROSS, klein, Titel"),
        ("", ""),
        ("", "--- Navigation ---"),
        ("Tab", "Naechstes Panel"),
        ("Shift+Tab", "Vorheriges Panel"),
        ("Esc", "Zurueck zur Dateiliste"),
        ("", ""),
        ("", "--- Aktionen ---"),
        ("Enter", "Umbenennung ausfuehren"),
        ("?", "Hilfe anzeigen"),
        ("q", "Programm beenden"),
    ];

    let lines: Vec<Line> = help_sections
        .iter()
        .map(|(key, desc)| {
            if key.is_empty() {
                Line::from(Span::styled(*desc, Style::default().fg(INPUT_COLOR).bold()))
            } else {
                Line::from(vec![
                    Span::styled(format!("{:20}", key), Style::default().fg(HELP_KEY_COLOR).bold()),
                    Span::styled(*desc, Style::default().fg(TEXT_COLOR)),
                ])
            }
        })
        .collect();

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner_area);
}

/// Draw success dialog
fn draw_success_dialog(frame: &mut Frame, app: &App) {
    let area = centered_rect(50, 20, frame.area());

    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" Erfolg ")
        .title_style(Style::default().fg(SUCCESS_COLOR).bold())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(SUCCESS_COLOR))
        .style(Style::default().bg(DIALOG_BG));

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    let msg = app
        .success_message
        .as_deref()
        .unwrap_or("Operation erfolgreich");

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(msg, Style::default().fg(SUCCESS_COLOR).bold())),
        Line::from(""),
        Line::from(Span::styled(
            "[Enter] Schliessen",
            Style::default().fg(TEXT_DIM),
        )),
    ];

    let paragraph = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(paragraph, inner_area);
}

/// Draw error dialog
fn draw_error_dialog(frame: &mut Frame, app: &App) {
    let area = centered_rect(60, 30, frame.area());

    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" Fehler ")
        .title_style(Style::default().fg(ERROR_COLOR).bold())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(ERROR_COLOR))
        .style(Style::default().bg(DIALOG_BG));

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    let msg = app.error_message.as_deref().unwrap_or("Unbekannter Fehler");

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(msg, Style::default().fg(ERROR_COLOR))),
        Line::from(""),
        Line::from(Span::styled(
            "[Enter] Schliessen",
            Style::default().fg(TEXT_DIM),
        )),
    ];

    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
    frame.render_widget(paragraph, inner_area);
}

/// Create a centered rectangle with given percentage of width and height
fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
