use super::create_lobby::CreateLobby;
use super::menu::Menu;
use super::traits::{HasOptions, ListEnum, Render, State, Update};
use super::utils::{
    draw_inner_rectangle, draw_outer_rectangle, evenly_distanced_rects, render_list,
    render_text_in_center_of_rect, Input,
};

use cli_clipboard::{ClipboardContext, ClipboardProvider};
use crossterm::event::KeyCode;
use ratatui::layout::{Constraint, Layout, Position};
use ratatui::style::{Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph};
use ratatui::Frame;

#[derive(Clone, PartialEq)]
pub enum Options {
    Create,
    Join,
}

impl ListEnum for Options {
    fn list() -> Vec<Self> {
        vec![Options::Create, Options::Join]
    }
}

impl std::fmt::Display for Options {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Options::Create => write!(f, "Create Lobby"),
            Options::Join => write!(f, "Join Lobby"),
        }
    }
}

#[derive(Clone)]
pub struct CreateOrJoinLobby {
    options: Vec<Options>,
    selected: usize,
    join_lobby_input: Input,
}

impl CreateOrJoinLobby {
    pub fn new() -> Self {
        Self {
            options: Options::list(),
            selected: 0,
            join_lobby_input: Input::new(),
        }
    }
}

impl HasOptions for CreateOrJoinLobby {
    fn next(&mut self) {
        self.selected = (self.selected + 1) % self.options.len();
    }

    fn previous(&mut self) {
        if self.selected == 0 {
            self.selected = self.options.len() - 1;
        } else {
            self.selected -= 1;
        }
    }
}

impl State for CreateOrJoinLobby {
    fn clone_box(&self) -> Box<dyn State> {
        Box::new(self.clone())
    }
}
impl Update for CreateOrJoinLobby {
    fn update(
        &mut self,
        key_code: Option<KeyCode>,
    ) -> Result<Option<Box<dyn State>>, std::io::Error> {
        if let Some(key_code) = key_code {
            // match navigation keys between options/states
            match key_code {
                KeyCode::Up => self.previous(),
                KeyCode::Down => self.next(),
                KeyCode::Esc => {
                    return Ok(Some(Box::new(Menu::new())));
                }

                _ => {}
            };
            // match keys for the selected option
            match self.options[self.selected] {
                Options::Create => match key_code {
                    KeyCode::Enter => {
                        return Ok(Some(Box::new(CreateLobby::new())));
                    }
                    _ => {}
                },
                Options::Join => {
                    match key_code {
                        KeyCode::Left => {
                            self.join_lobby_input.move_left();
                        }
                        KeyCode::Right => {
                            self.join_lobby_input.move_right();
                        }
                        KeyCode::Char(c) => {
                            self.join_lobby_input.insert_char(c);
                        }
                        KeyCode::Backspace => {
                            self.join_lobby_input.delete_char();
                        }
                        KeyCode::Tab => {
                            if let Ok(mut ctx) = ClipboardContext::new() {
                                if let Ok(clipboard_content) = ctx.get_contents() {
                                    self.join_lobby_input.insert_clipboard(clipboard_content);
                                }
                            }
                        }
                        KeyCode::Enter => {
                            // TODO join the lobby
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(Some(Box::new(self.clone())))
    }
}

impl Render for CreateOrJoinLobby {
    fn render(&self, frame: &mut Frame) {
        let outer_rect = draw_outer_rectangle(
            frame,
            " quadropong ",
            vec![" Back ".into(), "<Esc> ".blue().bold()],
        );

        let inner_rect = draw_inner_rectangle(frame, outer_rect);

        let layout = Layout::vertical(vec![Constraint::Percentage(30), Constraint::Percentage(60)]);

        let [create_area, join_area] = layout.areas(inner_rect);

        // render create lobby area
        let create_area_text = if self.options[self.selected] == Options::Create {
            Line::from(format!("> {} <", Options::Create)).bold()
        } else {
            Line::from(Options::Create.to_string())
        };
        frame.render_widget(
            Paragraph::new(create_area_text).centered(),
            evenly_distanced_rects(create_area, 2)[1],
        );

        // render join lobby area
        let join_area_layout = Layout::horizontal(vec![
            Constraint::Percentage(15),
            Constraint::Percentage(70),
            Constraint::Percentage(15),
        ]);
        let [left_join, join_input_area, right_join] = join_area_layout.areas(join_area);
        let join_input_block = Block::bordered()
            .title(Options::Join.to_string())
            .title_bottom(" Join <Enter> | Paste <Tab> ");
        let inner_join_input_area = join_input_block.inner(join_input_area);
        let mut style = Style::default();
        if self.options[self.selected] == Options::Join {
            render_text_in_center_of_rect(frame, Paragraph::new(">").bold(), left_join);
            render_text_in_center_of_rect(frame, Paragraph::new("<").bold(), right_join);
            frame.set_cursor_position(Position::new(
                inner_join_input_area.x + self.join_lobby_input.char_index as u16, // TODO the `as` seems sus here
                inner_join_input_area.y,
            ));
            style = Style::default().bold();
        }
        frame.render_widget(join_input_block.style(style), join_input_area);
        frame.render_widget(
            Paragraph::new(self.join_lobby_input.input.clone()),
            inner_join_input_area,
        );
    }
}
