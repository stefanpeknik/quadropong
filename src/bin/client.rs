use std::{error::Error, io};

use ratatui::{
    backend::{Backend, CrosstermBackend},
    crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    Terminal,
};
use cli_clipboard::{ClipboardContext, ClipboardProvider};

use quadropong::ui::{app::*, ui::*};

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stderr = io::stderr();
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
                        if matches!(online_state, OnlineOptions::Create | OnlineOptions::Join) {
                            *online_state = online_state.clone().next();
                        }
                    }
                    KeyCode::Up => {
                        if matches!(online_state, OnlineOptions::Create | OnlineOptions::Join) {
                            *online_state = online_state.clone().previous();
                        }
                    }
                    KeyCode::Enter => {
                        match online_state {
                            OnlineOptions::Create => {
                                app.current_screen = CurrentScreen::OnlineCreateScreen;
                            }
                            OnlineOptions::Join => {
                                // Transition to EnterCode with a new Input instance
                                *online_state = OnlineOptions::EnterCode(Input::new());
                            }
                            OnlineOptions::EnterCode(input) => {
                                // Attempt to join lobby
                                // TODO: Add lobby joining logic here
                                // On success:
                                app.current_screen = CurrentScreen::OnlineLobbyScreen;
                                // On failure:
                                // TODO: Handle failure case
                            }
                        }
                    }
                    KeyCode::Left => {
                        if let OnlineOptions::EnterCode(input) = online_state {
                            input.move_left();
                        }
                    }
                    KeyCode::Right => {
                        if let OnlineOptions::EnterCode(input) = online_state {
                            input.move_right();
                        }
                    }
                    KeyCode::Backspace => {
                        if let OnlineOptions::EnterCode(input) = online_state {
                            input.delete_char();
                        }
                    }
                    KeyCode::Tab => {
                        if let OnlineOptions::EnterCode(input) = online_state {
                            if let Ok(mut ctx) = ClipboardContext::new() {
                                if let Ok(clipboard_content) = ctx.get_contents() {
                                    input.insert_clipboard(clipboard_content);
                                }
                            }
                        }
                    }
                    KeyCode::Char(char) => {
                        // Allow character input in EnterCode state
                        if let OnlineOptions::EnterCode(input) = online_state {
                            input.insert_char(char);
                        } else if char == 'q' {
                            return Ok(true);
                        }
                    }
                    KeyCode::Esc => {
                        match online_state {
                            OnlineOptions::EnterCode(_) => {
                                *online_state = OnlineOptions::Join;
                            }
                            _ => {
                                app.current_screen = CurrentScreen::MenuScreen(MenuOptions::Online);
                            }
                        }
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
            }
        }
    }
}
