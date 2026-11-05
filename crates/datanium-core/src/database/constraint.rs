use super::column::ColumnId;
use super::table::TableId;

#[expect(
    dead_code,
    reason = "Reserved for upcoming row and constraint operations"
)]
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

#[expect(
    dead_code,
    reason = "Reserved for upcoming row and constraint operations"
)]
pub(super) struct ForeignKeyColumn {
    source: ColumnId,
    target: ColumnId,
}

#[expect(
    dead_code,
    reason = "Reserved for upcoming row and constraint operations"
)]
pub(super) enum ReferentialAction {
    Restrict,
    Cascade,
    SetNull,
}
