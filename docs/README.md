# `SIMPLE` documentation

Published at <https://germolinal.github.io/simple/> **from release tags
only**: pushing a `v*` tag (see `make release`) runs
`.github/workflows/release.yaml`, which publishes the website after the tests
pass and the binaries are uploaded. Everything on the site (landing page,
book, IO reference, API docs and validation reports) describes that release,
so changes merged to `main` appear with the next release. To re-publish an
existing tag, run "Build and deploy docs" from the Actions tab.

Pull requests that touch the docs build the site (without publishing it) to
make sure it still works.

| Path | Content | Should you edit? |
|--|--|--|
| `website/` | Landing page ([Astro](https://astro.build) + [Tailwind](https://tailwindcss.com)): downloads and links to the book and the validation reports | Yes |
| `website/src/data/validation.ts` | The list of validation reports on the landing page | Yes, when adding reports |
| `book/src/*.md` (except `auto-*`) | Hand-written chapters (tutorials, etc.) | Yes, please |
| `book/SUMMARY_TEMPLATE.md` | Table of contents for the hand-written chapters | Yes, when adding chapters |
| `book/src/auto-*.md`, `SUMMARY.md`, `IO_SUMMARY.md` | Input/Output reference, generated from the `model` crate (see `derive`). Git-ignored | NO! |
| `validation/*.html` | Validation reports, written by `make validate` | NO! Re-run `make validate` and commit before releasing |
| `validation/.last_validated` | Written when a full `make validate` succeeds; marks when the reports were last validated | NO! Commit it with the reports |

## Building locally

Requires Node.js (24 or newer), `mdbook` 0.4.x and `mdbook-katex` 0.9.x:

```sh
cargo install mdbook --version "~0.4" --locked
cargo install mdbook-katex --version "~0.9" --locked

make serve_book                 # live preview of the book
cd docs/website && npm run dev  # live preview of the landing page (after `npm ci`)
make book                       # full site into target/site
```

The landing page shows download links for `$SIMPLE_VERSION` or, if it is not
set, the latest `v*` tag.

The documentation in this directory is released under CC0 (see `LICENSE`).
