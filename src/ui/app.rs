use std::clone;

#[derive(Debug, Clone, Copy)]
pub enum MenuOptions {
    Online,
    Training,
    Settings,
}

impl MenuOptions {
    pub fn next(self) -> Self {
        match self {
            Self::Online => Self::Training,
            Self::Training => Self::Settings,
            Self::Settings => Self::Online,
        }
    }

    pub fn previous(self) -> Self {
        match self {
            Self::Online => Self::Settings,
            Self::Training => Self::Online,
            Self::Settings => Self::Training,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Input {
    pub input: String,
    char_index: usize,
}

impl Input {
    pub fn new() -> Self {
        Self { input: String::new(), char_index: 0 }
    }
}

#[derive(Debug, Clone)]
pub enum OnlineOptions {
    Create,
    Join,
    EnterCode(Input),
}

impl OnlineOptions {
    pub fn next(self) -> Self {
        match self {
            Self::Create => Self::Join,
            Self::Join => Self::Create,
            Self::EnterCode(input) => Self::EnterCode(input),
        }
    }

    pub fn previous(self) -> Self {
        self.next()
    }
}

pub enum CurrentScreen {
    MenuScreen(MenuOptions),
    OnlineScreen(OnlineOptions),
    OnlineCreateScreen,
    OnlineLobbyScreen,
    TrainingCreateScreen,
    SettingsScreen,
}

pub enum CurrentlyEditing {
    Key,
    Value,
}

pub struct App {
    pub current_screen: CurrentScreen,
}

impl App {
    pub fn new() -> App {
        App {
            current_screen: CurrentScreen::MenuScreen(MenuOptions::Online),
        }
    }
}

// Clippy wanted this
impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
