# Developer Guide

## Local Development

```sh
# Install dependencies and build the program
cargo build --release

# Run the program
./target/release/main -i <filename>.jsonl

# Validate the output SQL file
pgsanity ./schema.sql
```
