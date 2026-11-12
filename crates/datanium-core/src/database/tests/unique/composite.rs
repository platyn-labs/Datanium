use super::*;

#[test]
fn adding_composite_unique_warns_on_all_key_cells_only_for_equal_whole_keys() -> TestResult {
    let (mut database, table_id, columns @ [code, label]) = setup_pair()?;
    let table = database.get_table_mut(&table_id)?;
    let a = table.add_row([(code, Value::Int(7)), (label, Value::Text("A".into()))])?;
    let b = table.add_row([(code, Value::Int(7)), (label, Value::Text("A".into()))])?;
    let c = table.add_row([(code, Value::Int(7)), (label, Value::Text("A".into()))])?;
    let same_code = table.add_row([(code, Value::Int(7)), (label, Value::Text("B".into()))])?;
    let same_label = table.add_row([(code, Value::Int(8)), (label, Value::Text("A".into()))])?;

    database.add_constraint(&table_id, pair_rule(columns))?;

    let table = database.get_table(&table_id)?;
    let warning = [CellWarning::Unique {
        columns: columns.to_vec(),
    }];
    assert_pair(table, &[a, b, c], columns, 7, "A", &warning);
    assert_pair(table, &[same_code], columns, 7, "B", &[]);
    assert_pair(table, &[same_label], columns, 8, "A", &[]);
    Ok(())
}

#[test]
fn composite_unique_insert_and_delete_revalidate_every_member_of_the_group() -> TestResult {
    let (mut database, table_id, columns @ [code, label]) = setup_pair()?;
    database.add_constraint(&table_id, pair_rule(columns))?;
    let table = database.get_table_mut(&table_id)?;
    let a = table.add_row([(code, Value::Int(7)), (label, Value::Text("A".into()))])?;
    assert_pair(table, &[a], columns, 7, "A", &[]);
    let b = table.add_row([(code, Value::Int(7)), (label, Value::Text("A".into()))])?;
    let warning = [CellWarning::Unique {
        columns: columns.to_vec(),
    }];
    assert_pair(table, &[a, b], columns, 7, "A", &warning);
    let c = table
        .add_row_by_column_names([("Code", Value::Int(7)), ("Label", Value::Text("A".into()))])?;
    let distinct = table.add_row([(code, Value::Int(7)), (label, Value::Text("B".into()))])?;
    assert_pair(table, &[a, b, c], columns, 7, "A", &warning);
    assert_pair(table, &[distinct], columns, 7, "B", &[]);

    table.delete_row(&a)?;
    assert_pair(table, &[b, c], columns, 7, "A", &warning);
    table.delete_row(&b)?;
    assert_pair(table, &[c], columns, 7, "A", &[]);
    assert_pair(table, &[distinct], columns, 7, "B", &[]);
    Ok(())
}

#[test]
fn composite_unique_update_revalidates_old_and_new_groups_for_either_key_column() -> TestResult {
    let (mut database, table_id, columns @ [code, label]) = setup_pair()?;
    database.add_constraint(&table_id, pair_rule(columns))?;
    let table = database.get_table_mut(&table_id)?;
    let moving = table.add_row([(code, Value::Int(7)), (label, Value::Text("A".into()))])?;
    let old_peer = table.add_row([(code, Value::Int(7)), (label, Value::Text("A".into()))])?;
    let label_peer = table.add_row([(code, Value::Int(7)), (label, Value::Text("B".into()))])?;
    let code_peer = table.add_row([(code, Value::Int(8)), (label, Value::Text("B".into()))])?;
    let warning = [CellWarning::Unique {
        columns: columns.to_vec(),
    }];

    table.update_row(&moving, [(label, Value::Text("B".into()))])?;
    assert_pair(table, &[old_peer], columns, 7, "A", &[]);
    assert_pair(table, &[moving, label_peer], columns, 7, "B", &warning);
    assert_pair(table, &[code_peer], columns, 8, "B", &[]);

    table.update_row(&moving, [(code, Value::Int(8))])?;
    assert_pair(table, &[label_peer], columns, 7, "B", &[]);
    assert_pair(table, &[moving, code_peer], columns, 8, "B", &warning);

    table.update_row(&moving, [(code, Value::Int(9))])?;
    assert_pair(table, &[moving], columns, 9, "B", &[]);
    assert_pair(table, &[code_peer], columns, 8, "B", &[]);
    Ok(())
}

#[test]
fn composite_unique_excludes_the_whole_key_when_either_component_is_null() -> TestResult {
    for null_first in [false, true] {
        let (mut database, table_id, columns @ [code, label]) = setup_pair()?;
        let table = database.get_table_mut(&table_id)?;
        // Omitting one component must behave like explicitly storing NULL.
        let a = if null_first {
            table.add_row([(label, Value::Text("A".into()))])?
        } else {
            table.add_row([(code, Value::Int(7))])?
        };
        let b = if null_first {
            table.add_row([(code, Value::Null), (label, Value::Text("A".into()))])?
        } else {
            table.add_row([(code, Value::Int(7)), (label, Value::Null)])?
        };
        let valid = table.add_row([(code, Value::Int(7)), (label, Value::Text("A".into()))])?;
        database.add_constraint(&table_id, pair_rule(columns))?;
        let table = database.get_table_mut(&table_id)?;
        let inserted = if null_first {
            table.add_row([(code, Value::Null), (label, Value::Text("A".into()))])?
        } else {
            table.add_row([(code, Value::Int(7)), (label, Value::Null)])?
        };
        for row in [a, b, inserted] {
            assert_cell(
                table,
                row,
                code,
                if null_first {
                    Value::Null
                } else {
                    Value::Int(7)
                },
                &[],
            );
            assert_cell(
                table,
                row,
                label,
                if null_first {
                    Value::Text("A".into())
                } else {
                    Value::Null
                },
                &[],
            );
        }
        assert_pair(table, &[valid], columns, 7, "A", &[]);

        let changed = if null_first { code } else { label };
        let corrected = if null_first {
            Value::Int(7)
        } else {
            Value::Text("A".into())
        };
        table.update_row(&a, [(changed, corrected)])?;
        let warning = [CellWarning::Unique {
            columns: columns.to_vec(),
        }];
        assert_pair(table, &[a, valid], columns, 7, "A", &warning);

        table.update_row(&a, [(changed, Value::Null)])?;
        assert_cell(table, a, changed, Value::Null, &[]);
        let other = if null_first { label } else { code };
        let other_value = if null_first {
            Value::Text("A".into())
        } else {
            Value::Int(7)
        };
        assert_cell(table, a, other, other_value, &[]);
        assert_pair(table, &[valid], columns, 7, "A", &[]);
    }
    Ok(())
}

#[test]
fn composite_unique_excludes_the_whole_key_when_either_component_has_a_type_warning() -> TestResult
{
    for invalid_first in [false, true] {
        let (mut database, table_id, columns @ [code, label]) = setup_pair()?;
        let table = database.get_table_mut(&table_id)?;
        let invalid_values = || {
            if invalid_first {
                [
                    (code, Value::Text("7".into())),
                    (label, Value::Text("A".into())),
                ]
            } else {
                [(code, Value::Int(7)), (label, Value::Int(7))]
            }
        };
        let a = table.add_row(invalid_values())?;
        let b = table.add_row(invalid_values())?;
        let valid = table.add_row([(code, Value::Int(7)), (label, Value::Text("A".into()))])?;
        database.add_constraint(&table_id, pair_rule(columns))?;
        let table = database.get_table_mut(&table_id)?;
        let inserted = table.add_row(invalid_values())?;
        let (invalid_column, expected, actual) = if invalid_first {
            (code, DataType::Int, DataType::Text)
        } else {
            (label, DataType::Text, DataType::Int)
        };
        let assert_excluded = |table: &Table, rows: &[RowId]| {
            for &row in rows {
                for (column, value) in invalid_values() {
                    if column == invalid_column {
                        assert_cell(
                            table,
                            row,
                            column,
                            value,
                            &[CellWarning::TypeMismatch { expected, actual }],
                        );
                    } else {
                        assert_cell(table, row, column, value, &[]);
                    }
                }
            }
        };
        assert_excluded(table, &[a, b, inserted]);
        assert_pair(table, &[valid], columns, 7, "A", &[]);

        let corrected = if invalid_first {
            Value::Int(7)
        } else {
            Value::Text("A".into())
        };
        table.update_row(&a, [(invalid_column, corrected)])?;
        let warning = [CellWarning::Unique {
            columns: columns.to_vec(),
        }];
        assert_pair(table, &[a, valid], columns, 7, "A", &warning);

        table.update_row(&a, invalid_values())?;
        assert_excluded(table, &[a, b, inserted]);
        assert_pair(table, &[valid], columns, 7, "A", &[]);
    }
    Ok(())
}

#[test]
fn overlapping_unique_rules_keep_two_warnings_on_the_same_cell_and_resolve_independently()
-> TestResult {
    for composite_first in [false, true] {
        let (mut database, table_id, columns @ [code, label]) = setup_pair()?;
        let table = database.get_table_mut(&table_id)?;
        let a = table.add_row([(code, Value::Int(7)), (label, Value::Text("A".into()))])?;
        let b = table.add_row([(code, Value::Int(7)), (label, Value::Text("A".into()))])?;
        let rules = if composite_first {
            [pair_rule(columns), unique(code)]
        } else {
            [unique(code), pair_rule(columns)]
        };
        for rule in rules {
            database.add_constraint(&table_id, rule)?;
        }
        let table = database.get_table_mut(&table_id)?;
        let single_warning = CellWarning::Unique {
            columns: vec![code],
        };
        let pair_warning = CellWarning::Unique {
            columns: columns.to_vec(),
        };
        // Rechecking the same value must not duplicate either rule's warning.
        for _ in 0..2 {
            table.update_row(&a, [(code, Value::Int(7))])?;
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
                    std::slice::from_ref(&pair_warning),
                );
            }
        }

        table.update_row(&a, [(label, Value::Text("B".into()))])?;
        for row in [a, b] {
            assert_cell(
                table,
                row,
                code,
                Value::Int(7),
                std::slice::from_ref(&single_warning),
            );
        }
        assert_cell(table, a, label, Value::Text("B".into()), &[]);
        assert_cell(table, b, label, Value::Text("A".into()), &[]);

        table.update_row(&a, [(code, Value::Int(8))])?;
        assert_pair(table, &[a], columns, 8, "B", &[]);
        assert_pair(table, &[b], columns, 7, "A", &[]);
    }
    Ok(())
}
