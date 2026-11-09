use super::*;

#[test]
fn rejected_columns_preserve_state_and_counter() {
    let mut table = Table::new(TableId::new(1), "Items");
    let id = table.create_column("Name", DataType::Text).unwrap();
    let long_name = "a".repeat(65);
    for name in ["", "1name", "my name", "my-name", "ä", "name\n", &long_name] {
        assert_eq!(
            table.create_column(name, DataType::Int),
            Err(TableError::InvalidColumnName)
        );
        assert_eq!(table.columns.len(), 1);
        assert_eq!(table.next_column_id, 2);
    }
    assert_eq!(
        table.create_column("Name", DataType::Bool),
        Err(TableError::ColumnAlreadyExists)
    );
    table.next_column_id = u64::MAX;
    assert_eq!(
        table.create_column("Other", DataType::Int),
        Err(TableError::ColumnIdExhausted)
    );
    assert_eq!(table.next_column_id, u64::MAX);
    assert_eq!(table.columns.len(), 1);
    assert_eq!(table.get_column(&id).unwrap().data_type(), &DataType::Text);
}

#[test]
fn new_column_backfills_existing_rows_without_replacing_cells() {
    let mut table = Table::new(TableId::new(1), "Items");
    let first = table.create_column("Name", DataType::Text).unwrap();
    for number in 1..=2 {
        table.rows.insert(
            RowId::new(number),
            Row::new(BTreeMap::from([(first, Cell::Valid(Value::Null))])),
        );
    }
    let second = table.create_column("Active", DataType::Bool).unwrap();
    for row in table.rows.values() {
        assert_eq!(row.get_cell(&first), Some(&Cell::Valid(Value::Null)));
        assert_eq!(row.get_cell(&second), Some(&Cell::Valid(Value::Null)));
    }
}

#[test]
fn rename_column_preserves_id_type_and_cells_and_updates_name_lookup() -> Result<(), TableError> {
    let mut table = Table::new(TableId::new(1), "Items");
    let id = table.create_column("Name", DataType::Text)?;
    let other = table.create_column("Active", DataType::Bool)?;
    let valid = table.add_row([(id, Value::Text("Book".into())), (other, Value::Bool(true))])?;
    let warned = table.add_row([(id, Value::Int(42))])?;
    let next_column_id = table.next_column_id;

    table.rename_column(&id, "Label")?;

    assert_eq!(table.get_column(&id)?.name(), "Label");
    assert_eq!(table.get_column(&id)?.data_type(), &DataType::Text);
    assert_eq!(table.get_column_by_name("Label")?.id(), &id);
    assert!(matches!(
        table.get_column_by_name("Name"),
        Err(TableError::ColumnNotFound)
    ));
    assert_eq!(table.get_column_by_name("Active")?.id(), &other);
    assert_eq!(table.columns().count(), 2);
    assert_eq!(table.next_column_id, next_column_id);
    assert_eq!(
        table.get_row(&valid)?.get_cell(&id),
        Some(&Cell::Valid(Value::Text("Book".into())))
    );
    assert_eq!(
        table.get_row(&valid)?.get_cell(&other),
        Some(&Cell::Valid(Value::Bool(true)))
    );
    assert_eq!(
        table.get_row(&warned)?.get_cell(&id),
        Some(&Cell::Warning {
            value: Value::Int(42),
            warning: crate::database::CellWarning::TypeMismatch {
                expected: DataType::Text,
                actual: DataType::Int,
            },
        })
    );

    let inserted = table.add_row_by_column_names([("Label", Value::Text("Pen".into()))])?;
    assert_eq!(
        table.get_row(&inserted)?.get_cell(&id),
        Some(&Cell::Valid(Value::Text("Pen".into())))
    );
    Ok(())
}

#[test]
fn rename_column_accepts_current_name_and_checks_uniqueness_case_sensitively()
-> Result<(), TableError> {
    let mut table = Table::new(TableId::new(1), "Items");
    let id = table.create_column("Name", DataType::Text)?;
    let other = table.create_column("Label", DataType::Text)?;
    let next_column_id = table.next_column_id;

    table.rename_column(&id, "Name")?;
    assert_eq!(table.get_column_by_name("Name")?.id(), &id);
    table.rename_column(&id, "name")?;
    assert_eq!(table.get_column_by_name("name")?.id(), &id);
    table.rename_column(&id, "label")?;
    assert_eq!(table.get_column_by_name("label")?.id(), &id);
    assert_eq!(table.get_column_by_name("Label")?.id(), &other);
    assert_eq!(table.next_column_id, next_column_id);
    Ok(())
}

#[test]
fn rename_column_accepts_a_name_with_64_characters() -> Result<(), TableError> {
    let mut table = Table::new(TableId::new(1), "Items");
    let id = table.create_column("Name", DataType::Text)?;
    let max_name = "a".repeat(64);

    table.rename_column(&id, &max_name)?;

    assert_eq!(table.get_column_by_name(&max_name)?.id(), &id);
    Ok(())
}

#[test]
fn rename_column_rejects_invalid_names_without_changing_state() -> Result<(), TableError> {
    let mut table = Table::new(TableId::new(1), "Items");
    let id = table.create_column("Name", DataType::Text)?;
    let next_column_id = table.next_column_id;
    let too_long = "a".repeat(65);

    for name in ["", "1name", "my name", "my-name", "ä", "name\n", &too_long] {
        assert_eq!(
            table.rename_column(&id, name),
            Err(TableError::InvalidColumnName),
            "expected invalid column name: {name:?}"
        );
        assert_eq!(table.get_column(&id)?.name(), "Name");
        assert_eq!(table.columns().count(), 1);
        assert_eq!(table.next_column_id, next_column_id);
    }
    Ok(())
}

#[test]
fn rename_column_rejects_missing_id_and_duplicate_name_without_changing_state()
-> Result<(), TableError> {
    let mut table = Table::new(TableId::new(1), "Items");
    let id = table.create_column("Name", DataType::Text)?;
    let other = table.create_column("Taken", DataType::Bool)?;
    let next_column_id = table.next_column_id;

    assert_eq!(
        table.rename_column(&ColumnId::new(42), "Taken"),
        Err(TableError::ColumnNotFound)
    );
    assert_eq!(table.get_column_by_name("Name")?.id(), &id);
    assert_eq!(table.get_column_by_name("Taken")?.id(), &other);
    assert_eq!(table.columns().count(), 2);
    assert_eq!(table.next_column_id, next_column_id);

    assert_eq!(
        table.rename_column(&id, "Taken"),
        Err(TableError::ColumnAlreadyExists)
    );
    assert_eq!(table.get_column_by_name("Name")?.id(), &id);
    assert_eq!(table.get_column_by_name("Taken")?.id(), &other);
    assert_eq!(table.columns().count(), 2);
    assert_eq!(table.next_column_id, next_column_id);
    Ok(())
}

#[test]
fn delete_column_removes_the_column_and_its_cells_from_every_row() -> Result<(), TableError> {
    let mut table = Table::new(TableId::new(1), "Items");
    let deleted = table.create_column("Name", DataType::Text)?;
    let retained = table.create_column("Count", DataType::Int)?;
    let first = table.add_row([
        (deleted, Value::Text("Book".into())),
        (retained, Value::Int(3)),
    ])?;
    let second = table.add_row([(deleted, Value::Int(42)), (retained, Value::Int(7))])?;
    let next_column_id = table.next_column_id;
    let next_row_id = table.next_row_id;

    assert_eq!(table.delete_column(&deleted), Ok(()));

    assert!(matches!(
        table.get_column(&deleted),
        Err(TableError::ColumnNotFound)
    ));
    assert!(matches!(
        table.get_column_by_name("Name"),
        Err(TableError::ColumnNotFound)
    ));
    assert_eq!(
        table
            .columns()
            .map(|column| *column.id())
            .collect::<Vec<_>>(),
        [retained]
    );
    assert_eq!(table.get_column_by_name("Count")?.id(), &retained);
    assert_eq!(table.get_column(&retained)?.data_type(), &DataType::Int);
    assert_eq!(
        table.rows().map(|(id, _)| id).collect::<Vec<_>>(),
        [first, second]
    );
    for (id, count) in [(first, 3), (second, 7)] {
        let row = table.get_row(&id)?;
        assert_eq!(row.get_cell(&deleted), None);
        assert_eq!(
            row.get_cell(&retained),
            Some(&Cell::Valid(Value::Int(count)))
        );
    }
    assert_eq!(table.next_column_id, next_column_id);
    assert_eq!(table.next_row_id, next_row_id);
    assert_eq!(
        table.delete_column(&deleted),
        Err(TableError::ColumnNotFound)
    );
    Ok(())
}

#[test]
fn delete_column_rejects_unknown_id_without_changing_state() -> Result<(), TableError> {
    let mut table = Table::new(TableId::new(1), "Items");
    let retained = table.create_column("Count", DataType::Int)?;
    let row = table.add_row([(retained, Value::Int(7))])?;
    let next_column_id = table.next_column_id;
    let next_row_id = table.next_row_id;

    assert_eq!(
        table.delete_column(&ColumnId::new(999)),
        Err(TableError::ColumnNotFound)
    );

    assert_eq!(table.columns().count(), 1);
    assert_eq!(table.get_column_by_name("Count")?.id(), &retained);
    assert_eq!(table.rows().map(|(id, _)| id).collect::<Vec<_>>(), [row]);
    assert_eq!(
        table.get_row(&row)?.get_cell(&retained),
        Some(&Cell::Valid(Value::Int(7)))
    );
    assert_eq!(table.next_column_id, next_column_id);
    assert_eq!(table.next_row_id, next_row_id);
    Ok(())
}

#[test]
fn delete_column_removes_only_its_not_null_rule() -> Result<(), TableError> {
    let mut table = Table::new(TableId::new(1), "Items");
    let deleted = table.create_column("Name", DataType::Text)?;
    let retained = table.create_column("Count", DataType::Int)?;
    table.add_not_null_constraint(deleted)?;
    table.add_not_null_constraint(retained)?;

    table.delete_column(&deleted)?;

    assert!(matches!(
        table.constraints.as_slice(),
        [Constraint::NotNull { column }] if *column == retained
    ));
    Ok(())
}

#[test]
fn delete_column_keeps_rows_and_frees_the_name_when_deleting_the_last_column()
-> Result<(), TableError> {
    let mut table = Table::new(TableId::new(1), "Items");
    let deleted = table.create_column("Count", DataType::Int)?;
    let row = table.add_row([(deleted, Value::Int(7))])?;

    table.delete_column(&deleted)?;

    assert_eq!(table.columns().count(), 0);
    assert_eq!(table.rows().map(|(id, _)| id).collect::<Vec<_>>(), [row]);
    assert_eq!(table.get_row(&row)?.get_cell(&deleted), None);

    let created = table.create_column("Count", DataType::Int)?;

    assert!(created > deleted);
    assert_eq!(table.get_column_by_name("Count")?.id(), &created);
    let retained = table.get_row(&row)?;
    assert_eq!(retained.get_cell(&deleted), None);
    assert_eq!(retained.get_cell(&created), Some(&Cell::Valid(Value::Null)));
    Ok(())
}
