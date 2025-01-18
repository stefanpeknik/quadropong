use crossterm::event::poll;
use std::{
    io::{self, Stderr},
    sync::{atomic::AtomicBool, Arc},
    time::Duration,
};
use tokio::{self, sync::Mutex, task};

use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    Terminal,
};

use quadropong::ui2::states::{menu::Menu, traits::State};

fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stderr>>, io::Error> {
    enable_raw_mode()?;
    let mut stderr = io::stderr();
    execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stderr);
    let terminal = Terminal::new(backend)?;

    Ok(terminal)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stderr>>) -> Result<(), io::Error> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

struct App {
    current_state: Arc<Mutex<Box<dyn State>>>,
}

impl App {
    fn new() -> Self {
        Self {
            current_state: Arc::new(Mutex::new(Box::new(Menu::new()))),
        }
    }

    async fn run(&mut self) -> Result<(), io::Error> {
        // Shared flag to stop the tasks
        let running = Arc::new(AtomicBool::new(true));

        // Clone the shared state and the running flag for the render task
        let render_state = Arc::clone(&self.current_state);
        let render_running = Arc::clone(&running);
        let render_handle: tokio::task::JoinHandle<Result<(), io::Error>> =
            task::spawn(async move {
                let mut terminal = setup_terminal()?;
                let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(16));
                // Render loop
                while render_running.load(std::sync::atomic::Ordering::Relaxed) {
                    interval.tick().await;
                    let current_state = render_state.lock().await;
                    terminal.draw(|f| current_state.render(f))?;
                }
                restore_terminal(&mut terminal)?;

                Ok(())
            });

        // Clone the shared state and the running flag for the task that updates the state
        let update_state = Arc::clone(&self.current_state);
        let update_running = Arc::clone(&running);
        let update_handle: tokio::task::JoinHandle<Result<(), io::Error>> =
            task::spawn(async move {
                let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(5));
                // Update loop
                while update_running.load(std::sync::atomic::Ordering::Relaxed) {
                    interval.tick().await;
                    // Poll for user input
                    let input = if poll(Duration::ZERO)? {
                        if let Event::Key(key) = event::read()? {
                            if key.kind == event::KeyEventKind::Release {
                                continue;
                            } else {
                                Some(key.code)
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    let mut current_state = update_state.lock().await;
                    // Update the state
                    match current_state.update(input) {
                        Ok(Some(new_state)) => {
                            // Update the state if a new state is returned
                            *current_state = new_state;
                        }
                        Ok(None) => {
                            // Stop the tasks if the state returns None, which means the user wants to exit
                            update_running.store(false, std::sync::atomic::Ordering::Relaxed);
                        }
                        Err(e) => {
                            return Err(e);
                        }
                    }
                }

                Ok(())
            });

        // Wait for the tasks to finish
        update_handle.await??;
        render_handle.await??;

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), io::Error> {
    let mut app = App::new();
    app.run().await?;

    Ok(())
}
