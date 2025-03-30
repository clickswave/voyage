use crate::libs;
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
use std::sync::{Arc, Mutex};
use tokio::time::Duration;
use crate::libs::args::Args;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Tab {
    Home,
    Logs,
}

#[derive(Debug)]
pub struct Tui {
    pub halt: bool,
    pub pause: Arc<core::sync::atomic::AtomicBool>,
    pub scroll_offset: usize,
    pub refresh_rate: f64,
    pub sqlite_pool: SqlitePool,
    pub scan_id: String,
    pub results: ScanResults,
    pub logs: Vec<Log>,
    pub log_level: Arc<Mutex<String>>,
    pub status: String,
    pub current_tab: Tab,
    pub output_written: bool,
    pub args: Args,
}

impl Tui {
    pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.halt {
            let get_progress =
                libs::sqlite::get_results(self.scan_id.clone(), self.sqlite_pool.clone()).await;
            if let Ok(results) = get_progress {
                self.results = results;

                self.status = if self.results.found.len() as i32 + self.results.not_found
                    >= self.results.total
                {
                    if !self.output_written {
                        let _ = crate::export_results::export(
                            self.scan_id.clone(),
                            self.sqlite_pool.clone(),
                            self.args.output_path.clone(),
                            self.args.output_format.to_string().clone(),
                        ).await;
                        self.output_written = true;
                    }

                    "Completed".to_string()
                } else if self.pause.load(core::sync::atomic::Ordering::SeqCst) {
                    "Paused".to_string()
                } else {
                    "Running".to_string()
                };
            }
            let get_logs = libs::sqlite::get_logs(
                self.scan_id.clone(),
                self.log_level.lock().unwrap().clone(),
                self.sqlite_pool.clone(),
            ).await;

            if let Ok(logs) = get_logs {
                self.logs = logs;
            }

            terminal.draw(|frame| self.render(frame.area(), frame.buffer_mut()))?;

            self.handle_events().await?;
        }
        ratatui::restore();
        Ok(())
    }

    async fn handle_events(&mut self) -> io::Result<()> {
        let timeout = (1000.0 / self.refresh_rate) as u64;
        if event::poll(Duration::from_millis(600000))? {
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
                        let mut log_level = self.log_level.lock().unwrap();
                        *log_level = match log_level.as_str() {
                            "debug" => "info".to_string(),
                            "info" => "warn".to_string(),
                            "warn" => "error".to_string(),
                            _ => "debug".to_string(),
                        };
                    }
                    KeyCode::Right => {
                        let mut log_level = self.log_level.lock().unwrap();
                        *log_level = match log_level.as_str() {
                            "debug" => "error".to_string(),
                            "info" => "debug".to_string(),
                            "warn" => "info".to_string(),
                            _ => "warn".to_string(),
                        };
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

        let progress_percentage = if self.results.total > 0 {
            ((self.results.found.len() as i32 + self.results.not_found) as f64
                / self.results.total as f64
                * 100.0)
                .round() as u32
        } else {
            0
        };

        let progress_text = format!(
            "Progress: {}% | Found: {} | Total: {}",
            progress_percentage,
            self.results.found.len(),
            self.results.total
        );
        let progress_area = Rect::new(1, 1, area.width - 2, 1);

        Gauge::default()
            .gauge_style(Style::default().fg(Color::Indexed(2)).bg(Color::Indexed(0)))
            .ratio(
                (self.results.found.len() as i32 + self.results.not_found) as f64
                    / self.results.total as f64,
            )
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

        let mut displayed_list = vec![];
        for (index, result) in self.results.found.iter().enumerate() {
            if index >= self.scroll_offset && index < self.scroll_offset + visible_items as usize {
                let status_style = Style::default().fg(Color::Green);
                let row = Row::new(vec![
                    format!("{}", index + 1),
                    format!("{}.{}", result.subdomain, result.domain.clone()),
                    "Found".to_string(),
                ])
                .style(status_style);
                displayed_list.push(row);
            }
        }

        let header_style = Style::default().fg(Color::Indexed(1));

        let header = ["No.", "Domain", "Status"]
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
        ])
        .style(Style::default().fg(Color::White));

        let instructions = Line::from(" <Up/Down> Navigate".bold());
        let table = Table::new(
            // Insert separator row after header
            std::iter::once(header.clone())
                .chain(std::iter::once(separator))
                .chain(displayed_list),
            [
                Constraint::Percentage(10),
                Constraint::Fill(1),
                Constraint::Percentage(20),
            ],
        )
        .block(
            Block::default()
                .title(" Results ")
                .title_bottom(instructions.left_aligned())
                .borders(ratatui::widgets::Borders::all())
                .border_type(BorderType::Rounded),
        )
        .widths(&[
            Constraint::Percentage(10),
            Constraint::Percentage(60),
            Constraint::Percentage(30),
        ])
        .highlight_spacing(HighlightSpacing::Always);

        let table_area = Rect::new(area.x + 1, area.y + 3, area.width - 2, area.height - 4);
        table.render(table_area, buf);
    }

    fn render_logs(&self, area: Rect, buf: &mut Buffer) {
        let visible_items = area.height as usize - 4;
        let mut displayed_list = vec![];

        for (index, log) in self.logs.iter().enumerate() {
            if index < self.scroll_offset || index >= self.scroll_offset + visible_items {
                continue;
            }

            let max_desc_width = (area.width as usize * 3) / 5;
            let wrapped_description = textwrap::wrap(&log.description, max_desc_width);

            let first_row_cells = vec![
                Cell::from(format!("{}", index + 1)),
                Cell::from(wrapped_description.join("\n")), // Multi-line cell instead of extra rows
                Cell::from(log.level.clone()),
                Cell::from(log.created_at.clone()),
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

        let instructions = Line::from(" <Up/Down> Navigate | <Left/Right> Cycle Log Level ".bold());
        let logs_len = Line::from(self.logs.len().to_string());

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
                .title(format!(" Log Level: {} ", self.log_level.lock().unwrap()).bold())
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
