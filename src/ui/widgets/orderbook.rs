//! Orderbook widget for displaying market depth and liquidity.

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
};
use rust_decimal::Decimal;

use crate::state::{OrderBook, OrderBookDisplayMode, Store};

/// Orderbook widget for displaying bids, asks, and liquidity depth.
pub struct OrderBookWidget;

impl OrderBookWidget {
    /// Render the orderbook widget.
    pub fn render(frame: &mut Frame, area: Rect, store: &Store) {
        let orderbook = store.orderbook.selected_orderbook();

        let block = Block::default()
            .title(Self::build_title(store))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        match orderbook {
            Some(book) => {
                match store.orderbook.display_mode {
                    OrderBookDisplayMode::Split => {
                        Self::render_split_view(frame, inner, book, store.orderbook.display_levels);
                    }
                    OrderBookDisplayMode::Combined => {
                        Self::render_combined_view(frame, inner, book, store.orderbook.display_levels);
                    }
                    OrderBookDisplayMode::BidsOnly => {
                        Self::render_bids_only(frame, inner, book, store.orderbook.display_levels);
                    }
                    OrderBookDisplayMode::AsksOnly => {
                        Self::render_asks_only(frame, inner, book, store.orderbook.display_levels);
                    }
                }
            }
            None => {
                Self::render_no_data(frame, inner, store);
            }
        }
    }

    fn build_title(store: &Store) -> String {
        let outcome_name = if store.orderbook.selected_outcome_index == 0 {
            "Yes"
        } else {
            "No"
        };
        
        let mode = match store.orderbook.display_mode {
            OrderBookDisplayMode::Split => "Split",
            OrderBookDisplayMode::Combined => "Combined",
            OrderBookDisplayMode::BidsOnly => "Bids",
            OrderBookDisplayMode::AsksOnly => "Asks",
        };

        format!(" Order Book - {} ({}) ", outcome_name, mode)
    }

    /// Render split view with bids on left, asks on right.
    fn render_split_view(frame: &mut Frame, area: Rect, book: &OrderBook, levels: usize) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Summary
                Constraint::Min(0),    // Book
            ])
            .split(area);

        Self::render_summary(frame, chunks[0], book);

        let book_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[1]);

        Self::render_bids_table(frame, book_chunks[0], book, levels, true);
        Self::render_asks_table(frame, book_chunks[1], book, levels, false);
    }

    /// Render combined view with asks on top, bids on bottom.
    fn render_combined_view(frame: &mut Frame, area: Rect, book: &OrderBook, levels: usize) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Summary
                Constraint::Min(0),    // Book
            ])
            .split(area);

        Self::render_summary(frame, chunks[0], book);

        let book_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[1]);

        // Asks on top (reversed order so lowest ask is at bottom)
        Self::render_asks_table(frame, book_chunks[0], book, levels, true);
        // Bids on bottom
        Self::render_bids_table(frame, book_chunks[1], book, levels, false);
    }

    /// Render bids only view.
    fn render_bids_only(frame: &mut Frame, area: Rect, book: &OrderBook, levels: usize) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Summary
                Constraint::Min(0),    // Book
            ])
            .split(area);

        Self::render_summary(frame, chunks[0], book);
        Self::render_bids_table(frame, chunks[1], book, levels, false);
    }

    /// Render asks only view.
    fn render_asks_only(frame: &mut Frame, area: Rect, book: &OrderBook, levels: usize) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Summary
                Constraint::Min(0),    // Book
            ])
            .split(area);

        Self::render_summary(frame, chunks[0], book);
        Self::render_asks_table(frame, chunks[1], book, levels, false);
    }

    /// Render orderbook summary (spread, mid price, depth).
    fn render_summary(frame: &mut Frame, area: Rect, book: &OrderBook) {
        let best_bid = book.best_bid().map(|l| l.price);
        let best_ask = book.best_ask().map(|l| l.price);
        let mid_price = book.mid_price();
        let spread = book.spread();
        let spread_pct = book.spread_percent();

        let bid_depth = book.total_bid_depth();
        let ask_depth = book.total_ask_depth();

        let summary_parts = vec![
            Span::styled("Bid: ", Style::default().fg(Color::Gray)),
            Span::styled(
                best_bid.map(|p| format!("{:.2}¢", p * Decimal::ONE_HUNDRED)).unwrap_or_else(|| "-".to_string()),
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled("Ask: ", Style::default().fg(Color::Gray)),
            Span::styled(
                best_ask.map(|p| format!("{:.2}¢", p * Decimal::ONE_HUNDRED)).unwrap_or_else(|| "-".to_string()),
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled("Mid: ", Style::default().fg(Color::Gray)),
            Span::styled(
                mid_price.map(|p| format!("{:.2}¢", p * Decimal::ONE_HUNDRED)).unwrap_or_else(|| "-".to_string()),
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled("Spread: ", Style::default().fg(Color::Gray)),
            Span::styled(
                spread.map(|s| format!("{:.2}¢", s * Decimal::ONE_HUNDRED)).unwrap_or_else(|| "-".to_string()),
                Style::default().fg(Color::Cyan),
            ),
            Span::styled(
                spread_pct.map(|p| format!(" ({:.2}%)", p)).unwrap_or_default(),
                Style::default().fg(Color::DarkGray),
            ),
        ];

        let depth_parts = vec![
            Span::styled("Bid Depth: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:.2}", bid_depth),
                Style::default().fg(Color::Green),
            ),
            Span::raw("  "),
            Span::styled("Ask Depth: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:.2}", ask_depth),
                Style::default().fg(Color::Red),
            ),
            Span::raw("  "),
            Span::styled("Imbalance: ", Style::default().fg(Color::Gray)),
            Span::styled(
                Self::format_imbalance(bid_depth, ask_depth),
                Self::imbalance_color(bid_depth, ask_depth),
            ),
        ];

        let summary = Paragraph::new(vec![
            Line::from(summary_parts),
            Line::from(depth_parts),
        ])
        .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(Color::DarkGray)));

        frame.render_widget(summary, area);
    }

    fn format_imbalance(bid_depth: Decimal, ask_depth: Decimal) -> String {
        let total = bid_depth + ask_depth;
        if total.is_zero() {
            return "N/A".to_string();
        }
        let imbalance = (bid_depth - ask_depth) / total * Decimal::ONE_HUNDRED;
        if imbalance >= Decimal::ZERO {
            format!("+{:.1}% buyers", imbalance)
        } else {
            format!("{:.1}% sellers", imbalance.abs())
        }
    }

    fn imbalance_color(bid_depth: Decimal, ask_depth: Decimal) -> Style {
        if bid_depth > ask_depth {
            Style::default().fg(Color::Green)
        } else if ask_depth > bid_depth {
            Style::default().fg(Color::Red)
        } else {
            Style::default().fg(Color::Yellow)
        }
    }

    /// Render bids table.
    fn render_bids_table(frame: &mut Frame, area: Rect, book: &OrderBook, levels: usize, show_header: bool) {
        let max_depth = book.total_bid_depth();
        let mut cumulative = Decimal::ZERO;

        let bids: Vec<_> = book.bids.iter().take(levels).collect();

        let rows = bids.iter().map(|level| {
            cumulative += level.size;
            let depth_pct = if max_depth.is_zero() {
                0.0
            } else {
                (cumulative / max_depth * Decimal::ONE_HUNDRED).to_string().parse::<f64>().unwrap_or(0.0)
            };

            let depth_bar = Self::depth_bar(depth_pct, 10);

            Row::new(vec![
                Cell::from(format!("{:.2}¢", level.price * Decimal::ONE_HUNDRED))
                    .style(Style::default().fg(Color::Green)),
                Cell::from(format!("{:.2}", level.size))
                    .style(Style::default().fg(Color::White)),
                Cell::from(format!("{:.2}", cumulative))
                    .style(Style::default().fg(Color::DarkGray)),
                Cell::from(depth_bar),
            ])
        });

        let constraints = vec![
            Constraint::Length(10), // Price
            Constraint::Length(12), // Size
            Constraint::Length(12), // Cumulative
            Constraint::Min(10),    // Depth bar
        ];

        let table = if show_header {
            let header = Row::new(vec![
                Cell::from("Bid Price").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Cell::from("Size").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Cell::from("Total").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Cell::from("Depth").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ])
            .height(1)
            .bottom_margin(1);

            Table::new(rows, constraints)
                .header(header)
                .block(Block::default().title(" Bids (Buyers) ").title_style(Style::default().fg(Color::Green)))
        } else {
            Table::new(rows, constraints)
                .block(Block::default().title(" Bids (Buyers) ").title_style(Style::default().fg(Color::Green)))
        };

        frame.render_widget(table, area);
    }

    /// Render asks table.
    fn render_asks_table(frame: &mut Frame, area: Rect, book: &OrderBook, levels: usize, reversed: bool) {
        let max_depth = book.total_ask_depth();
        let mut cumulative = Decimal::ZERO;

        let asks: Vec<_> = if reversed {
            book.asks.iter().take(levels).collect::<Vec<_>>().into_iter().rev().collect()
        } else {
            book.asks.iter().take(levels).collect()
        };

        // For reversed view, we need to calculate cumulative from the bottom
        let mut cumulatives: Vec<Decimal> = Vec::new();
        if reversed {
            let forward_asks: Vec<_> = book.asks.iter().take(levels).collect();
            let mut cum = Decimal::ZERO;
            for ask in forward_asks.iter() {
                cum += ask.size;
                cumulatives.push(cum);
            }
            cumulatives.reverse();
        }

        let rows = asks.iter().enumerate().map(|(i, level)| {
            if reversed {
                cumulative = cumulatives.get(i).copied().unwrap_or(Decimal::ZERO);
            } else {
                cumulative += level.size;
            }

            let depth_pct = if max_depth.is_zero() {
                0.0
            } else {
                (cumulative / max_depth * Decimal::ONE_HUNDRED).to_string().parse::<f64>().unwrap_or(0.0)
            };

            let depth_bar = Self::depth_bar(depth_pct, 10);

            Row::new(vec![
                Cell::from(format!("{:.2}¢", level.price * Decimal::ONE_HUNDRED))
                    .style(Style::default().fg(Color::Red)),
                Cell::from(format!("{:.2}", level.size))
                    .style(Style::default().fg(Color::White)),
                Cell::from(format!("{:.2}", cumulative))
                    .style(Style::default().fg(Color::DarkGray)),
                Cell::from(depth_bar),
            ])
        });

        let constraints = vec![
            Constraint::Length(10), // Price
            Constraint::Length(12), // Size
            Constraint::Length(12), // Cumulative
            Constraint::Min(10),    // Depth bar
        ];

        let table = if !reversed {
            let header = Row::new(vec![
                Cell::from("Ask Price").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Cell::from("Size").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Cell::from("Total").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Cell::from("Depth").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ])
            .height(1)
            .bottom_margin(1);

            Table::new(rows, constraints)
                .header(header)
                .block(Block::default().title(" Asks (Sellers) ").title_style(Style::default().fg(Color::Red)))
        } else {
            Table::new(rows, constraints)
                .block(Block::default().title(" Asks (Sellers) ").title_style(Style::default().fg(Color::Red)))
        };

        frame.render_widget(table, area);
    }

    /// Create a visual depth bar.
    fn depth_bar(percent: f64, width: usize) -> String {
        let filled = ((percent / 100.0) * width as f64).round() as usize;
        let filled = filled.min(width);
        let empty = width.saturating_sub(filled);
        format!("{}{}", "█".repeat(filled), "░".repeat(empty))
    }

    /// Render no data message.
    fn render_no_data(frame: &mut Frame, area: Rect, store: &Store) {
        let message = if store.orderbook.selected_market_orderbook().map(|m| m.loading).unwrap_or(false) {
            "Loading orderbook..."
        } else if store.orderbook.selected_market_id.is_none() {
            "Select a market to view orderbook"
        } else {
            "No orderbook data available"
        };

        let paragraph = Paragraph::new(message)
            .style(Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC))
            .alignment(Alignment::Center);

        // Center vertically
        let vertical_center = Rect {
            x: area.x,
            y: area.y + area.height / 2,
            width: area.width,
            height: 1,
        };

        frame.render_widget(paragraph, vertical_center);
    }
}

/// Compact orderbook summary widget for embedding in other views.
pub struct OrderBookSummaryWidget;

impl OrderBookSummaryWidget {
    /// Render a compact orderbook summary.
    pub fn render(frame: &mut Frame, area: Rect, book: &OrderBook) {
        let best_bid = book.best_bid().map(|l| l.price);
        let best_ask = book.best_ask().map(|l| l.price);
        let spread = book.spread();
        let bid_depth = book.total_bid_depth();
        let ask_depth = book.total_ask_depth();

        let lines = vec![
            Line::from(vec![
                Span::styled("Bid: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    best_bid.map(|p| format!("{:.2}¢", p * Decimal::ONE_HUNDRED)).unwrap_or_else(|| "-".to_string()),
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                ),
                Span::raw(" / "),
                Span::styled("Ask: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    best_ask.map(|p| format!("{:.2}¢", p * Decimal::ONE_HUNDRED)).unwrap_or_else(|| "-".to_string()),
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("Spread: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    spread.map(|s| format!("{:.2}¢", s * Decimal::ONE_HUNDRED)).unwrap_or_else(|| "-".to_string()),
                    Style::default().fg(Color::Cyan),
                ),
                Span::raw(" | "),
                Span::styled("Depth: ", Style::default().fg(Color::Gray)),
                Span::styled(format!("{:.0}", bid_depth), Style::default().fg(Color::Green)),
                Span::raw("/"),
                Span::styled(format!("{:.0}", ask_depth), Style::default().fg(Color::Red)),
            ]),
        ];

        let paragraph = Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title(" Liquidity "));

        frame.render_widget(paragraph, area);
    }
}
