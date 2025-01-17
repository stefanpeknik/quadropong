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
                    KeyCode::Esc => {
                        match online_state {
                            OnlineOptions::EnterCode(_input) => *online_state = OnlineOptions::Join,
                            _ => app.current_screen = CurrentScreen::MenuScreen(MenuOptions::Online),
                        }
                    }
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
            }
        }
    }
}
