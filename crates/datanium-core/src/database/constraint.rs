use super::table::{ColumnId, TableId};

pub(super) enum Constraint {
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

pub(super) struct ForeignKeyColumn {
    source: ColumnId,
    target: ColumnId,
}

pub(super) enum ReferentialAction {
    Restrict,
    Cascade,
    SetNull,
}
