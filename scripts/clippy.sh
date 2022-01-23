cargo clippy -- \
  -D clippy::pedantic \
  -A clippy::must-use-candidate \
  -A clippy::doc-markdown \
  -A clippy::missing-errors-doc \
  -D clippy::dbg_macro \
  -D clippy::unwrap_used
