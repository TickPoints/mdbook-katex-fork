use clap::{crate_version, Arg, ArgMatches, Command};
use mdbook_katex_fork::{init_tracing, preprocess::KatexProcessor};
use mdbook_preprocessor::errors::{Error, Result};
use mdbook_preprocessor::Preprocessor;
use std::io;

/// Parse CLI options.
pub fn make_app() -> Command {
    Command::new("mdbook-katex-fork")
        .version(crate_version!())
        .about("A preprocessor that renders KaTex equations to HTML.")
        .subcommand(
            Command::new("supports")
                .arg(Arg::new("renderer").required(true))
                .about("Check whether a renderer is supported by this preprocessor"),
        )
}

/// Tell mdBook if we support what it asks for.
fn handle_supports(pre: &dyn Preprocessor, sub_args: &ArgMatches) -> Result<()> {
    let renderer = sub_args
        .get_one::<String>("renderer")
        .expect("Required argument");
    let supported = pre.supports_renderer(renderer).unwrap_or(false);
    if supported {
        Ok(())
    } else {
        Err(Error::msg(format!(
            "The katex preprocessor does not support the '{renderer}' renderer",
        )))
    }
}

/// Preprocess `book` using `pre` and print it out.
fn handle_preprocessing(pre: &dyn Preprocessor) -> Result<()> {
    let (ctx, book) = mdbook_preprocessor::parse_input(io::stdin())?;

    let processed_book = pre.run(&ctx, book)?;
    serde_json::to_writer(io::stdout(), &processed_book)?;
    Ok(())
}

fn main() -> Result<()> {
    init_tracing();

    // set up app
    let matches = make_app().get_matches();
    let pre = KatexProcessor;

    // determine what behaviour has been requested
    if let Some(sub_args) = matches.subcommand_matches("supports") {
        // handle cmdline supports
        handle_supports(&pre, sub_args)
    } else {
        // handle preprocessing
        handle_preprocessing(&pre)
    }
}
