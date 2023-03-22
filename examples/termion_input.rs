use std::io::{stdin, stdout, Result, Write};
use termion::cursor::{Hide, Show};
use termion::event::{Event, Key};
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::IntoAlternateScreen;
use tui_input::backend::termion as backend;
use tui_input::backend::termion::EventHandler;
use tui_input::Input;

fn main() -> Result<()> {
    let mut input: Input = "Hello ".into();
    {
        let stdin = stdin();
        let mut stdout = stdout().into_raw_mode()?.into_alternate_screen()?;

        write!(&mut stdout, "{}", Hide)?;
        backend::write(&mut stdout, input.value(), input.cursor(), (0, 0), 15)?;
        stdout.flush()?;

        for evt in stdin.events() {
            let evt = evt?;
            if evt == Event::Key(Key::Esc) || evt == Event::Key(Key::Char('\n')) {
                break;
            }

            if input.handle_event(&evt).is_some() {
                backend::write(&mut stdout, input.value(), input.cursor(), (0, 0), 15)?;
                stdout.flush()?;
            }
        }

        write!(stdout, "{}", Show)?;
    }

    println!("{}", input);
    Ok(())
}
