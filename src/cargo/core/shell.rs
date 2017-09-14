use std::fmt;
use std::io::prelude::*;

use atty;
use console::{self, Color, Term, Style};

use util::errors::CargoResult;

#[derive(Clone, Copy, PartialEq)]
pub enum Verbosity {
    Verbose,
    Normal,
    Quiet
}

pub struct Shell {
    pub err: ShellOut,
    verbosity: Verbosity,
}

impl fmt::Debug for Shell {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Shell")
    }
}

pub enum ShellOut {
    Write(Box<Write>),
    Stream(Term, ColorChoice),
}

#[derive(PartialEq, Clone, Copy)]
pub enum ColorChoice {
    Always,
    Never,
    CargoAuto,
}

impl Shell {
    pub fn new() -> Shell {
        set_colors(&ColorChoice::CargoAuto);

        Shell {
            err: ShellOut::Stream(
                Term::stderr(),
                ColorChoice::CargoAuto,
            ),
            verbosity: Verbosity::Verbose,
        }
    }

    pub fn from_write(out: Box<Write>) -> Shell {
        Shell {
            err: ShellOut::Write(out),
            verbosity: Verbosity::Verbose,
        }
    }

    fn print(&mut self,
             status: &fmt::Display,
             message: &fmt::Display,
             color: Color,
             justified: bool) -> CargoResult<()> {
        match self.verbosity {
            Verbosity::Quiet => Ok(()),
            _ => {
                self.err.print(status, message, color, justified)
            }
        }
    }

    pub fn err(&mut self) -> &mut Write {
        self.err.as_write()
    }

    pub fn status<T, U>(&mut self, status: T, message: U) -> CargoResult<()>
        where T: fmt::Display, U: fmt::Display
    {
        self.print(&status, &message, Color::Green, true)
    }

    pub fn status_with_color<T, U>(&mut self,
                                   status: T,
                                   message: U,
                                   color: Color) -> CargoResult<()>
        where T: fmt::Display, U: fmt::Display
    {
        self.print(&status, &message, color, true)
    }

    pub fn verbose<F>(&mut self, mut callback: F) -> CargoResult<()>
        where F: FnMut(&mut Shell) -> CargoResult<()>
    {
        match self.verbosity {
            Verbosity::Verbose => callback(self),
            _ => Ok(())
        }
    }

    pub fn concise<F>(&mut self, mut callback: F) -> CargoResult<()>
        where F: FnMut(&mut Shell) -> CargoResult<()>
    {
        match self.verbosity {
            Verbosity::Verbose => Ok(()),
            _ => callback(self)
        }
    }

    pub fn error<T: fmt::Display>(&mut self, message: T) -> CargoResult<()> {
        self.print(&"error:", &message, Color::Red, false)
    }

    pub fn warn<T: fmt::Display>(&mut self, message: T) -> CargoResult<()> {
        match self.verbosity {
            Verbosity::Quiet => Ok(()),
            _ => self.print(&"warning:", &message, Color::Yellow, false),
        }
    }

    pub fn set_verbosity(&mut self, verbosity: Verbosity) {
        self.verbosity = verbosity;
    }

    pub fn verbosity(&self) -> Verbosity {
        self.verbosity
    }

    pub fn set_color_choice(&mut self, color: Option<&str>) -> CargoResult<()> {
        if let ShellOut::Stream(ref mut err, ref mut cc) =  self.err {
            let cfg = match color {
                Some("always") => ColorChoice::Always,
                Some("never") => ColorChoice::Never,

                Some("auto") |
                None => ColorChoice::CargoAuto,

                Some(arg) => bail!("argument for --color must be auto, always, or \
                                    never, but found `{}`", arg),
            };
            *cc = cfg;
            set_colors(&cfg);
            *err = Term::stderr();
        }
        Ok(())
    }

    pub fn color_choice(&self) -> ColorChoice {
        match self.err {
            ShellOut::Stream(_, cc) => cc,
            ShellOut::Write(_) => ColorChoice::Never,
        }
    }
}

impl ShellOut {
    fn print(&mut self,
             status: &fmt::Display,
             message: &fmt::Display,
             color: Color,
             justified: bool) -> CargoResult<()> {
        match *self {
            ShellOut::Stream(ref mut err, _) => {
                let style = Style::new()
                    .fg(color)
                    .bold();

                if justified {
                    write!(err, "{:>12}", style.apply_to(status))?;
                } else {
                    write!(err, "{}", style.apply_to(status))?;
                }
                write!(err, " {}\n", message)?;
            }
            ShellOut::Write(ref mut w) => {
                if justified {
                    write!(w, "{:>12}", status)?;
                } else {
                    write!(w, "{}", status)?;
                }
                write!(w, " {}\n", message)?;
            }
        }
        Ok(())
    }

    fn as_write(&mut self) -> &mut Write {
        match *self {
            ShellOut::Stream(ref mut err, _) => err,
            ShellOut::Write(ref mut w) => w,
        }
    }
}

fn set_colors(choice: &ColorChoice) {
    match *choice {
        ColorChoice::Always => console::set_colors_enabled(true),
        ColorChoice::Never => console::set_colors_enabled(false),
        ColorChoice::CargoAuto => {
            if atty::is(atty::Stream::Stderr) {
                console::set_colors_enabled(true);
            } else {
                console::set_colors_enabled(false);
            }
        }
    }
}
