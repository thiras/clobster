//! Market list widget.

use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Row, Table, TableState},
};

use crate::state::{MarketStatus, Store};

/// Market list widget.
pub struct MarketList;

impl MarketList {
    /// Render the market list.
    pub fn render(frame: &mut Frame, area: Rect, store: &Store) {
        let markets = store.markets.filtered_markets();

        let header_cells = ["Market", "Status", "Yes", "No", "Volume"].iter().map(|h| {
            Cell::from(*h).style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
        });
        let header = Row::new(header_cells).height(1).bottom_margin(1);

        let rows = markets.iter().enumerate().map(|(i, market)| {
            let selected = store.markets.selected_index == Some(i);
            let style = if selected {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let status_style = match market.status {
                MarketStatus::Active => Style::default().fg(Color::Green),
                MarketStatus::Closed => Style::default().fg(Color::Red),
                MarketStatus::Resolved => Style::default().fg(Color::Blue),
                MarketStatus::Paused => Style::default().fg(Color::Yellow),
            };

            // Get outcome prices
            let yes_price = market
                .outcomes
                .first()
                .map(|o| format!("{:.2}¢", o.mid_price() * rust_decimal::Decimal::ONE_HUNDRED))
                .unwrap_or_default();
            let no_price = market
                .outcomes
                .get(1)
                .map(|o| format!("{:.2}¢", o.mid_price() * rust_decimal::Decimal::ONE_HUNDRED))
                .unwrap_or_default();

            let cells = vec![
                Cell::from(truncate_string(&market.question, 50)),
                Cell::from(format!("{}", market.status)).style(status_style),
                Cell::from(yes_price).style(Style::default().fg(Color::Green)),
                Cell::from(no_price).style(Style::default().fg(Color::Red)),
                Cell::from(format!("${:.0}", market.volume)),
            ];

            Row::new(cells).style(style).height(1)
        });

        let table = Table::new(
            rows,
            [
                Constraint::Percentage(50),
                Constraint::Length(10),
                Constraint::Length(10),
                Constraint::Length(10),
                Constraint::Length(12),
            ],
        )
        .header(header)
        .block(
            Block::default()
                .title(format!(" Markets ({}) ", store.markets.markets.len()))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol("▶ ");

        let mut state = TableState::default();
        state.select(store.markets.selected_index);

        frame.render_stateful_widget(table, area, &mut state);

        // Render loading indicator if loading
        if store.markets.loading {
            render_loading(frame, area);
        }
    }
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

fn render_loading(frame: &mut Frame, area: Rect) {
    let loading = Line::from(vec![Span::styled(
        "Loading...",
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::ITALIC),
    )]);

    let block = Block::default();
    let inner = block.inner(area);

    // Render at bottom right
    let loading_area = Rect {
        x: inner.x + inner.width - 15,
        y: inner.y + inner.height - 1,
        width: 15,
        height: 1,
    };

    frame.render_widget(ratatui::widgets::Paragraph::new(loading), loading_area);
}
