use std::rc::Rc;

use ratatui::{
    layout::{Alignment, Constraint, Direction, Flex, Layout, Position, Rect},
    style::{Color, Modifier, Style, Stylize},
    symbols::border,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};
use uuid::Uuid;

use crate::game_models::{
    ball::Ball,
    player::{Player, PlayerPosition},
};

const SERVER_GAME_BOARD_SIZE: f32 = 10.0;

/// Draws the outer rectangle, renders it, and returns its Rect
pub fn render_outer_rectangle(
    frame: &mut Frame,
    title_text: &str,
    instructions_text: Vec<Span>,
) -> Rect {
    // Create the outer block
    let outer_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .title(Line::from(title_text.bold()).left_aligned())
        .title_bottom(Line::from(instructions_text).centered());

    // Render the outer block
    frame.render_widget(outer_block, frame.area());

    // Return the outer rectangle (entire frame area)
    frame.area()
}

/// Draws the inner rectangle, renders it, and returns its Rect
pub fn render_inner_rectangle(frame: &mut Frame, outer_rect: Rect) -> Rect {
    // Calculate the inner rectangle with 40% spacing on each side
    let inner_rect = centered_rect(outer_rect, 60, 60);

    // Create the inner block
    let inner_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Thick);

    // Render the inner block
    frame.render_widget(inner_block, inner_rect);

    // Return the inner rectangle
    inner_rect
}

/// Helper function to calculate evenly distributed rectangles within a given rectangle
pub fn evenly_distanced_rects(rect: Rect, num_rects: usize) -> Rc<[Rect]> {
    let vertical = Layout::vertical(
        std::iter::repeat(Constraint::Percentage(100 / num_rects as u16)).take(num_rects),
    )
    .split(rect);

    vertical
}

/// Renders a list of strings evenly distributed and centered within a rectangle.
/// The selected item is highlighted.
pub fn render_list(frame: &mut Frame, items: &[String], selected_index: usize, rect: Rect) {
    let vertical_text_spaces = evenly_distanced_rects(rect, items.len());

    for (i, (text, area)) in items
        .iter()
        .zip(vertical_text_spaces.iter()) // skip the first area as that only creates space from the top
        .enumerate()
    {
        let text = if i == selected_index {
            Line::from(format!("> {} <", text)).bold()
        } else {
            Line::from(text.as_str())
        };

        render_text_in_center_of_rect(frame, Paragraph::new(text), *area);
    }
}

pub fn render_text_in_center_of_rect(frame: &mut Frame, text: Paragraph, rect: Rect) {
    frame.render_widget(text.centered(), evenly_distanced_rects(rect, 2)[1]);
}

/// Helper function to calculate a centered rectangle with given horizontal and vertical percentages
pub fn centered_rect(area: Rect, horizontal_percent: u16, vertical_percent: u16) -> Rect {
    let horizontal =
        Layout::horizontal([Constraint::Percentage(horizontal_percent)]).flex(Flex::Center);
    let vertical = Layout::vertical([Constraint::Percentage(vertical_percent)]).flex(Flex::Center);
    let [rect] = vertical.areas(area);
    let [rect] = horizontal.areas(rect);

    rect
}
/// Helper function to calculate the game area and scaling factors
pub fn calculate_game_area(frame: &Frame) -> (Rect, f32, f32) {
    let terminal_size = frame.area();
    let game_area_width = terminal_size.width.min(terminal_size.height);
    let game_area = Rect {
        x: (terminal_size.width - game_area_width) / 2,
        y: (terminal_size.height - game_area_width) / 2,
        width: game_area_width,
        height: game_area_width,
    };
    let scale_x = game_area.width as f32 / 10.0;
    let scale_y = game_area.height as f32 / 10.0;
    (game_area, scale_x, scale_y)
}

/// Render all players
pub fn render_players(
    players: &[&Player],
    our_player: Uuid,
    frame: &mut Frame,
    game_area: &Rect,
    scale_x: f32,
    scale_y: f32,
) {
    for player in players {
        render_player(
            player,
            player.id == our_player,
            frame,
            game_area,
            scale_x,
            scale_y,
        );
    }
}

/// Render a single player's paddle
pub fn render_player(
    player: &Player,
    is_our_player: bool,
    frame: &mut Frame,
    game_area: &Rect,
    scale_x: f32,
    scale_y: f32,
) {
    // Calculate paddle dimensions and position
    let paddle_length = (player.paddle_width * 2.0 * scale_x) as u16;
    let paddle_thickness = 1; // Paddle depth is 1 character
    let paddle_center = (player.paddle_position * scale_x) as u16;

    let player_style = if is_our_player {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::White)
    };

    // Determine paddle position based on player side
    match player.position {
        Some(PlayerPosition::Top) => {
            let paddle_x = game_area.x + paddle_center - paddle_length / 2;
            let paddle_y = game_area.y;
            frame.render_widget(
                Paragraph::new("─".repeat(paddle_length as usize)).style(player_style),
                Rect {
                    x: paddle_x,
                    y: paddle_y,
                    width: paddle_length,
                    height: paddle_thickness,
                },
            );
        }
        Some(PlayerPosition::Bottom) => {
            let paddle_x = game_area.x + paddle_center - paddle_length / 2;
            let paddle_y = game_area.y + game_area.height - paddle_thickness;
            frame.render_widget(
                Paragraph::new("─".repeat(paddle_length as usize)).style(player_style),
                Rect {
                    x: paddle_x,
                    y: paddle_y,
                    width: paddle_length,
                    height: paddle_thickness,
                },
            );
        }
        Some(PlayerPosition::Left) => {
            let paddle_x = game_area.x;
            let paddle_y = game_area.y + paddle_center - paddle_length / 2;
            frame.render_widget(
                Paragraph::new("│".repeat(paddle_length as usize)).style(player_style),
                Rect {
                    x: paddle_x,
                    y: paddle_y,
                    width: paddle_thickness,
                    height: paddle_length,
                },
            );
        }
        Some(PlayerPosition::Right) => {
            let paddle_x = game_area.x + game_area.width - paddle_thickness;
            let paddle_y = game_area.y + paddle_center - paddle_length / 2;
            frame.render_widget(
                Paragraph::new("│".repeat(paddle_length as usize)).style(player_style),
                Rect {
                    x: paddle_x,
                    y: paddle_y,
                    width: paddle_thickness,
                    height: paddle_length,
                },
            );
        }
        None => {}
    }
}

/// Render the ball
pub fn render_ball(ball: &Ball, frame: &mut Frame, game_area: &Rect, scale_x: f32, scale_y: f32) {
    // Calculate ball position in terminal coordinates
    let ball_x = game_area.x + (ball.position.x * scale_x) as u16;
    let ball_y = game_area.y + (ball.position.y * scale_y) as u16;

    // Render the ball as a single character
    frame.render_widget(
        Paragraph::new("●").style(Style::default().fg(Color::White)),
        Rect {
            x: ball_x,
            y: ball_y,
            width: 1,
            height: 1,
        },
    );
}
