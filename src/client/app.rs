use crossterm::event::{Event, EventStream, KeyEvent};
use futures_util::TryStreamExt;
use ratatui::{prelude::Backend, Terminal};
use std::{sync::Arc, thread::sleep, time::Duration};
use tokio::{self, sync::Mutex, task, time::Instant};
use tokio_util::sync::CancellationToken;

use super::{
    config::Config,
    error::ClientError,
    states::{menu::Menu, quit::Quit, traits::State},
};

pub struct App<'a, B: Backend> {
    current_state: Arc<Mutex<Box<dyn State>>>,
    config: Arc<Mutex<Config>>,
    cancellation_token: CancellationToken,
    terminal: &'a mut Terminal<B>,
}

impl<'a, B: Backend> App<'a, B> {
    pub fn new(terminal: &'a mut Terminal<B>, config: Config) -> Result<Self, ClientError> {
        Ok(Self {
            current_state: Arc::new(Mutex::new(Box::new(Menu::new(0, config.clone())?))),
            config: Arc::new(Mutex::new(config)),
            cancellation_token: CancellationToken::new(),
            terminal,
        })
    }

    pub async fn run(&mut self) -> Result<(), ClientError> {
        // Clone the shared state and the running flag for the task that updates the state
        let update_state = Arc::clone(&self.current_state);
        let cancellation_token_clone = self.cancellation_token.clone();
        let update_settings = Arc::clone(&self.config);
        let update_handle = task::spawn(async move {
            let mut reader = EventStream::new();
            let mut last_key_event_time = Instant::now();
            let key_event_interval = Duration::from_millis(10);
            let mut last_key_event: Option<KeyEvent> = None;

            loop {
                let mut input = None;

                // Use select! to wait for either cancellation or input/timeout
                tokio::select! {
                    // Check for cancellation first
                    biased;
                    _ = cancellation_token_clone.cancelled() => break,

                    // Process incoming events
                    maybe_event = reader.try_next() => {
                        match maybe_event {
                            Ok(Some(Event::Key(key_event))) => {
                                let now = Instant::now();
                                let time_since_last = now.duration_since(last_key_event_time);

                                if Some(key_event) != last_key_event || time_since_last >= key_event_interval {
                                    last_key_event_time = now;
                                    last_key_event = Some(key_event);
                                    input = Some(key_event.code);
                                }
                            }
                            Err(e) => return Err(e.into()),
                            _ => {}
                        }
                    }

                    // Regular state updates (5ms timeout matches original poll interval)
                    _ = tokio::time::sleep(Duration::from_millis(5)) => {}
                }

                // Process state update with or without input
                let mut current_state = update_state.lock().await;
                match current_state.update(input).await {
                    Ok(Some(new_state)) => {
                        if new_state.as_any().downcast_ref::<Quit>().is_some() {
                            cancellation_token_clone.cancel();
                            break;
                        } else {
                            // Update settings before changing state
                            let mut settings = update_settings.lock().await;
                            *settings = current_state.config().clone();
                            *current_state = new_state;
                        }
                    }
                    Ok(None) => {}
                    Err(e) => return Err(e),
                }
            }

            Ok(())
        });

        // Main render loop
        loop {
            // Check for cancellation
            if self.cancellation_token.is_cancelled() {
                break;
            }

            // Lock the state and render (release the lock as soon as possible)
            {
                let current_state = self.current_state.lock().await;
                self.terminal.draw(|f| current_state.render(f))?;
            }
            {
                let fps = self.config.lock().await.fps;
                sleep(Duration::from_secs(1) / fps);
            }
        }

        // Wait for the update task to finish
        update_handle.await??;

        Ok(())
    }
}
