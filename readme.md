# Wicked

## Prerequisites

- `rust` (>= v1.75), `cargo`
- postgresql database
- list of `.com` domains obtained from https://czds.icann.org
- `bun` `v1.0.21` or higher (https://bun.sh/docs/installation)

## Steps to replicate:

- make a `config.local.yaml` file, and fill in your own values for `database_url`,
  `raw_domains_filepath`, `bun_bin_path`, and `words`, at least
- ensure your database exists and is reachable from your `database_url`
- you can test the classification of individual domains by running
  `cargo run -- check-domain somedomain.com`, using this to tune the weights and words in
  your configuration
- bootstrap the database by running
  `RUST_LOG=wicked=debug cargo run --release -- bootstrap` (this will take several minutes
  to insert over 100 million domains)
- check your desired sample size, and run
  `RUST_LOG=wicked=info cargo run --release -- exec`
