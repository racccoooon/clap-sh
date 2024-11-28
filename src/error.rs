use std::fmt::Write;
use clap::builder::StyledStr;
use miette::Report;

pub struct Error {
    msg: StyledStr,
    use_stderr: bool,
}

impl Error {
    pub fn use_stderr(&self) -> bool {
        self.use_stderr
    }
    pub fn message(&self) -> &StyledStr {
        &self.msg
    }
}

impl From<Report> for Error {
    fn from(report: Report) -> Self {
        let mut msg = StyledStr::new();
        write!(msg, "{:?}", report).unwrap();
        Error {
            msg,
            use_stderr: true,
        }
    }
}

impl From<clap::Error> for Error {
    fn from(err: clap::Error) -> Self {
        Error {
            msg: err.render(),
            use_stderr: err.use_stderr(),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        let mut msg = StyledStr::new();
        write!(msg, "{:?}", err).unwrap();
        Error {
            msg,
            use_stderr: true,
        }
    }
}