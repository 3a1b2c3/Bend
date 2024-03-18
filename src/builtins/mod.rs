use std::{
  str::FromStr,
  sync::{Arc, Mutex},
};

use hvmc::{ast, host::Host, stdlib::LogDef};

use crate::{
  readback_hvmc,
  term::{
    builtins::{SCONS, SNIL},
    term_to_net::Labels,
    AdtEncoding, Book, Term,
  },
};

use self::query::make_query_def;

pub mod exit;
pub mod fs;
pub mod query;
pub mod util;

/// These are the names of builtin defs that are not in the hvm-lang book, but
/// are present in the hvm-core book. They are implemented using Rust code by
/// [`create_host`] and they can not be rewritten as hvm-lang functions.
pub const CORE_BUILTINS: [&str; 7] =
  ["HVM.log", "HVM.black_box", "HVM.print", "HVM.query", "HVM.store", "HVM.load", "HVM.exit"];
/// List of definition names used by the core builtins
pub const CORE_BUILTINS_USES: [&[&str]; 7] = [&[], &[], &[], &[SCONS, SNIL], &[], &[], &[]];

/// Creates a host with the hvm-core primitive definitions built-in.
/// This needs the book as an Arc because the closure that logs
/// data needs access to the book.
pub fn create_host(book: Arc<Book>, labels: Arc<Labels>, adt_encoding: AdtEncoding) -> Arc<Mutex<Host>> {
  let host = Arc::new(Mutex::new(Host::default()));
  host.lock().unwrap().insert_def("HVM.log", unsafe {
    LogDef::new(host.clone(), {
      let book = book.clone();
      let labels = labels.clone();
      move |tree| {
        let net = hvmc::ast::Net { root: tree, redexes: vec![] };
        let (term, errs) = readback_hvmc(&net, &book, &labels, false, adt_encoding);
        println!("{}{}", errs.display_with_severity(crate::diagnostics::Severity::Error), term);
      }
    })
  });
  host.lock().unwrap().insert_def("HVM.print", unsafe {
    LogDef::new(host.clone(), {
      let book = book.clone();
      let labels = labels.clone();
      move |tree| {
        let net = hvmc::ast::Net { root: tree, redexes: vec![] };
        let (term, _errs) = readback_hvmc(&net, &book, &labels, false, adt_encoding);
        if let Term::Str { val } = &term {
          println!("{val}");
        }
      }
    })
  });
  host.lock().unwrap().insert_def("HVM.query", make_query_def(host.clone(), labels.clone()));
  fs::add_fs_defs(book.clone(), host.clone(), labels.clone(), adt_encoding);
  exit::add_exit_def(host.clone());
  let book = ast::Book::from_str("@HVM.black_box = (x x)").unwrap();
  host.lock().unwrap().insert_book(&book);

  host
}