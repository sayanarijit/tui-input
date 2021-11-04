use std::io::{stdin, stdout, Result, Write};
use termion::cursor::{Hide, Show};
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui_input::backend::termion as backend;
use tui_input::{Input, InputResponse};

fn main() -> Result<()> {
    let stdin = stdin();
    let stdout = stdout().into_raw_mode().unwrap();
    let mut stdout = AlternateScreen::from(stdout);

    let value = "Hello ".to_string();
    let mut input = Input::default().with_cursor(value.len()).with_value(value);
    write!(&mut stdout, "{}", Hide)?;
    backend::write(&mut stdout, input.value(), input.cursor(), (0, 0), 15)?;
    stdout.flush()?;

    for evt in stdin.events() {
        let evt = evt?;
        if let Some(resp) =
            backend::to_input_request(&evt).and_then(|req| input.handle(req))
        {
            match resp {
                InputResponse::Escaped | InputResponse::Submitted => {
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

    write!(stdout, "{}", Show)?;
    Ok(())
}
