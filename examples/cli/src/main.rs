use std::fs::File;
use std::io::{self, Read};
use std::ops::Range;
use std::path::PathBuf;

use jval::{ErrorKind, Json, Spacing};
use structopt::StructOpt;
use termion::{
    color::{self, Bg, Fg},
    style,
};

#[derive(Debug)]
enum Error {
    IO(io::Error),
    Json(ErrorKind, Range<usize>),
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::IO(e)
    }
}

impl From<(ErrorKind, Range<usize>)> for Error {
    fn from((e, r): (ErrorKind, Range<usize>)) -> Self {
        Error::Json(e, r)
    }
}

#[derive(StructOpt)]
struct Cli {
    /// Don't print the data to stdout
    #[structopt(short, long, group = "printing")]
    quiet: bool,
    /// Print with whitespace removed
    #[structopt(short, long, group = "printing")]
    compact: bool,
    /// Print indented with tabs [default]
    #[structopt(short, long, group = "printing")]
    #[allow(dead_code)]
    tabs: bool,
    /// Print indented with <n> spaces
    #[structopt(long, group = "printing", name = "number")]
    spaces: Option<usize>,
    /// File to read JSON from
    #[structopt(short, long, name = "path", group = "input")]
    file: Option<PathBuf>,
    /// JSON data to validate
    #[structopt(name = "json_data", group = "input")]
    json: Option<String>,
}

fn main() -> Result<(), Error> {
    let cli = Cli::from_args();

    let json = if let Some(j) = cli.json {
        j
    } else if let Some(p) = cli.file {
        let mut b = String::new();
        File::open(p)?.read_to_string(&mut b)?;
        b
    } else {
        let mut b = String::new();
        io::stdin().read_to_string(&mut b)?;
        b
    };

    let parsed = match json.parse::<Json>() {
        Ok(data) => data,
        Err(errvec) => {
            eprintln!("Encountered {} error(s) while parsing JSON:", errvec.len());
            for (kind, Range { start, end }) in errvec {
                eprintln!("\n{:?} from position {} to {}.", kind, start, end);

                let range_start = json[..start]
                    .rmatch_indices('\n')
                    .map(|(n, _)| n + 1)
                    .nth(2)
                    .unwrap_or_default();
                let range_end = json[end..]
                    .match_indices('\n')
                    .map(|(n, _)| n + end)
                    .nth(2)
                    .unwrap_or(json.len());
                eprintln!(
                    "{}{red}{white}{}{reset}{}",
                    &json[range_start..start],
                    &json[start..end],
                    &json[end..range_end].trim_end(),
                    red = Bg(color::Red),
                    white = Fg(color::LightWhite),
                    reset = style::Reset
                );
            }
            std::process::exit(1);
        }
    };

    if !cli.quiet {
        let indent = if cli.compact {
            Spacing::None
        } else if let Some(n) = cli.spaces {
            Spacing::Space(n)
        } else {
            Spacing::Tab
        };

        parsed.print(&indent, &mut io::stdout())?;
        println!();
    }

    Ok(())
}
