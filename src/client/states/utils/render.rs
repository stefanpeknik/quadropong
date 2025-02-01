use std::rc::Rc;

use ratatui::{
    layout::{Constraint, Direction, Flex, Layout, Margin, Position, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

use crate::common::models::{BallDto, PlayerDto, PlayerPosition};

use super::widget::{get_widget_text, Widget};

const SERVER_GAME_BOARD_SIZE: f32 = 10.0;

pub fn into_title(input: &str) -> String {
    input
        .to_uppercase()
        .chars()
        .map(|c| c.to_string())
        .collect::<Vec<_>>()
        .join(" ")
}

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
    Layout::vertical(
        std::iter::repeat(Constraint::Percentage(100 / num_rects as u16)).take(num_rects),
    )
    .split(rect)
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
            Line::from(format!(">{}<", text)).bold()
        } else {
            Line::from(text.as_str())
        };

        render_text_in_center_of_rect(frame, Paragraph::new(text), *area);
    }
}

/// Renders a list of players
pub fn render_player_list(frame: &mut Frame, items: &Vec<(String, bool)>, rect: Rect) {
    let vertical_text_spaces = evenly_distanced_rects(rect, 4);

    for ((text, is_ready), area) in items.iter().zip(vertical_text_spaces.iter()) {
        let [text_area, ready_area] =
            Layout::horizontal(vec![Constraint::Percentage(100), Constraint::Length(1)])
                .areas(*area);

        let ready_symbol = if *is_ready { "✓".green() } else { "X".red() };

        frame.render_widget(
            Paragraph::new(Line::from(text.clone())).centered(),
            evenly_distanced_rects(text_area, 2)[1],
        );
        frame.render_widget(
            Paragraph::new(Line::from(ready_symbol).right_aligned()),
            evenly_distanced_rects(ready_area, 2)[1],
        );
    }
}

pub fn render_settings(
    frame: &mut Frame,
    items: &[String],
    all_widgets: &Vec<&Widget>,
    active_widget: &Widget,
    selected_index: usize,
    rect: Rect,
) {
    let mut constraints = Vec::new();

    for _ in 0..items.len() {
        constraints.push(Constraint::Length(3));
        constraints.push(Constraint::Percentage(10));
    }

    let layout = Layout::vertical(constraints).split(rect);

    let input_instructions =
        Line::from(vec![" Paste ".into(), "<Tab> ".green().bold()]).right_aligned();
    let slider_instructions = Line::from(vec![
        " Right".into(),
        " \u{2190} ".green().into(),
        "| Left".into(),
        " \u{2192} ".green().into(),
    ])
    .right_aligned();

    for (i, ((text, areas), widget)) in items
        .iter()
        .zip(layout.chunks(2))
        .zip(all_widgets.iter()) // skip the first area as that only creates space from the top
        .enumerate()
    {
        if let [widget_area, _] = areas {
            let block_text = if i == selected_index {
                Line::from(format!(" >{}< ", text)).bold().centered()
            } else {
                Line::from(text.as_str()).centered()
            };
            let widget_block = Block::bordered().title(block_text.clone());

            let inner_input_area = widget_block.inner(*widget_area);
            let mut color_check = "";
            let mut style = Style::default();

            match widget {
                Widget::Input(input) => {
                    if let Widget::Input(active_input) = active_widget {
                        if std::ptr::eq(input, active_input) {
                            frame.set_cursor_position(Position::new(
                                inner_input_area.x + input.char_index as u16,
                                inner_input_area.y,
                            ));
                        }
                    }
                    frame.render_widget(
                        widget_block.title_bottom(input_instructions.clone()),
                        *widget_area,
                    );
                }
                Widget::Slider(slider) => {
                    frame.render_widget(
                        widget_block.title_bottom(slider_instructions.clone()),
                        *widget_area,
                    );
                    style = Style::default().bg(slider.clone().get_color());
                    color_check = "     ";
                }
            }

            frame.render_widget(
                Paragraph::new(Line::from(vec![
                    Span::raw(get_widget_text(&widget)),
                    Span::raw(" "),
                    Span::styled(color_check, style),
                ])),
                inner_input_area,
            );
        }
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

/// Helper function to scale dimensions such that width is exactly 2 times height,
/// while ensuring neither exceeds the original dimensions.
fn scale_dimensions(original_height: u16, original_width: u16) -> (u16, u16) {
    // Target ratio is width = 2 * height
    let target_ratio = 2;

    // Calculate the maximum possible height and width based on the original dimensions
    let max_height = original_height;
    let max_width = original_width;

    // Calculate the height and width that satisfy width = 2 * height
    // while ensuring neither exceeds the original dimensions
    let new_height = max_height.min(max_width / target_ratio);
    let new_width = target_ratio * new_height;

    (new_height, new_width)
}

/// Helper function to calculate the game area and scaling factors
pub fn calculate_game_area(frame: &Frame) -> (Rect, Rect, f32, f32) {
    let terminal_size = frame.area();
    let (game_area_bounding_box_height, game_area_bounding_box_width) =
        scale_dimensions(terminal_size.height, terminal_size.width);

    let game_area_bounding_box = Rect {
        x: terminal_size.x.saturating_add(
            (terminal_size
                .width
                .saturating_sub(game_area_bounding_box_width))
                / 2,
        ),
        y: terminal_size.y.saturating_add(
            (terminal_size
                .height
                .saturating_sub(game_area_bounding_box_height))
                / 2,
        ),
        width: game_area_bounding_box_width,
        height: game_area_bounding_box_height,
    };

    let game_area = game_area_bounding_box.inner(Margin {
        horizontal: 2, // Account for the scaled dimensions
        vertical: 1,
    });

    let scale_x = game_area.width as f32 / SERVER_GAME_BOARD_SIZE;
    let scale_y = game_area.height as f32 / SERVER_GAME_BOARD_SIZE;

    (game_area_bounding_box, game_area, scale_x, scale_y)
}

/// Render a single player's paddle
pub fn render_player(
    player: &PlayerDto,
    player_color: ratatui::style::Color,
    frame: &mut Frame,
    game_area: &Rect,
    scale_x: f32,
    scale_y: f32,
) {
    const PLAYER_VERTICAL_BODY: &str = "█";
    const PLAYER_UP_BODY: &str = "▄";
    const PLAYER_BOTTOM_BODY: &str = "▀";

    let player_style = ratatui::style::Style::default().fg(player_color);

    match player.position {
        Some(PlayerPosition::Top) | Some(PlayerPosition::Bottom) => {
            // Calculate horizontal paddle dimensions and position
            let paddle_length = (player.paddle_width * scale_x) as u16;
            let paddle_thickness = 1; // Paddle depth is 1 character

            let paddle_center = (player.paddle_position * scale_x) as u16;
            let paddle_x = game_area
                .x
                .saturating_add(paddle_center)
                .saturating_sub(paddle_length / 2);
            match player.position {
                Some(PlayerPosition::Top) => {
                    let paddle_y = game_area.y;
                    frame.render_widget(
                        Paragraph::new(PLAYER_UP_BODY.repeat(paddle_length as usize))
                            .style(player_style),
                        Rect {
                            x: paddle_x,
                            y: paddle_y,
                            width: paddle_length,
                            height: paddle_thickness,
                        },
                    );
                }
                Some(PlayerPosition::Bottom) => {
                    let paddle_y = game_area.y + game_area.height - paddle_thickness;
                    frame.render_widget(
                        Paragraph::new(PLAYER_BOTTOM_BODY.repeat(paddle_length as usize))
                            .style(player_style),
                        Rect {
                            x: paddle_x,
                            y: paddle_y,
                            width: paddle_length,
                            height: paddle_thickness,
                        },
                    );
                }
                _ => {}
            }
        }
        Some(PlayerPosition::Left) | Some(PlayerPosition::Right) => {
            // Calculate vertical paddle dimensions and position
            let paddle_length = (player.paddle_width * scale_y) as u16;
            let paddle_thickness = 1; // Paddle depth is 1 character
            let paddle_center = (player.paddle_position * scale_y) as u16;
            let paddle_y = game_area
                .y
                .saturating_add(paddle_center)
                .saturating_sub(paddle_length / 2);
            match player.position {
                Some(PlayerPosition::Left) => {
                    let paddle_x = game_area.x;
                    frame.render_widget(
                        Paragraph::new(
                            format!("{}\n", PLAYER_VERTICAL_BODY).repeat(paddle_length as usize),
                        )
                        .style(player_style),
                        Rect {
                            x: paddle_x,
                            y: paddle_y,
                            width: paddle_thickness,
                            height: paddle_length,
                        },
                    );
                }
                Some(PlayerPosition::Right) => {
                    let paddle_x = game_area
                        .x
                        .saturating_add(game_area.width)
                        .saturating_sub(paddle_thickness);
                    frame.render_widget(
                        Paragraph::new(
                            format!("{}\n", PLAYER_VERTICAL_BODY).repeat(paddle_length as usize),
                        )
                        .style(player_style),
                        Rect {
                            x: paddle_x,
                            y: paddle_y,
                            width: paddle_thickness,
                            height: paddle_length,
                        },
                    );
                }
                _ => {}
            }
        }
        None => {}
    }
}

/// Render the ball
pub fn render_ball(
    ball: &BallDto,
    frame: &mut Frame,
    game_area: &Rect,
    scale_x: f32,
    scale_y: f32,
) {
    // Calculate ball position in terminal coordinates
    let ball_x = game_area
        .x
        .saturating_add((ball.position.x * scale_x) as u16);
    let ball_y = game_area
        .y
        .saturating_add((ball.position.y * scale_y) as u16);

    // Render the ball as a single character
    frame.render_widget(
        Paragraph::new("●")
            .style(ratatui::style::Style::default().fg(ratatui::style::Color::White)),
        Rect {
            x: ball_x,
            y: ball_y,
            width: 1,
            height: 1,
        },
    );
}
