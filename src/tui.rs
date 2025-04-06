use crate::libs::args::Args;
use crate::libs::sqlite::{Log, ScanResults};
use crossterm::event;
use crossterm::event::{Event, KeyCode};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Rect};
use ratatui::prelude::{Line, Stylize, Widget};
use ratatui::style::{Color, Style};
use ratatui::symbols::border::THICK;
use ratatui::widgets::{Block, BorderType, Cell, Gauge, HighlightSpacing, Row, Table};
use ratatui::DefaultTerminal;
use sqlx::SqlitePool;
use std::io;
use std::process::exit;
use std::sync::{Arc, RwLock};
use tokio::time::Duration;
use crate::libs;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Tab {
    Home,
    Logs,
}

#[derive(Debug, Clone)]
pub struct Tui {
    pub halt: bool,
    pub pause: Arc<core::sync::atomic::AtomicBool>,
    pub scroll_offset: usize,
    pub refresh_rate: f64,
    pub sqlite_pool: SqlitePool,
    pub scan_id: String,
    pub results: Arc<RwLock<ScanResults>>,
    pub method_filter: String, // none, active, passive
    pub logs: Arc<RwLock<Vec<Log>>>,
    pub log_level: String, // debug info warn error
    pub status: String,
    pub current_tab: Tab,
    pub args: Args,
    pub output_written: bool,
}

impl Tui {
    pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.halt {
            self.status = if self.results.read().unwrap().found.len() as i32 + self.results.read().unwrap().not_found >= self.results.read().unwrap().total  {
                if !self.args.output_path.is_empty() && self.output_written == false {
                    let write_output = crate::export_results::export(
                        self.scan_id.clone(),
                        self.sqlite_pool.clone(),
                        self.args.output_path.clone(),
                        self.args.output_format.clone().to_string(),
                    ).await;

                    match write_output {
                        Ok(_) => {
                            let _ = libs::sqlite::insert_log(
                                self.scan_id.clone(),
                                "info".to_string(),
                                format!("Output written to {}", self.args.output_path),
                                &self.sqlite_pool,
                                self.args.log_level.to_string(),
                            ).await;
                            self.output_written = true;
                        }
                        Err(e) => {
                            let _ = libs::sqlite::insert_log(
                                self.scan_id.clone(),
                                "error".to_string(),
                                format!("Error writing output: {}", e),
                                &self.sqlite_pool,
                                self.args.log_level.to_string(),
                            ).await;
                            self.output_written = true;
                        }
                    }
                }
                "Completed".to_string()
            } else if self.pause.load(core::sync::atomic::Ordering::SeqCst) {
                "Paused".to_string()
            } else {
                "Running".to_string()
            };

            terminal.draw(|frame| self.render(frame.area(), frame.buffer_mut()))?;

            self.handle_events().await?;
        }
        ratatui::restore();
        // display banner unless disabled
        if !self.args.no_banner {
            libs::banner::full();
        }
        exit(0);
    }

    async fn handle_events(&mut self) -> io::Result<()> {
        let timeout = (1000.0 / self.refresh_rate) as u64;
        if event::poll(Duration::from_millis(timeout))? {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Char('q') | KeyCode::Char('Q') => self.halt = true,
                    KeyCode::Char('p') | KeyCode::Char('P') => {
                        self.pause.store(
                            !self.pause.load(core::sync::atomic::Ordering::SeqCst),
                            core::sync::atomic::Ordering::SeqCst,
                        );
                    }
                    KeyCode::Char('h') | KeyCode::Char('H') => self.current_tab = Tab::Home,
                    KeyCode::Char('l') | KeyCode::Char('L') => self.current_tab = Tab::Logs,
                    KeyCode::Up => {
                        if self.scroll_offset > 0 {
                            self.scroll_offset -= 1;
                        }
                    }
                    KeyCode::Down => self.scroll_offset += 1,
                    // left and right should change log level debug, info, warn, error
                    KeyCode::Left => {
                        match self.current_tab {
                            Tab::Home => {
                                self.method_filter = match self.method_filter.as_str() {
                                    "none" => "active".to_string(),
                                    "active" => "passive".to_string(),
                                    "passive" => "none".to_string(),
                                    _ => "none".to_string(),
                                };
                            }
                            Tab::Logs => {
                                self.log_level = match self.log_level.as_str() {
                                    "debug" => "info".to_string(),
                                    "info" => "warn".to_string(),
                                    "warn" => "error".to_string(),
                                    _ => "debug".to_string(),
                                };
                            }
                        }

                    }
                    KeyCode::Right => {
                        match self.current_tab {
                            Tab::Home => {
                                self.method_filter = match self.method_filter.as_str() {
                                    "none" => "passive".to_string(),
                                    "active" => "none".to_string(),
                                    "passive" => "active".to_string(),
                                    _ => "none".to_string(),
                                };
                            }
                            Tab::Logs => {
                                self.log_level = match self.log_level.as_str() {
                                    "debug" => "warn".to_string(),
                                    "info" => "debug".to_string(),
                                    "warn" => "error".to_string(),
                                    _ => "info".to_string(),
                                };
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }
}

impl Widget for &Tui {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from("[ V O Y A G E ]".bold());
        let status = Line::from(format!(" Status: {} ", self.status).bold());
        let tab = Line::from(match self.current_tab {
            Tab::Home => " Home ",
            Tab::Logs => " Logs ",
        });
        let instructions = Line::from(" <Q> Quit | <P> Toggle Pause | <H/L> Home/Logs ".bold());
        let version = Line::from(format!(" v{} ", env!("CARGO_PKG_VERSION")).bold());
        let block = Block::bordered()
            .title(title.centered())
            .title_top(status.right_aligned())
            .title_top(tab.left_aligned())
            .title_bottom(instructions.left_aligned())
            .title_bottom(version.right_aligned())
            .border_set(THICK)
            .border_type(BorderType::Rounded);
        block.render(area, buf);

        let total = self.results.read().unwrap().total;
        let found_len = self.results.read().unwrap().found.len() as i32;
        let not_found = self.results.read().unwrap().not_found;

        let progress_percentage = if total > 0 {
            ((found_len + not_found) as f64 / total as f64 * 100.0).round() as u32
        } else {
            0
        };

        let progress_text = format!(
            "Progress: {}% | Found: {} | Scanned: {} | Total: {}",
            progress_percentage, found_len, found_len + not_found, total
        );
        let progress_area = Rect::new(1, 1, area.width - 2, 1);
        let progress_ratio = match (found_len + not_found) as f64 / total as f64 {
            ratio if ratio > 1.0 => 1.0,
            ratio if ratio < 0.0 => 0.0,
            ratio => ratio,
        };
        Gauge::default()
            .gauge_style(Style::default().fg(Color::Indexed(2)).bg(Color::Indexed(0)))
            .ratio(progress_ratio)
            .label(progress_text)
            .render(progress_area, buf);

        match self.current_tab {
            Tab::Home => self.render_home(area, buf),
            Tab::Logs => self.render_logs(area, buf),
        }
    }
}

impl Tui {
    fn render_home(&self, area: Rect, buf: &mut Buffer) {
        let visible_items = area.height - 4;

        let found_items = self.results.read().unwrap().clone();

        let mut displayed_list = vec![];
        for (index, result) in found_items.found.iter().enumerate() {
            if index >= self.scroll_offset && index < self.scroll_offset + visible_items as usize {
                let found_style = Style::default().fg(Color::Green);

                let index_cell = Cell::from(format!("{}", index + 1));
                let domain_cell = Cell::from(format!("{}.{}", result.subdomain, result.domain.clone()));
                let status_cell = Cell::from("Found");
                let method_cell = Cell::from(result.method.clone());
                let source_cell = Cell::from(result.source.clone());

                if self.method_filter == "none" || self.method_filter == result.method {
                    let row = Row::new(vec![
                        index_cell,
                        domain_cell,
                        status_cell.style(found_style),
                        method_cell,
                        source_cell,
                    ]);
                    displayed_list.push(row);
                }
            }
        }

        let header_style = Style::default().fg(Color::Indexed(1));

        let header = ["No.", "Domain", "Status", "Method", "Source/Technique"]
            .into_iter()
            .map(Cell::from)
            .collect::<Row>()
            .style(header_style)
            .height(1);

        // Create a separator row with dashes spanning all columns
        let separator = Row::new(vec![
            "-".repeat(area.width as usize), // Adjust width for "No."
            "-".repeat(area.width as usize), // Adjust width for "Domain"
            "-".repeat(area.width as usize), // Adjust width for "Status"
            "-".repeat(area.width as usize), // Adjust width for "Method"
            "-".repeat(area.width as usize), // Adjust width for "Source"
        ])
        .style(Style::default().fg(Color::White));

        let instructions = Line::from(" <Up/Down> Navigate | <Left/Right> Cycle Filter ".bold());
        let cell_sizes = [
            Constraint::Length(5),
            Constraint::Fill(2),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(18),
        ];

        let table = Table::new(
            // Insert separator row after header
            std::iter::once(header.clone())
                .chain(std::iter::once(separator))
                .chain(displayed_list),
            cell_sizes,
        )
        .block(
            Block::default()
                .title(format!(" Filter Results: {} ", self.method_filter).bold())
                .title_bottom(instructions.left_aligned())
                .borders(ratatui::widgets::Borders::all())
                .border_type(BorderType::Rounded),
        )
        .widths(cell_sizes)
        .highlight_spacing(HighlightSpacing::Always);

        let table_area = Rect::new(area.x + 1, area.y + 3, area.width - 2, area.height - 4);
        table.render(table_area, buf);
    }

    fn render_logs(&self, area: Rect, buf: &mut Buffer) {
        let visible_items = area.height as usize - 4;
        let mut displayed_list = vec![];

        let log_levels = vec!["debug", "info", "warn", "error"];

        for (index, log) in self.logs.read().unwrap().iter().enumerate() {
            // Filter logs based on the log level filter
            if index < self.scroll_offset || index >= self.scroll_offset + visible_items {
                continue;
            }

            let min_log_level_index = log_levels
                .iter()
                .position(|&level| level == self.log_level)
                .unwrap_or(0);

            let log_level_index = log_levels
                .iter()
                .position(|&level| level == log.level)
                .unwrap_or(0);

            if log_level_index < min_log_level_index {
                continue;
            }

            let max_desc_width = (area.width as usize * 3) / 5;
            let wrapped_description = textwrap::wrap(&log.description, max_desc_width);

            let log_level_color = match log_level_index {
                0 => Color::White,
                1 => Color::Green,
                2 => Color::Yellow,
                3 => Color::Red,
                _ => Color::White,
            };

            let log_level_style = Style::default().fg(log_level_color);
            let log_level_cell = Cell::from(log.level.clone()).style(log_level_style);

            let log_created_at_cell = Cell::from(log.created_at.clone());
            let description_cell = Cell::from(wrapped_description.join("\n")).style(Style::default());

            let first_row_cells = vec![
                Cell::from(format!("{}", index + 1)),
                Cell::from(description_cell), // Multi-line cell instead of extra rows
                Cell::from(log_level_cell),
                Cell::from(log_created_at_cell),
            ];

            let row_height = wrapped_description.len() as u16; // Adjust height to fit all lines
            displayed_list.push(Row::new(first_row_cells).height(row_height));
        }

        let header_row = Row::new(vec!["No.", "Description", "Level", "Logged On"])
            .style(Style::default().fg(Color::Indexed(1)))
            .height(1);
        // Create a separator row with dashes spanning all columns
        let separator = Row::new(vec![
            "-".repeat(area.width as usize), // Adjust width for "No."
            "-".repeat(area.width as usize), // Adjust width for "Description"
            "-".repeat(area.width as usize), // Adjust width for "Level"
            "-".repeat(area.width as usize), // Adjust width for "Logged On"
        ])
        .style(Style::default().fg(Color::White));

        let instructions = Line::from(" <Up/Down> Navigate | <Left/Right> Cycle Filter ".bold());
        let logs_len = Line::from(self.logs.read().unwrap().len().to_string());

        let table = Table::new(
            std::iter::once(header_row)
                .chain(std::iter::once(separator))
                .chain(displayed_list),
            [
                Constraint::Length(10),
                Constraint::Fill(1),
                Constraint::Length(15),
                Constraint::Length(20),
            ],
        )
        .block(
            Block::default()
                .title(format!(" Log Level: {} ", self.log_level).bold())
                .title_top(logs_len.right_aligned())
                .title_bottom(instructions.left_aligned())
                .borders(ratatui::widgets::Borders::all())
                .border_type(BorderType::Rounded),
        )
        .highlight_spacing(HighlightSpacing::Always);

        let table_area = Rect::new(area.x + 1, area.y + 3, area.width - 2, area.height - 4);
        table.render(table_area, buf);
    }
}
