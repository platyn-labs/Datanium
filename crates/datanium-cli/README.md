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
datanium database open example
```

The last command starts a session with the `example>` prompt.

### Available shell commands

| Command | Description |
| --- | --- |
| `datanium --help` | Show the available commands and options. |
| `datanium --version` | Print the installed CLI version. |
| `datanium database create <name>` | Create an empty `<name>.dtn` file in the current directory. |
| `datanium database list` | List the databases in the current directory. |
| `datanium database open <name>` | Open a local database and start an interactive session. |

### Available session commands

| Command | Description |
| --- | --- |
| `help` | Show the commands available in the session. |
| `exit` | Close the session and return to the shell. |

### Proposed session commands

These commands are ideas for working with the database selected by `database open`.

| Command | Description |
| --- | --- |
| `table create <name>` | Create an empty table in the open database. |
| `table list` | List the tables in the open database. |
| `table rename <name> <new-name>` | Rename a table and keep its columns and rows. |
| `column add <table> <name> --type <type>` | Add a column with type `int`, `text`, or `bool`. |
| `column list <table>` | Show the names and types of the columns in a table. |
| `column rename <table> <name> <new-name>` | Rename a column and keep its existing values. |
| `row add <table> <column=value>...` | Insert a row with the supplied column values. |
| `row list <table>` | Show the rows in a table. |

## Proposed network commands

| Command | Description |
| --- | --- |
| `datanium serve <name> --bind <host:port>` | Serve one database from the current directory at the specified address so other machines on the local network can access it. |
| `datanium connect <url>` | Connect to a Datanium server and open an interactive session for the database it serves. |
| `datanium status <url>` | Check whether a Datanium server is reachable and show which database it serves. |
| `disconnect` | Close the connection from inside a remote session and return to the shell. |
