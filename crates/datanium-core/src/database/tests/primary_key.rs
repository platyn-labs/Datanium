//! Single-column PrimaryKey specification.
//!
//! Violations reuse NotNull and Unique warnings. Type mismatches are excluded
//! from uniqueness comparisons. An identical rule is idempotent; a different
//! second primary key is a definition error. Composite keys follow separately.

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

fn primary_key(key: ColumnId) -> Constraint {
    Constraint::PrimaryKey { columns: vec![key] }
}

fn assert_cell(table: &Table, row: RowId, column: ColumnId, expected: Cell) {
    assert_eq!(
        table.get_row(&row).unwrap().get_cell(&column),
        Some(&expected)
    );
}

fn assert_duplicates(table: &Table, rows: &[RowId], key: ColumnId, value: i64) {
    for &row in rows {
        assert_cell(
            table,
            row,
            key,
            Cell::Warning {
                value: Value::Int(value),
                warnings: vec![CellWarning::Unique { columns: vec![key] }],
            },
        );
    }
}

fn assert_not_null(table: &Table, rows: &[RowId], key: ColumnId) {
    for &row in rows {
        assert_cell(
            table,
            row,
            key,
            Cell::Warning {
                value: Value::Null,
                warnings: vec![CellWarning::NotNull],
            },
        );
    }
}

#[test]
fn adding_primary_key_warns_on_existing_nulls_and_all_duplicate_rows() -> TestResult {
    let (mut database, table_id, key) = setup()?;
    let table = database.get_table_mut(&table_id)?;
    let explicit_null = table.add_row([(key, Value::Null)])?;
    let omitted = table.add_row([])?;
    let a = table.add_row([(key, Value::Int(7))])?;
    let b = table.add_row([(key, Value::Int(7))])?;
    let c = table.add_row([(key, Value::Int(7))])?;
    let distinct = table.add_row([(key, Value::Int(8))])?;

    database.add_constraint(&table_id, primary_key(key))?;

    let table = database.get_table(&table_id)?;
    assert_not_null(table, &[explicit_null, omitted], key);
    assert_duplicates(table, &[a, b, c], key, 7);
    assert_cell(table, distinct, key, Cell::Valid(Value::Int(8)));
    assert_eq!(table.rows().count(), 6);
    Ok(())
}

#[test]
fn primary_key_insert_checks_explicit_and_omitted_nulls_and_revalidates_duplicate_peers()
-> TestResult {
    let (mut database, table_id, key) = setup()?;
    database.add_constraint(&table_id, primary_key(key))?;
    let table = database.get_table_mut(&table_id)?;
    let explicit_null = table.add_row([(key, Value::Null)])?;
    let omitted = table.add_row([])?;
    let a = table.add_row([(key, Value::Int(7))])?;
    assert_cell(table, a, key, Cell::Valid(Value::Int(7)));
    let b = table.add_row_by_column_names([("Code", Value::Int(7))])?;
    let c = table.add_row([(key, Value::Int(7))])?;

    assert_not_null(table, &[explicit_null, omitted], key);
    assert_duplicates(table, &[a, b, c], key, 7);
    assert_eq!(table.rows().count(), 5);
    Ok(())
}

#[test]
fn primary_key_update_moves_between_null_valid_and_duplicate_states() -> TestResult {
    let (mut database, table_id, key) = setup()?;
    let note = database
        .get_table_mut(&table_id)?
        .create_column("Note", DataType::Text)?;
    database.add_constraint(&table_id, primary_key(key))?;
    let table = database.get_table_mut(&table_id)?;
    let moving = table.add_row([])?;
    let old_peer = table.add_row([(key, Value::Int(7))])?;
    let new_peer = table.add_row([(key, Value::Int(8))])?;

    table.update_row(&moving, [(key, Value::Int(7))])?;
    assert_duplicates(table, &[moving, old_peer], key, 7);
    table.update_row(&moving, [(note, Value::Text("Keep".into()))])?;
    table.update_row(&moving, [])?;
    assert_duplicates(table, &[moving, old_peer], key, 7);

    table.update_row(&moving, [(key, Value::Int(8))])?;
    assert_cell(table, old_peer, key, Cell::Valid(Value::Int(7)));
    assert_duplicates(table, &[moving, new_peer], key, 8);

    table.update_row(&moving, [(key, Value::Null)])?;
    assert_not_null(table, &[moving], key);
    assert_cell(table, new_peer, key, Cell::Valid(Value::Int(8)));

    table.update_row(&moving, [(key, Value::Int(9))])?;
    table.update_row(&moving, [(key, Value::Int(9))])?;
    assert_cell(table, moving, key, Cell::Valid(Value::Int(9)));
    assert_cell(table, moving, note, Cell::Valid(Value::Text("Keep".into())));
    Ok(())
}

#[test]
fn primary_key_delete_keeps_remaining_duplicates_warned_until_one_survives() -> TestResult {
    let (mut database, table_id, key) = setup()?;
    database.add_constraint(&table_id, primary_key(key))?;
    let table = database.get_table_mut(&table_id)?;
    let a = table.add_row([(key, Value::Int(7))])?;
    let b = table.add_row([(key, Value::Int(7))])?;
    let c = table.add_row([(key, Value::Int(7))])?;
    let missing = table.add_row([])?;

    table.delete_row(&a)?;
    assert_duplicates(table, &[b, c], key, 7);
    assert_not_null(table, &[missing], key);
    table.delete_row(&b)?;
    assert_cell(table, c, key, Cell::Valid(Value::Int(7)));
    assert_not_null(table, &[missing], key);
    Ok(())
}

#[test]
fn primary_key_rejects_invalid_definitions_without_storing_rules_or_changing_cells() -> TestResult {
    for invalid in 0..3 {
        let (mut database, table_id, key) = setup()?;
        let table = database.get_table_mut(&table_id)?;
        let missing = table.add_row([])?;
        let a = table.add_row([(key, Value::Int(7))])?;
        let b = table.add_row([(key, Value::Int(7))])?;
        let (columns, error) = match invalid {
            0 => (vec![], TableError::EmptyConstraintColumns),
            1 => (vec![key, key], TableError::DuplicateColumn(key)),
            _ => (vec![key, ColumnId::new(999)], TableError::ColumnNotFound),
        };

        assert_eq!(
            database.add_constraint(&table_id, Constraint::PrimaryKey { columns }),
            Err(DatabaseError::Constraint(error))
        );

        let table = database.get_table_mut(&table_id)?;
        let inserted_null = table.add_row([])?;
        let inserted_duplicate = table.add_row([(key, Value::Int(7))])?;
        for row in [missing, inserted_null] {
            assert_cell(table, row, key, Cell::Valid(Value::Null));
        }
        for row in [a, b, inserted_duplicate] {
            assert_cell(table, row, key, Cell::Valid(Value::Int(7)));
        }
        // A rejected definition must not occupy the table's primary-key slot.
        database.add_constraint(&table_id, primary_key(key))?;
    }
    Ok(())
}

#[test]
fn primary_key_rejects_an_unknown_table_without_changing_existing_cells() -> TestResult {
    let (mut database, table_id, key) = setup()?;
    let row = database.get_table_mut(&table_id)?.add_row([])?;

    assert_eq!(
        database.add_constraint(&TableId::new(999), primary_key(key)),
        Err(DatabaseError::TableNotFound)
    );

    assert_cell(
        database.get_table(&table_id)?,
        row,
        key,
        Cell::Valid(Value::Null),
    );
    Ok(())
}

#[test]
fn an_identical_primary_key_is_idempotent_but_a_different_second_key_is_rejected() -> TestResult {
    let (mut database, table_id, key) = setup()?;
    let other = database
        .get_table_mut(&table_id)?
        .create_column("Other", DataType::Int)?;
    let table = database.get_table_mut(&table_id)?;
    let a = table.add_row([(key, Value::Int(7))])?;
    let b = table.add_row([(key, Value::Int(7))])?;
    let missing = table.add_row([])?;
    database.add_constraint(&table_id, primary_key(key))?;
    database.add_constraint(&table_id, primary_key(key))?;

    assert!(matches!(
        database.add_constraint(&table_id, primary_key(other)),
        Err(DatabaseError::Constraint(_))
    ));

    let table = database.get_table_mut(&table_id)?;
    let inserted = table.add_row([(key, Value::Int(7))])?;
    assert_duplicates(table, &[a, b, inserted], key, 7);
    assert_not_null(table, &[missing], key);
    for row in [a, b, missing, inserted] {
        assert_cell(table, row, other, Cell::Valid(Value::Null));
    }
    // The restriction is per table, not per database.
    let other_table = database.create_table("OtherItems")?;
    let other_key = database
        .get_table_mut(&other_table)?
        .create_column("Code", DataType::Int)?;
    database.add_constraint(&other_table, primary_key(other_key))?;
    let missing = database.get_table_mut(&other_table)?.add_row([])?;
    assert_not_null(database.get_table(&other_table)?, &[missing], other_key);
    Ok(())
}

#[test]
fn primary_key_excludes_type_mismatches_and_rechecks_corrected_values() -> TestResult {
    let (mut database, table_id, key) = setup()?;
    let table = database.get_table_mut(&table_id)?;
    let a = table.add_row([(key, Value::Text("7".into()))])?;
    let b = table.add_row([(key, Value::Text("7".into()))])?;
    let valid = table.add_row([(key, Value::Int(7))])?;
    database.add_constraint(&table_id, primary_key(key))?;
    let table = database.get_table_mut(&table_id)?;
    let inserted = table.add_row([(key, Value::Text("7".into()))])?;
    let assert_type_warning = |table: &Table, row| {
        assert_cell(
            table,
            row,
            key,
            Cell::Warning {
                value: Value::Text("7".into()),
                warnings: vec![CellWarning::TypeMismatch {
                    expected: DataType::Int,
                    actual: DataType::Text,
                }],
            },
        );
    };
    for row in [a, b, inserted] {
        assert_type_warning(table, row);
    }
    assert_cell(table, valid, key, Cell::Valid(Value::Int(7)));

    table.update_row(&a, [(key, Value::Int(7))])?;
    assert_duplicates(table, &[a, valid], key, 7);
    table.update_row(&a, [(key, Value::Text("7".into()))])?;
    assert_type_warning(table, a);
    assert_cell(table, valid, key, Cell::Valid(Value::Int(7)));
    Ok(())
}

#[test]
fn deleting_the_primary_key_column_preserves_other_rules_and_allows_a_new_primary_key() -> TestResult
{
    let (mut database, table_id, key) = setup()?;
    let other = database
        .get_table_mut(&table_id)?
        .create_column("Other", DataType::Int)?;
    database.add_constraint(&table_id, primary_key(key))?;
    database.add_constraint(&table_id, Constraint::NotNull { column: other })?;
    database.add_constraint(
        &table_id,
        Constraint::Unique {
            columns: vec![other],
        },
    )?;
    let table = database.get_table_mut(&table_id)?;
    let a = table.add_row([(key, Value::Int(7)), (other, Value::Int(10))])?;
    let b = table.add_row([(key, Value::Int(7)), (other, Value::Int(10))])?;
    let missing = table.add_row([])?;

    table.delete_column(&key)?;

    for row in [a, b, missing] {
        assert_eq!(table.get_row(&row)?.get_cell(&key), None);
    }
    assert_duplicates(table, &[a, b], other, 10);
    assert_not_null(table, &[missing], other);
    let replacement = table.create_column("Code", DataType::Int)?;
    assert!(replacement > key);
    for row in [a, b, missing] {
        assert_cell(table, row, replacement, Cell::Valid(Value::Null));
    }

    database.add_constraint(&table_id, primary_key(replacement))?;

    let table = database.get_table_mut(&table_id)?;
    assert_not_null(table, &[a, b, missing], replacement);
    let inserted = table.add_row([(other, Value::Int(10))])?;
    assert_not_null(table, &[inserted], replacement);
    assert_duplicates(table, &[a, b, inserted], other, 10);
    assert_not_null(table, &[missing], other);
    Ok(())
}

#[test]
fn primary_key_and_explicit_not_null_and_unique_rules_do_not_duplicate_cell_warnings() -> TestResult
{
    for primary_first in [false, true] {
        let (mut database, table_id, key) = setup()?;
        let table = database.get_table_mut(&table_id)?;
        let a = table.add_row([(key, Value::Int(7))])?;
        let b = table.add_row([(key, Value::Int(7))])?;
        let missing = table.add_row([])?;
        let explicit = [
            Constraint::NotNull { column: key },
            Constraint::Unique { columns: vec![key] },
        ];
        if primary_first {
            database.add_constraint(&table_id, primary_key(key))?;
        }
        for rule in explicit {
            database.add_constraint(&table_id, rule)?;
        }
        if !primary_first {
            database.add_constraint(&table_id, primary_key(key))?;
        }
        let table = database.get_table_mut(&table_id)?;
        assert_duplicates(table, &[a, b], key, 7);
        assert_not_null(table, &[missing], key);
        table.update_row(&a, [(key, Value::Null)])?;
        assert_not_null(table, &[a, missing], key);
        assert_cell(table, b, key, Cell::Valid(Value::Int(7)));
    }
    Ok(())
}

#[test]
fn deleting_either_composite_primary_key_column_clears_only_warnings_no_longer_required()
-> TestResult {
    for delete_first in [false, true] {
        for explicit_not_null in [false, true] {
            let (mut database, table_id, first) = setup()?;
            let second = database
                .get_table_mut(&table_id)?
                .create_column("Other", DataType::Int)?;
            let (deleted, retained) = if delete_first {
                (first, second)
            } else {
                (second, first)
            };
            database.add_constraint(
                &table_id,
                Constraint::PrimaryKey {
                    columns: vec![first, second],
                },
            )?;
            if explicit_not_null {
                database.add_constraint(&table_id, Constraint::NotNull { column: retained })?;
            }
            database.add_constraint(
                &table_id,
                Constraint::Unique {
                    columns: vec![retained],
                },
            )?;
            let table = database.get_table_mut(&table_id)?;
            let a = table.add_row([(first, Value::Int(7)), (second, Value::Int(7))])?;
            let b = table.add_row([(first, Value::Int(7)), (second, Value::Int(7))])?;
            let missing = table.add_row([])?;
            let mismatched = table.add_row([(retained, Value::Text("bad".into()))])?;

            table.delete_column(&deleted)?;

            for row in [a, b, missing, mismatched] {
                assert_eq!(table.get_row(&row)?.get_cell(&deleted), None);
            }
            assert_duplicates(table, &[a, b], retained, 7);
            let inserted_null = table.add_row([])?;
            for row in [missing, inserted_null] {
                if explicit_not_null {
                    assert_not_null(table, &[row], retained);
                } else {
                    assert_cell(table, row, retained, Cell::Valid(Value::Null));
                }
            }
            assert_cell(
                table,
                mismatched,
                retained,
                Cell::Warning {
                    value: Value::Text("bad".into()),
                    warnings: vec![CellWarning::TypeMismatch {
                        expected: DataType::Int,
                        actual: DataType::Text,
                    }],
                },
            );
            let replacement = table.create_column("Replacement", DataType::Int)?;
            database.add_constraint(&table_id, primary_key(replacement))?;
            assert_not_null(
                database.get_table(&table_id)?,
                &[a, b, missing, mismatched, inserted_null],
                replacement,
            );
        }
    }
    Ok(())
}
