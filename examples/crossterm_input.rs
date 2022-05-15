use crossterm::{
    cursor::{Hide, Show},
    event::{
        read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent,
    },
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
    Result,
};
use std::io::{stdout, Write};
use tui_input::backend::crossterm as backend;
use tui_input::{Input, InputResponse};

fn main() -> Result<()> {
    enable_raw_mode()?;
    let stdout = stdout();
    let mut stdout = stdout.lock();
    execute!(stdout, Hide, EnterAlternateScreen, EnableMouseCapture)?;

    let value = "Hello ".to_string();
    let mut input = Input::default().with_value(value);
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
                    if let Some(resp) = backend::to_input_request(event)
                        .map(|r| input.handle(r))
                    {
                        match resp {
                            InputResponse::Unchanged => {}
                            InputResponse::StateChanged { .. } => {
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
            },
            _ => {}
        }
    }

    execute!(stdout, Show, LeaveAlternateScreen, DisableMouseCapture)?;
    disable_raw_mode()?;
    println!("{}", input.value());
    Ok(())
}
