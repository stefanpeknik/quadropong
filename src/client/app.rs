use crossterm::event::{poll, KeyEvent};
use std::{
    io::{self, Stderr},
    sync::{atomic::AtomicBool, Arc},
    time::Duration,
};
use tokio::{self, sync::Mutex, task, time::Instant};

use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    Terminal,
};

use super::states::{menu::Menu, quit::Quit, traits::State};

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

pub struct App {
    current_state: Arc<Mutex<Box<dyn State>>>,
}

impl App {
    pub fn new() -> Self {
        Self {
            current_state: Arc::new(Mutex::new(Box::new(Menu::new(0)))),
        }
    }

    pub async fn run(&mut self) -> Result<(), io::Error> {
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
                let mut last_key_event_time = Instant::now();
                let key_event_interval = Duration::from_millis(10);
                let mut last_key_event: Option<KeyEvent> = None;

                // Update loop
                while update_running.load(std::sync::atomic::Ordering::Relaxed) {
                    interval.tick().await;
                    // Poll for user input
                    let input = if poll(Duration::ZERO)? {
                        let now = Instant::now();
                        if let Event::Key(key_event) = event::read()? {
                            // Reduction for continuous key events
                            if Some(key_event) != last_key_event
                                || now.duration_since(last_key_event_time) >= key_event_interval
                            {
                                // Update the last key event time and the last key event
                                last_key_event_time = now;
                                last_key_event = Some(key_event);
                                Some(key_event.code) // Capture the pressed key
                            } else {
                                None
                            }
                        } else {
                            None // No input captured
                        }
                    } else {
                        None // No input captured
                    };

                    let mut current_state = update_state.lock().await;
                    // Update the state
                    match current_state.update(input).await {
                        Ok(Some(new_state)) => {
                            if new_state.as_any().downcast_ref::<Quit>().is_some() {
                                // We got a Quit state, stop the tasks
                                update_running.store(false, std::sync::atomic::Ordering::Relaxed);
                            } else {
                                // Move to the new state
                                *current_state = new_state;
                            }
                        }
                        Ok(None) => {
                            // Do nothing as the state is unchanged
                        }
                        Err(e) => {
                            return Err(e);
                        }
                    }
                }

                Ok(())
            });

        // Wait for the tasks to finish
        render_handle.await??;
        update_handle.await??;

        Ok(())
    }
}
