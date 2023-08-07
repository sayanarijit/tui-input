use crossterm::{
    cursor::{Hide, Show},
    event::{read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
    },
};
use std::io::{stdout, Result, Write};
use tui_input::backend::crossterm as backend;
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

fn main() -> Result<()> {
    enable_raw_mode()?;
    let stdout = stdout();
    let mut stdout = stdout.lock();
    execute!(stdout, Hide, EnterAlternateScreen, EnableMouseCapture)?;

    let mut input: Input = "Hello ".into();
    backend::write(&mut stdout, input.value(), input.cursor(), (0, 0), 15)?;
    stdout.flush()?;

    loop {
        let event = read()?;

        if let Event::Key(KeyEvent { code, .. }) = event {
            match code {
                KeyCode::Esc | KeyCode::Enter => {
                    break;
                }
                _ => {
                    if input.handle_event(&event).is_some() {
                        backend::write(
                            &mut stdout,
                            input.value(),
                            input.cursor(),
                            (0, 0),
                            15,
                        )?;
                        stdout.flush()?;
                    }
                }
            }
        }
    }

    execute!(stdout, Show, LeaveAlternateScreen, DisableMouseCapture)?;
    disable_raw_mode()?;
    println!("{}", input);
    Ok(())
}
