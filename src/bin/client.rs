use std::{error::Error, io, time::Duration};

use crossterm::event::poll;
use ratatui::{
    backend::{Backend, CrosstermBackend},
    crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    Terminal,
};

use quadropong::{
    models::{
        client_input::{ClientInput, ClientInputType, Direction},
        game::Game,
        player::PlayerPosition,
    },
    ui::{app::*, ui::*},
};

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stderr: io::Stderr = io::stderr();
    execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let mut app = App::new();
    run_app(&mut terminal, &mut app)?;

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<bool> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if poll(Duration::ZERO)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Release {
                    continue;
                }
                match &mut app.current_screen {
                    CurrentScreen::MenuScreen(menu_state) => match key.code {
                        KeyCode::Down => {
                            *menu_state = menu_state.next();
                        }
                        KeyCode::Up => {
                            *menu_state = menu_state.previous();
                        }
                        KeyCode::Enter => {
                            // terminal.current_buffer_mut();
                            match menu_state {
                                MenuOptions::Online => {
                                    app.current_screen =
                                        CurrentScreen::OnlineScreen(OnlineOptions::Create)
                                }
                                MenuOptions::Training => {
                                    app.current_screen = CurrentScreen::TrainingCreateScreen
                                }
                                MenuOptions::Settings => {
                                    app.current_screen = CurrentScreen::SettingsScreen
                                }
                            }
                        }
                        KeyCode::Char('q') => {
                            return Ok(true);
                        }
                        _ => {}
                    },

                    CurrentScreen::OnlineScreen(online_state) => match key.code {
                        KeyCode::Down => {
                            *online_state = online_state.clone().next();
                        }
                        KeyCode::Up => {
                            *online_state = online_state.clone().previous();
                        }
                        KeyCode::Enter => {
                            match online_state {
                                OnlineOptions::Create => {
                                    app.current_screen = CurrentScreen::OnlineCreateScreen
                                }
                                OnlineOptions::Join => {
                                    // toggle pop up
                                    *online_state = OnlineOptions::EnterCode(Input::new());
                                }
                                OnlineOptions::EnterCode(input) => {
                                    // TODO try to join lobby
                                    // on success
                                    app.current_screen = CurrentScreen::OnlineLobbyScreen
                                    // on failure
                                    // TODO
                                }
                            }
                        }
                        KeyCode::Esc => match online_state {
                            OnlineOptions::EnterCode(_input) => *online_state = OnlineOptions::Join,
                            _ => {
                                app.current_screen = CurrentScreen::MenuScreen(MenuOptions::Online)
                            }
                        },
                        KeyCode::Char('q') => {
                            return Ok(true);
                        }
                        _ => {}
                    },

                    CurrentScreen::OnlineCreateScreen => match key.code {
                        KeyCode::Char('q') => {
                            return Ok(true);
                        }
                        KeyCode::Esc => {
                            app.current_screen = CurrentScreen::OnlineScreen(OnlineOptions::Create);
                        }
                        _ => {}
                    },

                    CurrentScreen::OnlineLobbyScreen => match key.code {
                        KeyCode::Char('q') => {
                            return Ok(true);
                        }
                        KeyCode::Esc => {
                            app.current_screen = CurrentScreen::OnlineScreen(OnlineOptions::Join);
                        }
                        _ => {}
                    },

                    CurrentScreen::TrainingCreateScreen => match key.code {
                        KeyCode::Char('q') => {
                            return Ok(true);
                        }
                        KeyCode::Esc => {
                            app.current_screen = CurrentScreen::MenuScreen(MenuOptions::Training);
                        }
                        _ => {}
                    },

                    CurrentScreen::SettingsScreen => match key.code {
                        KeyCode::Char('q') => {
                            return Ok(true);
                        }
                        KeyCode::Esc => {
                            app.current_screen = CurrentScreen::MenuScreen(MenuOptions::Settings);
                        }
                        _ => {}
                    },
                    CurrentScreen::GameScreen(game_info) => {
                        // TODO handle game input
                        if let Some(this_player) = game_info.game.players.get(&game_info.player_id)
                        {
                            let direction = match this_player.position {
                                Some(PlayerPosition::Bottom) | Some(PlayerPosition::Top) => {
                                    match key.code {
                                        KeyCode::Left | KeyCode::Char('a') => {
                                            Some(Direction::Negative)
                                        }
                                        KeyCode::Right | KeyCode::Char('d') => {
                                            Some(Direction::Positive)
                                        }
                                        _ => None,
                                    }
                                }
                                Some(PlayerPosition::Left) | Some(PlayerPosition::Right) => {
                                    match key.code {
                                        KeyCode::Up | KeyCode::Char('w') => {
                                            Some(Direction::Negative)
                                        }
                                        KeyCode::Down | KeyCode::Char('s') => {
                                            Some(Direction::Positive)
                                        }
                                        _ => None,
                                    }
                                }
                                _ => None,
                            };

                            // TODO this and the following block should be run async and continuously
                            if let Some(direction) = direction {
                                let input = ClientInput {
                                    game_id: game_info.game.id.to_string(),
                                    player_id: game_info.player_id.to_string(),
                                    action: ClientInputType::MovePaddle(direction),
                                };
                                let input_data: Vec<u8> = rmp_serde::to_vec(&input).unwrap();
                                let _ = game_info.udp_client.send(&input_data); // TODO should we just ignore it here?
                            }

                            // TODO this should be run async and continuously
                            let mut buf = vec![0; 1024];
                            match game_info.udp_client.recv_from(&mut buf) {
                                Ok((number_of_bytes, addr)) => {
                                    if number_of_bytes > 0 && addr == game_info.server_addr {
                                        let data = &buf[..number_of_bytes];
                                        if let Ok(updated_game) =
                                            rmp_serde::from_slice::<Game>(data)
                                        {
                                            game_info.game = updated_game;
                                        }
                                    }
                                }
                                _ => {} // TODO should we do something here?
                            }
                        }
                    }
                }
            }
        }
    }
}
