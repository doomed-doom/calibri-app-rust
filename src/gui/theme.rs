use iced::Theme;
use std::env;
use std::path::PathBuf;

pub fn detect_theme() -> Theme {
    if let Some(theme) = from_env() {
        return theme;
    }

    for folder in ["gtk-4.0", "gtk-3.0"] {
        if let Some(theme) = from_gtk_settings(folder) {
            return theme;
        }
    }

    Theme::Dark
}

fn from_env() -> Option<Theme> {
    if let Ok(value) = env::var("GTK_THEME") {
        let lower = value.to_lowercase();
        if lower.contains("dark") {
            return Some(Theme::Dark);
        } else if lower.contains("light") {
            return Some(Theme::Light);
        }
    }

    if let Ok(value) = env::var("COLORTERM") {
        if value.to_lowercase().contains("truecolor") {
            // не даёт точного ответа, пропускаем
        }
    }

    env::var("PREFER_DARK_THEME").ok().and_then(|value| {
        parse_bool(&value).map(|is_dark| if is_dark { Theme::Dark } else { Theme::Light })
    })
}

fn from_gtk_settings(folder: &str) -> Option<Theme> {
    let mut path = PathBuf::from(env::var_os("HOME")?);
    path.push(".config");
    path.push(folder);
    path.push("settings.ini");

    let contents = std::fs::read_to_string(path).ok()?;
    for line in contents.lines() {
        let line = line.trim();
        if !line.starts_with("gtk-application-prefer-dark-theme") {
            continue;
        }

        let value = line.split('=').nth(1)?;
        return parse_bool(value).map(|dark| if dark { Theme::Dark } else { Theme::Light });
    }
    None
}

fn parse_bool(value: &str) -> Option<bool> {
    match value.trim() {
        "1" | "true" | "True" | "TRUE" => Some(true),
        "0" | "false" | "False" | "FALSE" => Some(false),
        _ => None,
    }
}
