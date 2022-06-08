use crossterm::{
    cursor::{Hide, Show},
    event::{read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
    },
    Result,
};
use std::io::{stdout, Write};
use tui_input::backend::crossterm as backend;
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

        match event {
            Event::Key(KeyEvent { code, .. }) => match code {
                KeyCode::Esc | KeyCode::Enter => {
                    break;
                }
                _ => {
                    if backend::to_input_request(event)
                        .and_then(|r| input.handle(r))
                        .is_some()
                    {
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
            },
            _ => {}
        }
    }

    execute!(stdout, Show, LeaveAlternateScreen, DisableMouseCapture)?;
    disable_raw_mode()?;
    println!("{}", input);
    Ok(())
}
