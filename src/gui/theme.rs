use druid::{Color, Data, Key, Value, ValueType};

// Theme type
pub const THEME_TYPE: Key<ThemeType> = Key::new("ferrousdb.theme-type");

#[derive(Clone, Copy, PartialEq, Data)]
pub enum ThemeType {
    Light,
    Dark,
}

impl From<ThemeType> for Value {
    fn from(theme_type: ThemeType) -> Self {
        match theme_type {
            ThemeType::Light => Value::String("Light".into()),
            ThemeType::Dark => Value::String("Dark".into()),
        }
    }
}

impl From<Value> for ThemeType {
    fn from(value: Value) -> Self {
        match value {
            Value::String(s) => {
                if s.to_string().contains("Dark") {
                    ThemeType::Dark
                } else {
                    ThemeType::Light
                }
            }
            _ => ThemeType::Dark, // Default to dark theme
        }
    }
}

impl ValueType for ThemeType {
    fn try_from_value(value: &Value) -> Result<Self, druid::ValueTypeError> {
        match value {
            Value::String(s) => {
                if s.to_string().contains("Dark") {
                    Ok(ThemeType::Dark)
                } else {
                    Ok(ThemeType::Light)
                }
            }
            _ => Ok(ThemeType::Dark) // Default to dark theme
        }
    }
}

// Colors
pub const PRIMARY_COLOR: Key<Color> = Key::new("ferrousdb.primary-color");
pub const SECONDARY_COLOR: Key<Color> = Key::new("ferrousdb.secondary-color");
pub const BACKGROUND_COLOR: Key<Color> = Key::new("ferrousdb.background-color");
pub const TEXT_COLOR: Key<Color> = Key::new("ferrousdb.text-color");
pub const BUTTON_HOVER_COLOR: Key<Color> = Key::new("ferrousdb.button-hover-color");
pub const SURFACE_COLOR: Key<Color> = Key::new("ferrousdb.surface-color");

// Spacing
pub const PADDING_SMALL: f64 = 4.0;
pub const PADDING_MEDIUM: f64 = 8.0;
pub const PADDING_LARGE: f64 = 16.0;

// Sizes
pub const BUTTON_HEIGHT: f64 = 32.0;
pub const BUTTON_WIDTH: f64 = 120.0;
pub const TEXT_SIZE_SMALL: f64 = 12.0;
pub const TEXT_SIZE_MEDIUM: f64 = 14.0;
pub const TEXT_SIZE_LARGE: f64 = 16.0;

// Theme colors
fn get_dark_theme() -> [(Key<Color>, Color); 6] {
    [
        (PRIMARY_COLOR, Color::rgb8(0x64, 0xff, 0xda)),      // Ciano claro
        (SECONDARY_COLOR, Color::rgb8(0x00, 0xb8, 0xd4)),    // Ciano mais escuro
        (BACKGROUND_COLOR, Color::rgb8(0x1a, 0x1a, 0x1a)),   // Cinza muito escuro
        (TEXT_COLOR, Color::rgb8(0xe0, 0xe0, 0xe0)),         // Cinza claro
        (BUTTON_HOVER_COLOR, Color::rgb8(0x80, 0xff, 0xdb)), // Ciano mais claro
        (SURFACE_COLOR, Color::rgb8(0x2d, 0x2d, 0x2d)),      // Cinza escuro para superfícies
    ]
}

pub fn configure_env(env: &mut druid::Env) {
    // Set theme type
    env.set(THEME_TYPE, ThemeType::Dark);

    // Configure dark theme colors
    for (key, color) in get_dark_theme() {
        env.set(key, color);
    }

    // Default font
    env.set(
        druid::theme::UI_FONT,
        druid::FontDescriptor::new(druid::FontFamily::SYSTEM_UI).with_size(TEXT_SIZE_MEDIUM),
    );
}
