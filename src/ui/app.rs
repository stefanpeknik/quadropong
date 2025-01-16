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

#[derive(Debug, Clone, Copy)]
pub enum OnlineOptions {
    Create,
    Join,
}

impl OnlineOptions {
    pub fn next(self) -> Self {
        match self {
            Self::Create => Self::Join,
            Self::Join => Self::Create,
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
    OnlineJoinScreen,
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
