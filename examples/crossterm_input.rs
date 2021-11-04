use crossterm::{
    cursor::{Hide, Show},
    event::{read, DisableMouseCapture, EnableMouseCapture},
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
    let mut input = Input::default().with_cursor(value.len()).with_value(value);
    backend::write(&mut stdout, input.value(), input.cursor(), (0, 0), 15)?;
    stdout.flush()?;

    loop {
        let event = read()?;
        if let Some(resp) =
            backend::to_input_request(event).and_then(|r| input.handle(r))
        {
            match resp {
                InputResponse::Submitted | InputResponse::Escaped => {
                    break;
                }
                InputResponse::StateChanged(_) => {
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

    execute!(stdout, Show, LeaveAlternateScreen, DisableMouseCapture)?;
    disable_raw_mode()?;
    println!("{}", input.value());
    Ok(())
}
