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
    let invalid = table.add_row([(id, Value::Int(42))])?;
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
        table.get_row(&invalid)?.get_cell(&id),
        Some(&Cell::Error {
            value: Value::Int(42),
            error: crate::database::CellError::TypeMismatch {
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
