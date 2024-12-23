# Finance Fusion API

This is the REST API for the finance fusion app, made with rust 🚀.

## Run Instructions

### Prerequisites

- Rust

### Run

```bash
git clone https://github.com/HamzaAlsarakbi/finance-fusion
cd finance-fusion
cargo build
cargo run
```

### TODO

1. Unit Tests
2. Jenkins pipeline for CI/CD

## Development

### Updating the Schema (Pre v1.0)

1. Update the `up.sql` and `down.sql`
2. Run `diesel migration redo`
3. Run `diesel print-schema > src/database/schema.rs`

### Updating the Schema (Post v1.0)

1. Run `diesel migration generate [schema update name]`
2. Update the `up.sql` and `down.sql`
3. Run `diesel migration run`
4. Run `diesel print-schema > src/database/schema.rs`

### Logging into Postgres for debugging the database

1. Login to the postgress session with `psql -U postgres -d finance_fusion`
2. Print the tables with `\dt`
3. Run any command you wish

    Example:

    ```sql
    DROP TABLE IF EXISTS users CASCADE;
    ```
