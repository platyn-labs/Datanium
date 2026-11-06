use std::fmt;

use super::column::ColumnId;

/// An error returned when creating a database or accessing its tables.
#[derive(Debug, PartialEq, Eq)]
pub enum DatabaseError {
    /// The table ID counter cannot be advanced.
    TableIdExhausted,
    /// Another table in this database already has the requested name.
    TableAlreadyExists,
    /// The database name does not follow the documented name format.
    InvalidDatabaseName,
    /// The table name does not follow the documented name format.
    InvalidTableName,
    /// No table matches the requested ID or exact name.
    TableNotFound,
}

impl fmt::Display for DatabaseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::TableIdExhausted => "table ID counter is exhausted",
            Self::TableAlreadyExists => "a table with this name already exists",
            Self::InvalidDatabaseName => "invalid database name",
            Self::InvalidTableName => "invalid table name",
            Self::TableNotFound => "table not found",
        };

        formatter.write_str(message)
    }
}

impl std::error::Error for DatabaseError {}

#[derive(Debug, PartialEq, Eq)]
/// A schema, row lookup, or insertion failure.
pub enum TableError {
    ColumnIdExhausted,
    RowIdExhausted,
    ColumnAlreadyExists,
    InvalidColumnName,
    ColumnNotFound,
    DuplicateColumn(ColumnId),
    RowNotFound,
}

impl fmt::Display for TableError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::ColumnIdExhausted => "column ID counter is exhausted",
            Self::RowIdExhausted => "row ID counter is exhausted",
            Self::ColumnAlreadyExists => "a column with this name already exists",
            Self::InvalidColumnName => "invalid column name",
            Self::ColumnNotFound => "column not found",
            Self::DuplicateColumn(_) => "a column was supplied more than once",
            Self::RowNotFound => "row not found",
        })
    }
}

impl std::error::Error for TableError {}
