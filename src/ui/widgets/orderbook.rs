//! Order book depth widget.

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Bar, BarChart, BarGroup, Block, Borders, Cell, Paragraph, Row, Table},
};
use rust_decimal::Decimal;

use crate::state::{OrderBookDepth, Store};

/// Order book widget displaying bids and asks.
pub struct OrderBook;

impl OrderBook {
    /// Render the order book for the selected market.
    pub fn render(frame: &mut Frame, area: Rect, store: &Store) {
        // Get the selected order book
        let book = match store.orderbooks.selected_book() {
            Some(book) => book,
            None => {
                Self::render_empty(frame, area);
                return;
            }
        };

        let depth = store.orderbooks.display_depth;

        // Split area: stats on top, order book below
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(5), Constraint::Min(10)])
            .split(area);

        Self::render_stats(frame, chunks[0], book, depth);
        Self::render_depth(frame, chunks[1], book, depth);

        // Render loading indicator if loading
        if store.orderbooks.loading {
            Self::render_loading(frame, area);
        }
    }

    /// Render order book statistics.
    fn render_stats(frame: &mut Frame, area: Rect, book: &OrderBookDepth, depth: usize) {
        let best_bid = book
            .best_bid_price()
            .map(|p| format!("{:.2}¢", p * Decimal::ONE_HUNDRED))
            .unwrap_or_else(|| "-".to_string());
        let best_ask = book
            .best_ask_price()
            .map(|p| format!("{:.2}¢", p * Decimal::ONE_HUNDRED))
            .unwrap_or_else(|| "-".to_string());
        let mid = book
            .mid_price()
            .map(|p| format!("{:.2}¢", p * Decimal::ONE_HUNDRED))
            .unwrap_or_else(|| "-".to_string());
        let spread = book
            .spread()
            .map(|s| format!("{:.2}¢", s * Decimal::ONE_HUNDRED))
            .unwrap_or_else(|| "-".to_string());
        let spread_pct = book
            .spread_percent()
            .map(|s| format!("{:.2}%", s))
            .unwrap_or_else(|| "-".to_string());
        let imbalance = book
            .imbalance(depth)
            .map(|i| {
                let pct = i * Decimal::ONE_HUNDRED;
                if i > Decimal::ZERO {
                    format!("+{:.1}% BUY", pct)
                } else if i < Decimal::ZERO {
                    format!("{:.1}% SELL", pct)
                } else {
                    "0% NEUTRAL".to_string()
                }
            })
            .unwrap_or_else(|| "-".to_string());

        let imbalance_color = book.imbalance(depth).map_or(Color::Gray, |i| {
            if i > Decimal::ZERO {
                Color::Green
            } else if i < Decimal::ZERO {
                Color::Red
            } else {
                Color::Yellow
            }
        });

        let stats_text = vec![
            Line::from(vec![
                Span::styled("Best Bid: ", Style::default().fg(Color::Gray)),
                Span::styled(&best_bid, Style::default().fg(Color::Green)),
                Span::raw("  │  "),
                Span::styled("Best Ask: ", Style::default().fg(Color::Gray)),
                Span::styled(&best_ask, Style::default().fg(Color::Red)),
                Span::raw("  │  "),
                Span::styled("Mid: ", Style::default().fg(Color::Gray)),
                Span::styled(&mid, Style::default().fg(Color::Yellow)),
            ]),
            Line::from(vec![
                Span::styled("Spread: ", Style::default().fg(Color::Gray)),
                Span::styled(format!("{} ({})", spread, spread_pct), Style::default().fg(Color::Cyan)),
                Span::raw("  │  "),
                Span::styled("Imbalance: ", Style::default().fg(Color::Gray)),
                Span::styled(&imbalance, Style::default().fg(imbalance_color)),
            ]),
            Line::from(vec![
                Span::styled("Bid Depth: ", Style::default().fg(Color::Gray)),
                Span::styled(format!("{}", book.bid_depth()), Style::default().fg(Color::Green)),
                Span::raw("  │  "),
                Span::styled("Ask Depth: ", Style::default().fg(Color::Gray)),
                Span::styled(format!("{}", book.ask_depth()), Style::default().fg(Color::Red)),
            ]),
        ];

        let stats = Paragraph::new(stats_text).block(
            Block::default()
                .title(" Order Book Stats ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        );

        frame.render_widget(stats, area);
    }

    /// Render the order book depth (bids and asks).
    fn render_depth(frame: &mut Frame, area: Rect, book: &OrderBookDepth, depth: usize) {
        // Split into bids (left) and asks (right)
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        Self::render_bids(frame, chunks[0], book, depth);
        Self::render_asks(frame, chunks[1], book, depth);
    }

    /// Render bid side of the order book.
    fn render_bids(frame: &mut Frame, area: Rect, book: &OrderBookDepth, depth: usize) {
        let header_cells = ["Price", "Size", "Total"]
            .iter()
            .map(|h| {
                Cell::from(*h).style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
            });
        let header = Row::new(header_cells).height(1).bottom_margin(1);

        // Calculate cumulative totals
        let mut cumulative = Decimal::ZERO;
        let max_cumulative = book.bid_volume(depth);

        let rows = book.bids.iter().take(depth).map(|level| {
            cumulative += level.size;
            let fill_pct = if max_cumulative.is_zero() {
                0.0
            } else {
                (cumulative / max_cumulative).to_string().parse::<f64>().unwrap_or(0.0)
            };

            // Create a visual bar based on cumulative size
            let bar_width = ((area.width as f64 * 0.3 * fill_pct) as usize).max(0);
            let bar = "█".repeat(bar_width);

            let cells = vec![
                Cell::from(format!("{:.2}¢", level.price * Decimal::ONE_HUNDRED))
                    .style(Style::default().fg(Color::Green)),
                Cell::from(format!("{:.2}", level.size)),
                Cell::from(format!("{:.2} {}", cumulative, bar))
                    .style(Style::default().fg(Color::DarkGray)),
            ];

            Row::new(cells).height(1)
        });

        let table = Table::new(
            rows,
            [
                Constraint::Length(10),
                Constraint::Length(10),
                Constraint::Min(15),
            ],
        )
        .header(header)
        .block(
            Block::default()
                .title(format!(" Bids ({}) ", book.bid_depth()))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green)),
        );

        frame.render_widget(table, area);
    }

    /// Render ask side of the order book.
    fn render_asks(frame: &mut Frame, area: Rect, book: &OrderBookDepth, depth: usize) {
        let header_cells = ["Price", "Size", "Total"]
            .iter()
            .map(|h| {
                Cell::from(*h).style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
            });
        let header = Row::new(header_cells).height(1).bottom_margin(1);

        // Calculate cumulative totals
        let mut cumulative = Decimal::ZERO;
        let max_cumulative = book.ask_volume(depth);

        let rows = book.asks.iter().take(depth).map(|level| {
            cumulative += level.size;
            let fill_pct = if max_cumulative.is_zero() {
                0.0
            } else {
                (cumulative / max_cumulative).to_string().parse::<f64>().unwrap_or(0.0)
            };

            // Create a visual bar based on cumulative size
            let bar_width = ((area.width as f64 * 0.3 * fill_pct) as usize).max(0);
            let bar = "█".repeat(bar_width);

            let cells = vec![
                Cell::from(format!("{:.2}¢", level.price * Decimal::ONE_HUNDRED))
                    .style(Style::default().fg(Color::Red)),
                Cell::from(format!("{:.2}", level.size)),
                Cell::from(format!("{:.2} {}", cumulative, bar))
                    .style(Style::default().fg(Color::DarkGray)),
            ];

            Row::new(cells).height(1)
        });

        let table = Table::new(
            rows,
            [
                Constraint::Length(10),
                Constraint::Length(10),
                Constraint::Min(15),
            ],
        )
        .header(header)
        .block(
            Block::default()
                .title(format!(" Asks ({}) ", book.ask_depth()))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Red)),
        );

        frame.render_widget(table, area);
    }

    /// Render empty state when no order book is selected.
    fn render_empty(frame: &mut Frame, area: Rect) {
        let text = vec![
            Line::from(""),
            Line::from(Span::styled(
                "No order book selected",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Select a market to view its order book",
                Style::default().fg(Color::DarkGray),
            )),
        ];

        let paragraph = Paragraph::new(text)
            .block(
                Block::default()
                    .title(" Order Book ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray)),
            )
            .alignment(ratatui::layout::Alignment::Center);

        frame.render_widget(paragraph, area);
    }

    /// Render loading indicator.
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
            x: inner.x + inner.width.saturating_sub(15),
            y: inner.y + inner.height.saturating_sub(1),
            width: 15.min(inner.width),
            height: 1,
        };

        frame.render_widget(Paragraph::new(loading), loading_area);
    }
}

/// Compact order book widget for side panels.
pub struct OrderBookCompact;

impl OrderBookCompact {
    /// Render a compact order book view.
    pub fn render(frame: &mut Frame, area: Rect, book: &OrderBookDepth, depth: usize) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(5)])
            .split(area);

        // Spread info
        let spread_info = Self::build_spread_line(book);
        let spread_widget = Paragraph::new(spread_info).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        );
        frame.render_widget(spread_widget, chunks[0]);

        // Compact depth view
        Self::render_compact_depth(frame, chunks[1], book, depth);
    }

    fn build_spread_line(book: &OrderBookDepth) -> Line<'static> {
        let bid = book
            .best_bid_price()
            .map(|p| format!("{:.2}¢", p * Decimal::ONE_HUNDRED))
            .unwrap_or_else(|| "-".to_string());
        let ask = book
            .best_ask_price()
            .map(|p| format!("{:.2}¢", p * Decimal::ONE_HUNDRED))
            .unwrap_or_else(|| "-".to_string());
        let spread = book
            .spread_percent()
            .map(|s| format!("{:.1}%", s))
            .unwrap_or_else(|| "-".to_string());

        Line::from(vec![
            Span::styled(bid, Style::default().fg(Color::Green)),
            Span::raw(" / "),
            Span::styled(ask, Style::default().fg(Color::Red)),
            Span::raw(" ("),
            Span::styled(spread, Style::default().fg(Color::Cyan)),
            Span::raw(")"),
        ])
    }

    fn render_compact_depth(frame: &mut Frame, area: Rect, book: &OrderBookDepth, depth: usize) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        // Bids
        let bid_lines: Vec<Line> = book
            .bids
            .iter()
            .take(depth)
            .map(|level| {
                Line::from(vec![
                    Span::styled(
                        format!("{:.2}¢", level.price * Decimal::ONE_HUNDRED),
                        Style::default().fg(Color::Green),
                    ),
                    Span::raw(" "),
                    Span::styled(
                        format!("{:.1}", level.size),
                        Style::default().fg(Color::DarkGray),
                    ),
                ])
            })
            .collect();

        let bids = Paragraph::new(bid_lines).block(
            Block::default()
                .title(" Bids ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green)),
        );
        frame.render_widget(bids, chunks[0]);

        // Asks
        let ask_lines: Vec<Line> = book
            .asks
            .iter()
            .take(depth)
            .map(|level| {
                Line::from(vec![
                    Span::styled(
                        format!("{:.2}¢", level.price * Decimal::ONE_HUNDRED),
                        Style::default().fg(Color::Red),
                    ),
                    Span::raw(" "),
                    Span::styled(
                        format!("{:.1}", level.size),
                        Style::default().fg(Color::DarkGray),
                    ),
                ])
            })
            .collect();

        let asks = Paragraph::new(ask_lines).block(
            Block::default()
                .title(" Asks ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Red)),
        );
        frame.render_widget(asks, chunks[1]);
    }
}

/// Order book depth chart (visual representation).
pub struct OrderBookChart;

impl OrderBookChart {
    /// Render order book as a depth chart.
    pub fn render(frame: &mut Frame, area: Rect, book: &OrderBookDepth, depth: usize) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        Self::render_bid_chart(frame, chunks[0], book, depth);
        Self::render_ask_chart(frame, chunks[1], book, depth);
    }

    fn render_bid_chart(frame: &mut Frame, area: Rect, book: &OrderBookDepth, depth: usize) {
        let cumulative = book.cumulative_bids();
        let max_vol = cumulative.last().map(|(_, v)| *v).unwrap_or(Decimal::ONE);

        let bars: Vec<Bar> = cumulative
            .iter()
            .take(depth)
            .map(|(price, vol)| {
                let height = if max_vol.is_zero() {
                    0
                } else {
                    ((*vol / max_vol) * Decimal::from(100))
                        .to_string()
                        .parse::<u64>()
                        .unwrap_or(0)
                };
                Bar::default()
                    .value(height)
                    .label(Line::from(format!("{:.0}¢", *price * Decimal::ONE_HUNDRED)))
                    .style(Style::default().fg(Color::Green))
            })
            .collect();

        let chart = BarChart::default()
            .block(
                Block::default()
                    .title(" Bid Depth ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Green)),
            )
            .data(BarGroup::default().bars(&bars))
            .bar_width(3)
            .bar_gap(1)
            .direction(Direction::Horizontal);

        frame.render_widget(chart, area);
    }

    fn render_ask_chart(frame: &mut Frame, area: Rect, book: &OrderBookDepth, depth: usize) {
        let cumulative = book.cumulative_asks();
        let max_vol = cumulative.last().map(|(_, v)| *v).unwrap_or(Decimal::ONE);

        let bars: Vec<Bar> = cumulative
            .iter()
            .take(depth)
            .map(|(price, vol)| {
                let height = if max_vol.is_zero() {
                    0
                } else {
                    ((*vol / max_vol) * Decimal::from(100))
                        .to_string()
                        .parse::<u64>()
                        .unwrap_or(0)
                };
                Bar::default()
                    .value(height)
                    .label(Line::from(format!("{:.0}¢", *price * Decimal::ONE_HUNDRED)))
                    .style(Style::default().fg(Color::Red))
            })
            .collect();

        let chart = BarChart::default()
            .block(
                Block::default()
                    .title(" Ask Depth ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Red)),
            )
            .data(BarGroup::default().bars(&bars))
            .bar_width(3)
            .bar_gap(1)
            .direction(Direction::Horizontal);

        frame.render_widget(chart, area);
    }
}
