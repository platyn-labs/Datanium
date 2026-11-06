use super::*;

#[test]
fn rejected_updates_preserve_every_cell_and_id_counters() {
    let mut table = Table::new(TableId::new(1), "Items");
    let column = table.create_column("Name", DataType::Text).unwrap();
    let row = table
        .add_row([(column, Value::Text("kept".into()))])
        .unwrap();
    let other = table
        .add_row([(column, Value::Text("other".into()))])
        .unwrap();
    let unknown = ColumnId::new(99);
    assert_eq!(
        table.update_row(&row, [(column, Value::Null), (unknown, Value::Null)]),
        Err(TableError::ColumnNotFound)
    );
    assert_eq!(
        table.update_row(&row, [(column, Value::Null), (column, Value::Int(5))]),
        Err(TableError::DuplicateColumn(column))
    );
    assert_eq!(
        table.update_row(&RowId::new(99), [(unknown, Value::Null)]),
        Err(TableError::RowNotFound)
    );
    assert_eq!(
        table.update_row(&RowId::new(99), []),
        Err(TableError::RowNotFound)
    );
    assert_eq!(table.rows.len(), 2);
    assert_eq!(table.next_row_id, 3);
    assert_eq!(table.next_column_id, 2);
    assert_eq!(
        table.get_row(&row).unwrap().get_cell(&column),
        Some(&Cell::Valid(Value::Text("kept".into())))
    );
    assert_eq!(
        table.get_row(&other).unwrap().get_cell(&column),
        Some(&Cell::Valid(Value::Text("other".into())))
    );
}

#[test]
fn updates_work_even_when_row_ids_are_exhausted() {
    let mut table = Table::new(TableId::new(1), "Items");
    let column = table.create_column("Count", DataType::Int).unwrap();
    let row = table.add_row([]).unwrap();
    table.next_row_id = u64::MAX;
    table.update_row(&row, [(column, Value::Int(7))]).unwrap();
    table.update_row(&row, []).unwrap();
    assert_eq!(table.next_row_id, u64::MAX);
    assert_eq!(table.next_column_id, 2);
    assert_eq!(
        table.get_row(&row).unwrap().get_cell(&column),
        Some(&Cell::Valid(Value::Int(7)))
    );
}
