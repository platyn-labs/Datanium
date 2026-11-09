# Datanium CLI

## Local installation

To install the `datanium-cli` on you local machine, run this from the repository root.

```sh
cargo install --path crates/datanium-cli --locked
```

## Working with local databases

Change to the directory where you want to keep your databases, then run these commands.

```sh
datanium database create example
datanium database list
```

The first command creates `example.dtn` in the current directory. The second lists the databases in that directory.
