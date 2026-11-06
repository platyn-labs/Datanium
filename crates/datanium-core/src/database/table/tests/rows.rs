use super::*;

#[test]
fn rejected_inputs_preserve_rows_and_counter() {
    let mut table = Table::new(TableId::new(1), "Items");
    let column = table.create_column("Name", DataType::Text).unwrap();
    let existing = table
        .add_row([(column, Value::Text("kept".into()))])
        .unwrap();
    let unknown = ColumnId::new(99);
    assert_eq!(
        table.add_row([(column, Value::Null), (unknown, Value::Null)]),
        Err(TableError::ColumnNotFound)
    );
    assert_eq!(
        table.add_row([(column, Value::Null), (column, Value::Null)]),
        Err(TableError::DuplicateColumn(column))
    );
    assert_eq!(
        table.add_row_by_column_names([("Name", Value::Null), ("missing", Value::Null)]),
        Err(TableError::ColumnNotFound)
    );
    assert_eq!(
        table.add_row_by_column_names([("Name", Value::Null), ("Name", Value::Null)]),
        Err(TableError::DuplicateColumn(column))
    );
    assert_eq!(table.rows.len(), 1);
    assert_eq!(table.next_row_id, 2);
    assert_eq!(
        table.get_row(&existing).unwrap().get_cell(&column),
        Some(&Cell::Valid(Value::Text("kept".into())))
    );
}

#[test]
fn exhaustion_preserves_state_and_documents_error_precedence() {
    let mut table = Table::new(TableId::new(1), "Items");
    let id = table.add_row([]).unwrap();
    table.next_row_id = u64::MAX;
    assert_eq!(
        table.add_row([(ColumnId::new(99), Value::Null)]),
        Err(TableError::RowIdExhausted)
    );
    assert_eq!(
        table.add_row_by_column_names([("missing", Value::Null)]),
        Err(TableError::ColumnNotFound)
    );
    assert_eq!(
        table.add_row_by_column_names(Vec::<(String, Value)>::new()),
        Err(TableError::RowIdExhausted)
    );
    assert_eq!(table.next_row_id, u64::MAX);
    assert_eq!(table.rows.len(), 1);
    assert!(table.get_row(&id).is_ok());
}
