use super::column::ColumnId;
use super::table::TableId;

/// A rule that can be defined for a table.
pub enum Constraint {
    NotNull {
        column: ColumnId,
    },
    Unique {
        columns: Vec<ColumnId>,
    },
    PrimaryKey {
        columns: Vec<ColumnId>,
    },
    ForeignKey {
        target_table: TableId,
        columns: Vec<ForeignKeyColumn>,
        on_delete: ReferentialAction,
        on_update: ReferentialAction,
    },
}

/// A pair of source and target columns in a foreign key.
pub struct ForeignKeyColumn {
    pub source: ColumnId,
    pub target: ColumnId,
}

/// How a foreign key responds to a change in the referenced row.
pub enum ReferentialAction {
    Restrict,
    Cascade,
    SetNull,
}
