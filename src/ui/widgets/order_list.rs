//! Order list widget.

use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Row, Table, TableState},
};

use crate::state::{OrderSide, OrderStatus, Store};

/// Order list widget.
pub struct OrderList;

impl OrderList {
    /// Render the order list.
    pub fn render(frame: &mut Frame, area: Rect, store: &Store) {
        let orders = &store.orders.orders;

        let header_cells = ["Market", "Side", "Price", "Size", "Filled", "Status"]
            .iter()
            .map(|h| {
                Cell::from(*h).style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
            });
        let header = Row::new(header_cells).height(1).bottom_margin(1);

        let rows = orders.iter().enumerate().map(|(i, order)| {
            let selected = store.orders.selected_index == Some(i);
            let style = if selected {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let side_style = match order.side {
                OrderSide::Buy => Style::default().fg(Color::Green),
                OrderSide::Sell => Style::default().fg(Color::Red),
            };

            let status_style = match order.status {
                OrderStatus::Open | OrderStatus::PartiallyFilled => {
                    Style::default().fg(Color::Green)
                }
                OrderStatus::Filled => Style::default().fg(Color::Blue),
                OrderStatus::Cancelled | OrderStatus::Expired | OrderStatus::Failed => {
                    Style::default().fg(Color::Red)
                }
                OrderStatus::Pending => Style::default().fg(Color::Yellow),
            };

            let cells = vec![
                Cell::from(truncate_string(&order.market_question, 30)),
                Cell::from(format!("{:?}", order.side)).style(side_style),
                Cell::from(format!(
                    "{:.2}¢",
                    order.price * rust_decimal::Decimal::ONE_HUNDRED
                )),
                Cell::from(format!("{:.2}", order.original_size)),
                Cell::from(format!("{:.1}%", order.fill_percent())),
                Cell::from(format!("{}", order.status)).style(status_style),
            ];

            Row::new(cells).style(style).height(1)
        });

        let table = Table::new(
            rows,
            [
                Constraint::Percentage(35),
                Constraint::Length(8),
                Constraint::Length(10),
                Constraint::Length(10),
                Constraint::Length(10),
                Constraint::Length(12),
            ],
        )
        .header(header)
        .block(
            Block::default()
                .title(format!(" Orders ({} open) ", store.orders.open_count()))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol("▶ ");

        let mut state = TableState::default();
        state.select(store.orders.selected_index);

        frame.render_stateful_widget(table, area, &mut state);

        // Render loading indicator if loading
        if store.orders.loading {
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
