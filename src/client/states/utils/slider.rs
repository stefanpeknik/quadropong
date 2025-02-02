use ratatui::style::Color;

#[derive(Clone)]
pub struct Slider {
    pub all_colors: Vec<Color>,
    pub selected: usize,
}

impl Default for Slider {
    fn default() -> Self {
        Self::new()
    }
}

impl Slider {
    pub fn new() -> Self {
        Self {
            all_colors: Self::init_all_colors(),
            selected: 0,
        }
    }

    pub fn from(string: String) -> Self {
        let all_colors = Self::init_all_colors();
        let selected = all_colors
            .iter()
            .position(|color| color.to_string() == string)
            .unwrap_or(0);
        Self {
            all_colors,
            selected,
        }
    }

    pub fn get_text(self) -> String {
        self.all_colors[self.selected].to_string()
    }

    pub fn get_color(self) -> Color {
        self.all_colors[self.selected]
    }

    fn init_all_colors() -> Vec<Color> {
        vec![
            Color::Black,
            Color::Red,
            Color::Green,
            Color::Yellow,
            Color::Blue,
            Color::Magenta,
            Color::Cyan,
            Color::Gray,
            Color::DarkGray,
            Color::LightRed,
            Color::LightGreen,
            Color::LightYellow,
            Color::LightBlue,
            Color::LightMagenta,
            Color::LightCyan,
            Color::White,
        ]
    }

    pub fn next(&mut self) {
        self.selected = (self.selected + 1) % self.all_colors.len();
    }

    pub fn previous(&mut self) {
        if self.selected == 0 {
            self.selected = self.all_colors.len() - 1;
        } else {
            self.selected -= 1;
        }
    }
}
