//! Single-column Unique constraint behavior and definition validation.

use crate::database::{
    Cell, CellWarning, ColumnId, Constraint, DataType, Database, DatabaseError, RowId, Table,
    TableError, TableId, Value,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn setup() -> Result<(Database, TableId, ColumnId), Box<dyn std::error::Error>> {
    let mut database = Database::new("Shop")?;
    let table_id = database.create_table("Items")?;
    let key = database
        .get_table_mut(&table_id)?
        .create_column("Code", DataType::Int)?;
    Ok((database, table_id, key))
}

fn unique(key: ColumnId) -> Constraint {
    Constraint::Unique { columns: vec![key] }
}

// Warning order is not part of the contract. Each warning must occur once.
fn assert_cell(
    table: &Table,
    row: RowId,
    column: ColumnId,
    value: Value,
    expected_warnings: &[CellWarning],
) {
    let cell = table.get_row(&row).unwrap().get_cell(&column).unwrap();
    if expected_warnings.is_empty() {
        assert_eq!(cell, &Cell::Valid(value));
    } else {
        let Cell::Warning {
            value: actual_value,
            warnings,
        } = cell
        else {
            panic!("expected warnings {expected_warnings:?}, got {cell:?}");
        };
        assert_eq!(actual_value, &value);
        assert_eq!(warnings.len(), expected_warnings.len());
        for expected in expected_warnings {
            assert_eq!(
                warnings
                    .iter()
                    .filter(|warning| *warning == expected)
                    .count(),
                1
            );
        }
    }
}

fn assert_duplicates(table: &Table, rows: &[RowId], key: ColumnId, value: i64) {
    for &row in rows {
        assert_cell(
            table,
            row,
            key,
            Value::Int(value),
            &[CellWarning::Unique { columns: vec![key] }],
        );
    }
}

#[test]
fn adding_unique_warns_on_every_member_of_each_existing_duplicate_group() -> TestResult {
    let (mut database, table_id, key) = setup()?;
    let table = database.get_table_mut(&table_id)?;
    let a = table.add_row([(key, Value::Int(7))])?;
    let b = table.add_row([(key, Value::Int(7))])?;
    let c = table.add_row([(key, Value::Int(7))])?;
    let d = table.add_row([(key, Value::Int(8))])?;
    let e = table.add_row([(key, Value::Int(8))])?;
    let distinct = table.add_row([(key, Value::Int(9))])?;

    database.add_constraint(&table_id, unique(key))?;

    let table = database.get_table(&table_id)?;
    assert_duplicates(table, &[a, b, c], key, 7);
    assert_duplicates(table, &[d, e], key, 8);
    assert_cell(table, distinct, key, Value::Int(9), &[]);
    Ok(())
}

#[test]
fn unique_insert_revalidates_existing_and_new_rows_including_name_based_inserts() -> TestResult {
    let (mut database, table_id, key) = setup()?;
    database.add_constraint(&table_id, unique(key))?;
    let table = database.get_table_mut(&table_id)?;
    let a = table.add_row([(key, Value::Int(7))])?;
    assert_cell(table, a, key, Value::Int(7), &[]);
    let distinct = table.add_row([(key, Value::Int(8))])?;

    let b = table.add_row([(key, Value::Int(7))])?;
    assert_duplicates(table, &[a, b], key, 7);
    let c = table.add_row_by_column_names([("Code", Value::Int(7))])?;
    assert_duplicates(table, &[a, b, c], key, 7);
    assert_cell(table, distinct, key, Value::Int(8), &[]);
    assert_eq!(table.rows().count(), 4);
    Ok(())
}

#[test]
fn unique_updates_do_not_compare_a_row_with_itself() -> TestResult {
    let (mut database, table_id, key) = setup()?;
    database.add_constraint(&table_id, unique(key))?;
    let table = database.get_table_mut(&table_id)?;
    let row = table.add_row([(key, Value::Int(7))])?;

    table.update_row(&row, [(key, Value::Int(7))])?;
    assert_cell(table, row, key, Value::Int(7), &[]);
    table.update_row(&row, [(key, Value::Int(8))])?;
    assert_cell(table, row, key, Value::Int(8), &[]);
    table.update_row(&row, [])?;
    assert_cell(table, row, key, Value::Int(8), &[]);
    Ok(())
}

#[test]
fn unique_update_revalidates_both_old_and_new_duplicate_groups() -> TestResult {
    let (mut database, table_id, key) = setup()?;
    database.add_constraint(&table_id, unique(key))?;
    let table = database.get_table_mut(&table_id)?;
    let moving = table.add_row([(key, Value::Int(7))])?;
    let old_peer = table.add_row([(key, Value::Int(7))])?;
    let new_peer = table.add_row([(key, Value::Int(8))])?;
    assert_duplicates(table, &[moving, old_peer], key, 7);

    table.update_row(&moving, [(key, Value::Int(8))])?;

    assert_cell(table, old_peer, key, Value::Int(7), &[]);
    assert_duplicates(table, &[moving, new_peer], key, 8);

    table.update_row(&moving, [(key, Value::Int(9))])?;
    assert_cell(table, moving, key, Value::Int(9), &[]);
    assert_cell(table, new_peer, key, Value::Int(8), &[]);
    Ok(())
}

#[test]
fn unique_update_keeps_warnings_until_only_one_member_of_a_group_remains() -> TestResult {
    let (mut database, table_id, key) = setup()?;
    database.add_constraint(&table_id, unique(key))?;
    let table = database.get_table_mut(&table_id)?;
    let a = table.add_row([(key, Value::Int(7))])?;
    let b = table.add_row([(key, Value::Int(7))])?;
    let c = table.add_row([(key, Value::Int(7))])?;

    table.update_row(&a, [(key, Value::Int(8))])?;
    assert_cell(table, a, key, Value::Int(8), &[]);
    assert_duplicates(table, &[b, c], key, 7);

    table.update_row(&b, [(key, Value::Int(9))])?;
    assert_cell(table, b, key, Value::Int(9), &[]);
    assert_cell(table, c, key, Value::Int(7), &[]);
    Ok(())
}

#[test]
fn unique_delete_keeps_remaining_duplicates_warned_and_clears_the_last_survivor() -> TestResult {
    let (mut database, table_id, key) = setup()?;
    database.add_constraint(&table_id, unique(key))?;
    let table = database.get_table_mut(&table_id)?;
    let a = table.add_row([(key, Value::Int(7))])?;
    let b = table.add_row([(key, Value::Int(7))])?;
    let c = table.add_row([(key, Value::Int(7))])?;
    let d = table.add_row([(key, Value::Int(8))])?;
    let e = table.add_row([(key, Value::Int(8))])?;

    table.delete_row(&a)?;
    assert!(matches!(table.get_row(&a), Err(TableError::RowNotFound)));
    assert_duplicates(table, &[b, c], key, 7);
    assert_duplicates(table, &[d, e], key, 8);

    table.delete_row(&b)?;
    assert_cell(table, c, key, Value::Int(7), &[]);
    assert_duplicates(table, &[d, e], key, 8);
    Ok(())
}

#[test]
fn unique_ignores_existing_inserted_and_updated_nulls() -> TestResult {
    let (mut database, table_id, key) = setup()?;
    let table = database.get_table_mut(&table_id)?;
    let explicit = table.add_row([(key, Value::Null)])?;
    let omitted = table.add_row([])?;
    database.add_constraint(&table_id, unique(key))?;
    let table = database.get_table_mut(&table_id)?;
    let inserted_explicit = table.add_row([(key, Value::Null)])?;
    let inserted_omitted = table.add_row([])?;
    let a = table.add_row([(key, Value::Int(7))])?;
    let b = table.add_row([(key, Value::Int(7))])?;
    assert_duplicates(table, &[a, b], key, 7);

    table.update_row(&a, [(key, Value::Null)])?;

    for row in [explicit, omitted, inserted_explicit, inserted_omitted, a] {
        assert_cell(table, row, key, Value::Null, &[]);
    }
    assert_cell(table, b, key, Value::Int(7), &[]);
    Ok(())
}

#[test]
fn unique_ignores_type_mismatches_and_rechecks_a_corrected_value() -> TestResult {
    let (mut database, table_id, key) = setup()?;
    let table = database.get_table_mut(&table_id)?;
    let a = table.add_row([(key, Value::Text("7".into()))])?;
    let b = table.add_row([(key, Value::Text("7".into()))])?;
    let valid = table.add_row([(key, Value::Int(7))])?;
    database.add_constraint(&table_id, unique(key))?;
    let table = database.get_table_mut(&table_id)?;
    let c = table.add_row([(key, Value::Text("7".into()))])?;
    for row in [a, b, c] {
        assert_cell(
            table,
            row,
            key,
            Value::Text("7".into()),
            &[CellWarning::TypeMismatch {
                expected: DataType::Int,
                actual: DataType::Text,
            }],
        );
    }
    assert_cell(table, valid, key, Value::Int(7), &[]);

    table.update_row(&a, [(key, Value::Int(7))])?;
    assert_duplicates(table, &[a, valid], key, 7);

    table.update_row(&a, [(key, Value::Text("7".into()))])?;
    assert_cell(table, valid, key, Value::Int(7), &[]);
    for row in [a, b, c] {
        assert_cell(
            table,
            row,
            key,
            Value::Text("7".into()),
            &[CellWarning::TypeMismatch {
                expected: DataType::Int,
                actual: DataType::Text,
            }],
        );
    }
    Ok(())
}

#[test]
fn unique_revalidation_preserves_not_null_warnings_regardless_of_rule_order() -> TestResult {
    for unique_first in [false, true] {
        let (mut database, table_id, key) = setup()?;
        let table = database.get_table_mut(&table_id)?;
        let a = table.add_row([])?;
        let b = table.add_row([])?;
        let rules = if unique_first {
            [unique(key), Constraint::NotNull { column: key }]
        } else {
            [Constraint::NotNull { column: key }, unique(key)]
        };
        for rule in rules {
            database.add_constraint(&table_id, rule)?;
        }
        let table = database.get_table_mut(&table_id)?;
        let c = table.add_row([])?;
        for row in [a, b, c] {
            assert_cell(table, row, key, Value::Null, &[CellWarning::NotNull]);
        }
        table.update_row(&a, [(key, Value::Int(7))])?;
        table.update_row(&b, [(key, Value::Int(7))])?;
        assert_duplicates(table, &[a, b], key, 7);
        table.update_row(&a, [(key, Value::Null)])?;
        assert_cell(table, a, key, Value::Null, &[CellWarning::NotNull]);
        assert_cell(table, b, key, Value::Int(7), &[]);
        table.delete_row(&b)?;
        assert_cell(table, a, key, Value::Null, &[CellWarning::NotNull]);
        assert_cell(table, c, key, Value::Null, &[CellWarning::NotNull]);
    }
    Ok(())
}

#[test]
fn unique_revalidation_and_non_key_updates_preserve_other_cells_and_rules() -> TestResult {
    let (mut database, table_id, key) = setup()?;
    let note = database
        .get_table_mut(&table_id)?
        .create_column("Note", DataType::Text)?;
    database.add_constraint(&table_id, Constraint::NotNull { column: note })?;
    database.add_constraint(&table_id, unique(key))?;
    let table = database.get_table_mut(&table_id)?;
    let a = table.add_row([(key, Value::Int(7))])?;
    let b = table.add_row([(key, Value::Int(7)), (note, Value::Int(42))])?;

    table.update_row(&a, [(note, Value::Text("Restock".into()))])?;
    table.update_row(&b, [])?;
    table.update_row(&b, [(key, Value::Int(7))])?;
    assert_duplicates(table, &[a, b], key, 7);
    assert_cell(table, a, note, Value::Text("Restock".into()), &[]);
    assert_cell(
        table,
        b,
        note,
        Value::Int(42),
        &[CellWarning::TypeMismatch {
            expected: DataType::Text,
            actual: DataType::Int,
        }],
    );

    table.update_row(&a, [(key, Value::Int(8))])?;
    assert_cell(table, b, key, Value::Int(7), &[]);
    assert_cell(
        table,
        b,
        note,
        Value::Int(42),
        &[CellWarning::TypeMismatch {
            expected: DataType::Text,
            actual: DataType::Int,
        }],
    );
    Ok(())
}

#[test]
fn adding_unique_rejects_invalid_definitions_without_changing_cells_or_storing_a_rule() -> TestResult
{
    for invalid in 0..3 {
        let (mut database, table_id, key) = setup()?;
        let table = database.get_table_mut(&table_id)?;
        let a = table.add_row([(key, Value::Int(7))])?;
        let b = table.add_row([(key, Value::Int(7))])?;
        let (columns, expected) = match invalid {
            0 => (vec![], TableError::EmptyConstraintColumns),
            1 => (vec![key, key], TableError::DuplicateColumn(key)),
            _ => (vec![key, ColumnId::new(999)], TableError::ColumnNotFound),
        };

        assert_eq!(
            database.add_constraint(&table_id, Constraint::Unique { columns }),
            Err(DatabaseError::Constraint(expected))
        );

        let table = database.get_table_mut(&table_id)?;
        let c = table.add_row([(key, Value::Int(7))])?;
        for row in [a, b, c] {
            assert_cell(table, row, key, Value::Int(7), &[]);
        }
        assert_eq!(table.rows().count(), 3);
    }
    Ok(())
}

#[test]
fn adding_unique_rejects_an_unknown_table_without_changing_the_existing_table() -> TestResult {
    let (mut database, table_id, key) = setup()?;
    let table = database.get_table_mut(&table_id)?;
    let a = table.add_row([(key, Value::Int(7))])?;
    let b = table.add_row([(key, Value::Int(7))])?;

    assert_eq!(
        database.add_constraint(&TableId::new(999), unique(key)),
        Err(DatabaseError::TableNotFound)
    );

    let table = database.get_table_mut(&table_id)?;
    let c = table.add_row([(key, Value::Int(7))])?;
    for row in [a, b, c] {
        assert_cell(table, row, key, Value::Int(7), &[]);
    }
    Ok(())
}

#[test]
fn failed_row_mutations_preserve_unique_warnings_values_and_row_ids() -> TestResult {
    let (mut database, table_id, key) = setup()?;
    database.add_constraint(&table_id, unique(key))?;
    let table = database.get_table_mut(&table_id)?;
    let a = table.add_row([(key, Value::Int(7))])?;
    let b = table.add_row([(key, Value::Int(7))])?;
    let unknown = ColumnId::new(999);

    assert_eq!(
        table.add_row([(key, Value::Int(7)), (unknown, Value::Int(1))]),
        Err(TableError::ColumnNotFound)
    );
    assert_eq!(
        table.add_row([(key, Value::Int(7)), (key, Value::Int(8))]),
        Err(TableError::DuplicateColumn(key))
    );
    assert_eq!(
        table.update_row(&a, [(key, Value::Int(8)), (unknown, Value::Int(1))]),
        Err(TableError::ColumnNotFound)
    );
    assert_eq!(
        table.update_row(&a, [(key, Value::Int(8)), (key, Value::Int(9))]),
        Err(TableError::DuplicateColumn(key))
    );
    assert_eq!(
        table.delete_row(&RowId::new(999)),
        Err(TableError::RowNotFound)
    );
    assert_duplicates(table, &[a, b], key, 7);
    assert_eq!(table.rows().map(|(id, _)| id).collect::<Vec<_>>(), [a, b]);
    let c = table.add_row([(key, Value::Int(8))])?;
    assert_eq!(c, RowId::new(3));
    assert_cell(table, c, key, Value::Int(8), &[]);
    Ok(())
}
