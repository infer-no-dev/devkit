//! Theme system for customizable UI appearance.

use ratatui::style::{Color, Modifier, Style};
use std::collections::HashMap;

/// Theme configuration for the UI
#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub colors: ColorScheme,
    pub styles: StyleScheme,
}

/// Color scheme for the theme
#[derive(Debug, Clone)]
pub struct ColorScheme {
    pub background: Color,
    pub foreground: Color,
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    pub error: Color,
    pub warning: Color,
    pub info: Color,
    pub success: Color,
    pub muted: Color,
    pub border: Color,
    pub selection: Color,
}

/// Style scheme for different UI elements
#[derive(Debug, Clone)]
pub struct StyleScheme {
    pub base: Style,
    pub header: Style,
    pub footer: Style,
    pub border: Style,
    pub selection: Style,
    pub timestamp: Style,
    pub user_input: Style,
    pub agent_response: Style,
    pub command: Style,
    pub error: Style,
    pub warning: Style,
    pub info: Style,
    pub success: Style,
    pub code: Style,
    pub metadata: Style,
}

/// Predefined theme variants
#[derive(Debug, Clone, PartialEq)]
pub enum ThemeVariant {
    Dark,
    Light,
    HighContrast,
    Matrix,
    Solarized,
    Monokai,
    Custom(String),
}

impl Theme {
    /// Create a new theme
    pub fn new(name: String, colors: ColorScheme, styles: StyleScheme) -> Self {
        Self {
            name,
            colors,
            styles,
        }
    }

    /// Create the default dark theme
    pub fn dark() -> Self {
        let colors = ColorScheme {
            background: Color::Black,
            foreground: Color::White,
            primary: Color::Cyan,
            secondary: Color::Blue,
            accent: Color::Magenta,
            error: Color::Red,
            warning: Color::Yellow,
            info: Color::Blue,
            success: Color::Green,
            muted: Color::DarkGray,
            border: Color::Gray,
            selection: Color::DarkGray,
        };

        let styles = StyleScheme::from_colors(&colors);

        Self::new("Dark".to_string(), colors, styles)
    }

    /// Create a light theme
    pub fn light() -> Self {
        let colors = ColorScheme {
            background: Color::White,
            foreground: Color::Black,
            primary: Color::Blue,
            secondary: Color::DarkGray,
            accent: Color::Magenta,
            error: Color::Red,
            warning: Color::Rgb(255, 140, 0), // Dark orange
            info: Color::Blue,
            success: Color::Green,
            muted: Color::Gray,
            border: Color::DarkGray,
            selection: Color::LightBlue,
        };

        let styles = StyleScheme::from_colors(&colors);

        Self::new("Light".to_string(), colors, styles)
    }

    /// Create a high contrast theme
    pub fn high_contrast() -> Self {
        let colors = ColorScheme {
            background: Color::Black,
            foreground: Color::White,
            primary: Color::White,
            secondary: Color::White,
            accent: Color::Yellow,
            error: Color::Red,
            warning: Color::Yellow,
            info: Color::Cyan,
            success: Color::Green,
            muted: Color::Gray,
            border: Color::White,
            selection: Color::White,
        };

        let styles = StyleScheme::from_colors(&colors);

        Self::new("HighContrast".to_string(), colors, styles)
    }

    /// Create a matrix-style theme
    pub fn matrix() -> Self {
        let colors = ColorScheme {
            background: Color::Black,
            foreground: Color::Green,
            primary: Color::Green,
            secondary: Color::Green,
            accent: Color::LightGreen,
            error: Color::Red,
            warning: Color::Yellow,
            info: Color::Cyan,
            success: Color::Green,
            muted: Color::Green,
            border: Color::Green,
            selection: Color::Green,
        };

        let styles = StyleScheme::from_colors(&colors);

        Self::new("Matrix".to_string(), colors, styles)
    }

    /// Get theme by variant
    pub fn from_variant(variant: ThemeVariant) -> Self {
        match variant {
            ThemeVariant::Dark => Self::dark(),
            ThemeVariant::Light => Self::light(),
            ThemeVariant::HighContrast => Self::high_contrast(),
            ThemeVariant::Matrix => Self::matrix(),
            ThemeVariant::Solarized => Self::solarized(),
            ThemeVariant::Monokai => Self::monokai(),
            ThemeVariant::Custom(_name) => {
                // In a real implementation, this would load from file
                Self::dark() // Fallback
            }
        }
    }

    /// Create solarized theme
    pub fn solarized() -> Self {
        let colors = ColorScheme {
            background: Color::Rgb(0, 43, 54),     // base03
            foreground: Color::Rgb(131, 148, 150), // base0
            primary: Color::Rgb(38, 139, 210),     // blue
            secondary: Color::Rgb(42, 161, 152),   // cyan
            accent: Color::Rgb(211, 54, 130),      // magenta
            error: Color::Rgb(220, 50, 47),        // red
            warning: Color::Rgb(181, 137, 0),      // yellow
            info: Color::Rgb(38, 139, 210),        // blue
            success: Color::Rgb(133, 153, 0),      // green
            muted: Color::Rgb(88, 110, 117),       // base01
            border: Color::Rgb(88, 110, 117),      // base01
            selection: Color::Rgb(7, 54, 66),      // base02
        };

        let styles = StyleScheme::from_colors(&colors);

        Self::new("Solarized".to_string(), colors, styles)
    }

    /// Create monokai theme
    pub fn monokai() -> Self {
        let colors = ColorScheme {
            background: Color::Rgb(39, 40, 34),    // Dark background
            foreground: Color::Rgb(248, 248, 242), // Light foreground
            primary: Color::Rgb(102, 217, 239),    // Cyan
            secondary: Color::Rgb(166, 226, 46),   // Green
            accent: Color::Rgb(249, 38, 114),      // Pink
            error: Color::Rgb(249, 38, 114),       // Pink
            warning: Color::Rgb(230, 219, 116),    // Yellow
            info: Color::Rgb(102, 217, 239),       // Cyan
            success: Color::Rgb(166, 226, 46),     // Green
            muted: Color::Rgb(117, 113, 94),       // Comment gray
            border: Color::Rgb(73, 72, 62),        // Dark gray
            selection: Color::Rgb(73, 72, 62),     // Dark gray
        };

        let styles = StyleScheme::from_colors(&colors);

        Self::new("Monokai".to_string(), colors, styles)
    }

    // Style accessors for different UI elements
    pub fn base_style(&self) -> Style {
        self.styles.base
    }

    pub fn status_bar_style(&self) -> Style {
        Style::default()
            .bg(self.colors.primary)
            .fg(self.colors.background)
            .add_modifier(Modifier::BOLD)
    }

    pub fn output_area_style(&self) -> Style {
        Style::default()
            .bg(self.colors.background)
            .fg(self.colors.foreground)
    }

    pub fn timestamp_style(&self) -> Style {
        self.styles.timestamp
    }

    pub fn user_input_style(&self) -> Style {
        self.styles.user_input
    }

    pub fn user_input_content_style(&self) -> Style {
        Style::default()
            .fg(self.colors.foreground)
            .add_modifier(Modifier::ITALIC)
    }

    pub fn agent_response_style(&self) -> Style {
        self.styles.agent_response
    }

    pub fn agent_response_content_style(&self) -> Style {
        Style::default().fg(self.colors.foreground)
    }

    pub fn command_style(&self) -> Style {
        self.styles.command
    }

    pub fn command_content_style(&self) -> Style {
        Style::default()
            .fg(self.colors.secondary)
            .add_modifier(Modifier::BOLD)
    }

    pub fn command_output_style(&self) -> Style {
        Style::default().fg(self.colors.muted)
    }

    pub fn command_output_content_style(&self) -> Style {
        Style::default().fg(self.colors.muted)
    }

    pub fn error_style(&self) -> Style {
        self.styles.error
    }

    pub fn error_content_style(&self) -> Style {
        Style::default()
            .fg(self.colors.error)
            .add_modifier(Modifier::BOLD)
    }

    pub fn warning_style(&self) -> Style {
        self.styles.warning
    }

    pub fn warning_content_style(&self) -> Style {
        Style::default().fg(self.colors.warning)
    }

    pub fn info_style(&self) -> Style {
        self.styles.info
    }

    pub fn info_content_style(&self) -> Style {
        Style::default().fg(self.colors.info)
    }

    pub fn success_style(&self) -> Style {
        self.styles.success
    }

    pub fn success_content_style(&self) -> Style {
        Style::default()
            .fg(self.colors.success)
            .add_modifier(Modifier::BOLD)
    }

    pub fn code_generation_style(&self) -> Style {
        Style::default().fg(self.colors.accent)
    }

    pub fn code_generation_content_style(&self) -> Style {
        Style::default()
            .fg(self.colors.foreground)
            .bg(self.colors.selection)
    }

    pub fn analysis_style(&self) -> Style {
        Style::default().fg(self.colors.info)
    }

    pub fn analysis_content_style(&self) -> Style {
        Style::default().fg(self.colors.info)
    }

    pub fn notification_style(&self) -> Style {
        Style::default().fg(self.colors.warning)
    }

    pub fn notification_content_style(&self) -> Style {
        Style::default().fg(self.colors.warning)
    }

    pub fn system_style(&self) -> Style {
        Style::default().fg(self.colors.muted)
    }

    pub fn system_content_style(&self) -> Style {
        Style::default().fg(self.colors.muted)
    }

    pub fn agent_name_style(&self) -> Style {
        Style::default()
            .fg(self.colors.accent)
            .add_modifier(Modifier::BOLD)
    }

    pub fn metadata_key_style(&self) -> Style {
        Style::default()
            .fg(self.colors.muted)
            .add_modifier(Modifier::ITALIC)
    }

    pub fn metadata_value_style(&self) -> Style {
        Style::default().fg(self.colors.secondary)
    }

    pub fn border_style(&self) -> Style {
        self.styles.border
    }

    pub fn selection_style(&self) -> Style {
        self.styles.selection
    }

    pub fn input_style(&self) -> Style {
        Style::default()
            .fg(self.colors.foreground)
            .bg(self.colors.selection)
    }

    pub fn input_cursor_style(&self) -> Style {
        Style::default()
            .fg(self.colors.background)
            .bg(self.colors.primary)
            .add_modifier(Modifier::BOLD)
    }

    // Additional methods for panel compatibility
    pub fn secondary_style(&self) -> Style {
        Style::default().fg(self.colors.secondary)
    }

    pub fn primary_style(&self) -> Style {
        Style::default().fg(self.colors.primary)
    }

    pub fn label_style(&self) -> Style {
        Style::default()
            .fg(self.colors.muted)
            .add_modifier(Modifier::ITALIC)
    }

    pub fn value_style(&self) -> Style {
        Style::default().fg(self.colors.foreground)
    }

    pub fn muted_style(&self) -> Style {
        Style::default().fg(self.colors.muted)
    }

    /// Get border style for focused panels
    pub fn focused_border_style(&self) -> Style {
        Style::default()
            .fg(self.colors.accent)
            .add_modifier(Modifier::BOLD)
    }

    /// Get border style for unfocused panels
    pub fn unfocused_border_style(&self) -> Style {
        self.border_style()
    }

    /// Get border style based on focus state
    pub fn panel_border_style(&self, is_focused: bool) -> Style {
        if is_focused {
            self.focused_border_style()
        } else {
            self.unfocused_border_style()
        }
    }
}

impl StyleScheme {
    /// Create styles from a color scheme
    pub fn from_colors(colors: &ColorScheme) -> Self {
        Self {
            base: Style::default().bg(colors.background).fg(colors.foreground),
            header: Style::default()
                .bg(colors.primary)
                .fg(colors.background)
                .add_modifier(Modifier::BOLD),
            footer: Style::default().bg(colors.secondary).fg(colors.background),
            border: Style::default().fg(colors.border),
            selection: Style::default().bg(colors.selection).fg(colors.foreground),
            timestamp: Style::default()
                .fg(colors.muted)
                .add_modifier(Modifier::DIM),
            user_input: Style::default()
                .fg(colors.primary)
                .add_modifier(Modifier::BOLD),
            agent_response: Style::default()
                .fg(colors.accent)
                .add_modifier(Modifier::BOLD),
            command: Style::default()
                .fg(colors.secondary)
                .add_modifier(Modifier::BOLD),
            error: Style::default()
                .fg(colors.error)
                .add_modifier(Modifier::BOLD),
            warning: Style::default()
                .fg(colors.warning)
                .add_modifier(Modifier::BOLD),
            info: Style::default().fg(colors.info),
            success: Style::default()
                .fg(colors.success)
                .add_modifier(Modifier::BOLD),
            code: Style::default().fg(colors.foreground).bg(colors.selection),
            metadata: Style::default()
                .fg(colors.muted)
                .add_modifier(Modifier::ITALIC),
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}

/// Theme manager for loading and managing themes
#[derive(Debug)]
pub struct ThemeManager {
    themes: HashMap<String, Theme>,
    current_theme: String,
}

impl ThemeManager {
    /// Create a new theme manager
    pub fn new() -> Self {
        let mut themes = HashMap::new();

        // Add built-in themes
        let dark = Theme::dark();
        let light = Theme::light();
        let high_contrast = Theme::high_contrast();
        let matrix = Theme::matrix();
        let solarized = Theme::solarized();
        let monokai = Theme::monokai();

        themes.insert(dark.name.clone(), dark);
        themes.insert(light.name.clone(), light);
        themes.insert(high_contrast.name.clone(), high_contrast);
        themes.insert(matrix.name.clone(), matrix);
        themes.insert(solarized.name.clone(), solarized);
        themes.insert(monokai.name.clone(), monokai);

        Self {
            themes,
            current_theme: "Dark".to_string(),
        }
    }

    /// Get the current theme
    pub fn current_theme(&self) -> &Theme {
        self.themes
            .get(&self.current_theme)
            .unwrap_or_else(|| &self.themes["Dark"])
    }

    /// Set the current theme
    pub fn set_theme(&mut self, name: &str) -> bool {
        if self.themes.contains_key(name) {
            self.current_theme = name.to_string();
            true
        } else {
            false
        }
    }

    /// Get available theme names
    pub fn available_themes(&self) -> Vec<String> {
        self.themes.keys().cloned().collect()
    }

    /// Add a custom theme
    pub fn add_theme(&mut self, theme: Theme) {
        self.themes.insert(theme.name.clone(), theme);
    }

    /// Remove a theme
    pub fn remove_theme(&mut self, name: &str) -> bool {
        if name != "Dark" && name != "Light" {
            // Protect built-in themes
            self.themes.remove(name).is_some()
        } else {
            false
        }
    }

    /// Cycle to the next available theme
    pub fn cycle_theme(&mut self) {
        let themes: Vec<String> = self.available_themes();
        if themes.len() <= 1 {
            return;
        }

        if let Some(current_index) = themes.iter().position(|name| name == &self.current_theme) {
            let next_index = (current_index + 1) % themes.len();
            self.current_theme = themes[next_index].clone();
        } else {
            // Fallback to first theme if current theme is not found
            self.current_theme = themes[0].clone();
        }
    }
}
