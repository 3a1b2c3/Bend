use super::{parser::parse_definition_book, DefinitionBook};
use chumsky::prelude::Rich;
use itertools::Itertools;
use miette::{diagnostic, miette, Diagnostic, NamedSource, SourceSpan};
use std::{fmt::Display, path::Path};

/// Reads a file and parses to a definition book.
pub fn load_file_to_book(path: &Path) -> anyhow::Result<DefinitionBook> {
  let code = std::fs::read_to_string(path)?;
  match parse_definition_book(&code) {
    Ok(book) => Ok(book),
    Err(errs) => {
      let msg = errs.into_iter().map(|e| display_miette_err(e, path, &code)).join("\n");
      Err(anyhow::anyhow!(msg))
    }
  }
}

pub fn display_err_for_text<T: Display>(err: Rich<T>) -> String {
  err.to_string()
}

/// Displays a formatted [SyntaxError] from the given `err` based on the current report handler.
pub fn display_miette_err<T: Display>(err: Rich<T>, path: &Path, code: &str) -> String {
  let source = code.to_string();
  let name = path.to_str().unwrap();

  let src = NamedSource::new(name, source);
  let error = SyntaxError::from_rich(err, src);

  let report = miette!(error);

  format!("{report:?}")
}

#[derive(thiserror::Error, Debug, Diagnostic)]
#[error("{}", error)]
#[diagnostic()]
/// This structure holds information for syntax errors.
struct SyntaxError {
  /// The error name.
  error: String,
  #[source_code]
  /// The file name and source.
  src: NamedSource,
  #[label("{}", reason)]
  /// The error byte range.
  span: SourceSpan,
  /// The error reason.
  reason: String,
}

impl SyntaxError {
  /// Creates a new [SyntaxError] from a [Rich<T>] `err`.
  fn from_rich<T: Display>(err: Rich<T>, src: NamedSource) -> Self {
    let error = err.to_string();
    let reason = err.reason().to_string();
    let span = SourceSpan::from(err.span().into_range());

    Self { error, reason, span, src }
  }
}