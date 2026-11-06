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
            RowId(number),
            Row {
                cells: BTreeMap::from([(first, Cell::Valid(Value::Null))]),
            },
        );
    }
    let second = table.create_column("Active", DataType::Bool).unwrap();
    for row in table.rows.values() {
        assert_eq!(row.cells.len(), 2);
        assert_eq!(row.cells.get(&first), Some(&Cell::Valid(Value::Null)));
        assert_eq!(row.cells.get(&second), Some(&Cell::Valid(Value::Null)));
    }
}
