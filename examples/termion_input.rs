use std::io::{stdin, stdout, Result, Write};
use termion::cursor::{Hide, Show};
use termion::event::{Event, Key};
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui_input::backend::termion as backend;
use tui_input::Input;

fn main() -> Result<()> {
    let mut value = "Hello ".to_string();
    {
        let stdin = stdin();
        let stdout = stdout().into_raw_mode().unwrap();
        let mut stdout = AlternateScreen::from(stdout);

        let mut input = Input::default().with_value(value);
        write!(&mut stdout, "{}", Hide)?;
        backend::write(&mut stdout, input.value(), input.cursor(), (0, 0), 15)?;
        stdout.flush()?;

        for evt in stdin.events() {
            let evt = evt?;
            if evt == Event::Key(Key::Esc) || evt == Event::Key(Key::Char('\n'))
            {
                break;
            }

            if backend::to_input_request(&evt)
                .and_then(|req| input.handle(req))
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

        value = input.value().to_string();
        write!(stdout, "{}", Show)?;
    }

    println!("{}", value);
    Ok(())
}
