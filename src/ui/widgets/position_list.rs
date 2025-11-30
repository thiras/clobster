//! Position list widget.

use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Row, Table, TableState},
};

use crate::state::Store;
use rust_decimal::Decimal;

/// Position list widget.
pub struct PositionList;

impl PositionList {
    /// Render the position list.
    pub fn render(frame: &mut Frame, area: Rect, store: &Store) {
        let positions = &store.portfolio.positions;

        let header_cells = [
            "Market",
            "Outcome",
            "Size",
            "Avg Price",
            "Current",
            "P&L",
            "P&L %",
        ]
        .iter()
        .map(|h| {
            Cell::from(*h).style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
        });
        let header = Row::new(header_cells).height(1).bottom_margin(1);

        let rows = positions.iter().enumerate().map(|(i, position)| {
            let selected = store.portfolio.selected_position == Some(i);
            let style = if selected {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let pnl_style = if position.unrealized_pnl >= Decimal::ZERO {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Red)
            };

            let pnl_sign = if position.unrealized_pnl >= Decimal::ZERO {
                "+"
            } else {
                ""
            };

            let cells = vec![
                Cell::from(truncate_string(&position.market_question, 25)),
                Cell::from(position.outcome_name.clone()),
                Cell::from(format!("{:.2}", position.size)),
                Cell::from(format!("{:.2}¢", position.avg_price * Decimal::ONE_HUNDRED)),
                Cell::from(format!(
                    "{:.2}¢",
                    position.current_price * Decimal::ONE_HUNDRED
                )),
                Cell::from(format!("{}${:.2}", pnl_sign, position.unrealized_pnl)).style(pnl_style),
                Cell::from(format!(
                    "{}{:.1}%",
                    pnl_sign, position.unrealized_pnl_percent
                ))
                .style(pnl_style),
            ];

            Row::new(cells).style(style).height(1)
        });

        let table = Table::new(
            rows,
            [
                Constraint::Percentage(25),
                Constraint::Length(10),
                Constraint::Length(10),
                Constraint::Length(12),
                Constraint::Length(12),
                Constraint::Length(12),
                Constraint::Length(10),
            ],
        )
        .header(header)
        .block(
            Block::default()
                .title(format!(
                    " Positions ({}) | Value: ${:.2} | P&L: ${:.2} ",
                    positions.len(),
                    store.portfolio.total_value,
                    store.portfolio.total_unrealized_pnl
                ))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol("▶ ");

        let mut state = TableState::default();
        state.select(store.portfolio.selected_position);

        frame.render_stateful_widget(table, area, &mut state);

        // Render loading indicator if loading
        if store.portfolio.loading {
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

    let loading_area = Rect {
        x: inner.x + inner.width - 15,
        y: inner.y + inner.height - 1,
        width: 15,
        height: 1,
    };

    frame.render_widget(ratatui::widgets::Paragraph::new(loading), loading_area);
}
