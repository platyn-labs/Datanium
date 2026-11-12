use super::*;

#[test]
fn deleting_either_composite_key_column_removes_the_entire_rule_and_its_warnings() -> TestResult {
    for delete_first in [false, true] {
        let (mut database, table_id, columns @ [code, label]) = setup_pair()?;
        database.add_constraint(&table_id, pair_rule(columns))?;
        let table = database.get_table_mut(&table_id)?;
        let a = table.add_row([(code, Value::Int(7)), (label, Value::Text("A".into()))])?;
        let b = table.add_row([(code, Value::Int(7)), (label, Value::Text("A".into()))])?;
        let distinct = table.add_row([(code, Value::Int(7)), (label, Value::Text("B".into()))])?;
        let warning = [CellWarning::Unique {
            columns: columns.to_vec(),
        }];
        assert_pair(table, &[a, b], columns, 7, "A", &warning);
        let (deleted, retained) = if delete_first {
            (code, label)
        } else {
            (label, code)
        };

        table.delete_column(&deleted)?;

        for row in [a, b, distinct] {
            assert_eq!(table.get_row(&row)?.get_cell(&deleted), None);
            let value = if delete_first {
                Value::Text(if row == distinct { "B" } else { "A" }.into())
            } else {
                Value::Int(7)
            };
            // The rule must disappear entirely, rather than shrink to one column.
            assert_cell(table, row, retained, value, &[]);
        }
        // Removal is immediate; later writes must not resurrect the rule.
        let value = if delete_first {
            Value::Text("A".into())
        } else {
            Value::Int(7)
        };
        let inserted = table.add_row([(retained, value)])?;
        let value = if delete_first {
            Value::Text("A".into())
        } else {
            Value::Int(7)
        };
        assert_cell(table, inserted, retained, value, &[]);
        let value = if delete_first {
            Value::Text("A".into())
        } else {
            Value::Int(7)
        };
        assert_cell(table, a, retained, value, &[]);
    }
    Ok(())
}

#[test]
fn deleting_a_key_column_removes_only_its_rule_warnings_and_keeps_other_rules_active() -> TestResult
{
    let (mut database, table_id, columns @ [code, label]) = setup_pair()?;
    let note = database
        .get_table_mut(&table_id)?
        .create_column("Note", DataType::Text)?;
    database.add_constraint(&table_id, Constraint::NotNull { column: note })?;
    database.add_constraint(&table_id, unique(code))?;
    database.add_constraint(&table_id, pair_rule(columns))?;
    let table = database.get_table_mut(&table_id)?;
    let a = table.add_row([(code, Value::Int(7)), (label, Value::Text("A".into()))])?;
    let b = table.add_row([
        (code, Value::Int(7)),
        (label, Value::Text("A".into())),
        (note, Value::Int(42)),
    ])?;

    table.delete_column(&label)?;

    // Code loses its composite warning while retaining its single-column warning.
    assert_duplicates(table, &[a, b], code, 7);
    assert_cell(table, a, note, Value::Null, &[CellWarning::NotNull]);
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
    for row in [a, b] {
        assert_eq!(table.get_row(&row)?.get_cell(&label), None);
    }

    table.update_row(&a, [(code, Value::Int(8))])?;
    assert_cell(table, a, code, Value::Int(8), &[]);
    assert_cell(table, b, code, Value::Int(7), &[]);
    let inserted = table.add_row([(code, Value::Int(7))])?;
    assert_duplicates(table, &[b, inserted], code, 7);
    assert_cell(table, inserted, note, Value::Null, &[CellWarning::NotNull]);
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
fn deleting_an_unrelated_or_unknown_column_preserves_all_unique_warnings() -> TestResult {
    let (mut database, table_id, columns @ [code, label]) = setup_pair()?;
    let note = database
        .get_table_mut(&table_id)?
        .create_column("Note", DataType::Text)?;
    database.add_constraint(&table_id, unique(code))?;
    database.add_constraint(&table_id, pair_rule(columns))?;
    let table = database.get_table_mut(&table_id)?;
    let a = table.add_row([(code, Value::Int(7)), (label, Value::Text("A".into()))])?;
    let b = table.add_row([(code, Value::Int(7)), (label, Value::Text("A".into()))])?;

    for deleting_unknown in [true, false] {
        if deleting_unknown {
            assert_eq!(
                table.delete_column(&ColumnId::new(999)),
                Err(TableError::ColumnNotFound)
            );
            assert!(table.get_column(&note).is_ok());
        } else {
            table.delete_column(&note)?;
        }
        for row in [a, b] {
            assert_cell(
                table,
                row,
                code,
                Value::Int(7),
                &[
                    CellWarning::Unique {
                        columns: vec![code],
                    },
                    CellWarning::Unique {
                        columns: columns.to_vec(),
                    },
                ],
            );
            assert_cell(
                table,
                row,
                label,
                Value::Text("A".into()),
                &[CellWarning::Unique {
                    columns: columns.to_vec(),
                }],
            );
        }
    }
    let inserted = table.add_row([(code, Value::Int(7)), (label, Value::Text("A".into()))])?;
    assert_cell(
        table,
        inserted,
        code,
        Value::Int(7),
        &[
            CellWarning::Unique {
                columns: vec![code],
            },
            CellWarning::Unique {
                columns: columns.to_vec(),
            },
        ],
    );
    assert_cell(
        table,
        inserted,
        label,
        Value::Text("A".into()),
        &[CellWarning::Unique {
            columns: columns.to_vec(),
        }],
    );
    Ok(())
}
