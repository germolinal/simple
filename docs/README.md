# `SIMPLE` documentation

Published at <https://germolinal.github.io/simple/>:

* **Pushing to `main`** with changes in `docs/` (`.github/workflows/docs.yaml`)
  re-publishes the hand-written docs. The IO reference and validation reports
  still come from the latest `v*` tag, so they always describe a release.
* **Pushing a `v*` tag** (`.github/workflows/docs-release.yaml`) re-publishes
  everything from the tag. If code changed after the last full validation (see
  `check_validation.sh`), `make release` asks before tagging, and the release
  workflows only warn.

| Path | Content | Should you edit? |
|--|--|--|
| `index.html` | Landing page, linking to the book and the validation reports | Yes |
| `book/src/*.md` (except `auto-*`) | Hand-written chapters (tutorials, etc.) | Yes, please |
| `book/SUMMARY_TEMPLATE.md` | Table of contents for the hand-written chapters | Yes, when adding chapters |
| `book/src/auto-*.md`, `SUMMARY.md`, `IO_SUMMARY.md` | Input/Output reference, generated from the `model` crate (see `derive`). Git-ignored | NO! |
| `validation/*.html` | Validation reports, written by `make validate` | NO! Re-run `make validate` and commit before tagging |
| `validation/.last_validated` | Written when a full `make validate` succeeds; marks when the reports were last validated | NO! Commit it with the reports |

## Building locally

Requires `mdbook` 0.4.x and `mdbook-katex` 0.9.x:

```sh
cargo install mdbook --version "~0.4" --locked
cargo install mdbook-katex --version "~0.9" --locked

make serve_book   # live preview of the book
make book         # full site (book + validation reports) into target/site
```

The documentation in this directory is released under CC0 (see `LICENSE`).
