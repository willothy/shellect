use anyhow::Result;
use crossterm::{
    cursor::{Hide, MoveDown, MoveTo, MoveToColumn, MoveUp, Show},
    event::{read, Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers},
    execute, queue,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use hex_color::HexColor;
use serde::Deserialize;
use std::{io::Write, os::unix::process::CommandExt, path::PathBuf, process::Command};

#[derive(Debug, Deserialize)]
pub struct Shell {
    name: String,
    path: PathBuf,
    args: Vec<String>,
    #[serde(with = "hex_color::rgb")]
    color: HexColor,
    default: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct SColor {
    #[serde(with = "hex_color::rgb")]
    color: HexColor,
}

fn make_opt<'a, T: Deserialize<'a>>(x: T) -> Option<T> {
    Some(x)
}

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(alias = "shell")]
    shells: Vec<Shell>,
}

fn run(config: &Config) -> Result<Option<&Shell>> {
    let mut stdout = std::io::stdout().lock();
    execute!(stdout, EnterAlternateScreen, Show, MoveTo(0, 0))?;
    enable_raw_mode()?;

    let mut selected = config
        .shells
        .iter()
        .position(|shell| shell.default.is_some() && shell.default.unwrap())
        .unwrap_or(0);

    let mut render_options = |selected: usize| -> Result<()> {
        queue!(stdout, MoveTo(0, 0), Clear(ClearType::FromCursorDown))?;
        for (idx, shell) in config.shells.iter().enumerate() {
            let color = if let Some(color) = &shell.color {
                Color::Rgb {
                    r: color.r,
                    g: color.g,
                    b: color.b,
                }
            } else {
                Color::White
            };
            if idx == selected {
                queue!(
                    stdout,
                    SetForegroundColor(color),
                    Print(format!("{:<2}> {} <", idx, shell.name)),
                    ResetColor
                )?;
            } else {
                queue!(
                    stdout,
                    SetForegroundColor(color),
                    Print(format!("{:<2}  {}", idx, shell.name)),
                    ResetColor
                )?;
            }
            queue!(stdout, MoveDown(1), MoveToColumn(0))?;
        }
        stdout.flush()?;
        Ok(())
    };

    let result = loop {
        render_options(selected)?;
        match read()? {
            Event::Key(key) => match key {
                KeyEvent {
                    code: KeyCode::Char('c'),
                    modifiers: KeyModifiers::CONTROL,
                    kind: KeyEventKind::Press,
                    ..
                } => break None,
                KeyEvent {
                    code: KeyCode::Enter,
                    modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
                    kind: KeyEventKind::Press,
                    ..
                } => break Some(selected),
                KeyEvent {
                    code: KeyCode::Up,
                    modifiers: KeyModifiers::NONE,
                    kind: KeyEventKind::Press,
                    ..
                } => selected = selected.checked_sub(1).unwrap_or(0),
                KeyEvent {
                    code: KeyCode::Down,
                    modifiers: KeyModifiers::NONE,
                    kind: KeyEventKind::Press,
                    ..
                } => {
                    selected = if selected + 1 >= config.shells.len() {
                        config.shells.len() - 1
                    } else {
                        selected + 1
                    }
                }
                _ => {}
            },
            _ => {}
        }
    };

    execute!(stdout, Show, LeaveAlternateScreen)?;

    disable_raw_mode()?;

    if let Some(selected) = result {
        Ok(Some(&config.shells[selected]))
    } else {
        Ok(None)
    }
}

fn main() -> Result<()> {
    let cfg_path = PathBuf::from(std::env::var("HOME")?).join(".shellect.toml");
    let cfg = std::fs::read_to_string(&cfg_path)?;

    let config: Config = toml::from_str(&cfg)?;

    match run(&config) {
        Ok(Some(shell)) => {
            let mut cmd = Command::new(&shell.path);
            cmd.args(&shell.args);

            cmd.exec();
        }
        Ok(None) => {}
        Err(e) => {
            execute!(std::io::stdout(), Show, LeaveAlternateScreen)?;
            disable_raw_mode()?;
            eprintln!("{}", e);
        }
    }

    Ok(())
}
