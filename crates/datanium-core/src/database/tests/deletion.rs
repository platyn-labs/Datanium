use crate::database::{Cell, DataType, Database, DatabaseError, TableId, Value};

#[test]
fn delete_table_removes_only_the_requested_table() -> Result<(), Box<dyn std::error::Error>> {
    let mut database = Database::new("Shop")?;
    let deleted = database.create_table("ArchivedItems")?;
    let retained = database.create_table("Items")?;
    let table = database.get_table_mut(&retained)?;
    let column = table.create_column("Count", DataType::Int)?;
    let row = table.add_row([(column, Value::Int(7))])?;
    let next_table_id = database.next_table_id;

    assert_eq!(database.delete_table(&deleted), Ok(()));

    assert!(matches!(
        database.get_table(&deleted),
        Err(DatabaseError::TableNotFound)
    ));
    assert!(matches!(
        database.get_table_by_name("ArchivedItems"),
        Err(DatabaseError::TableNotFound)
    ));
    assert_eq!(
        database
            .tables()
            .map(|table| table.id())
            .collect::<Vec<_>>(),
        [retained]
    );
    let table = database.get_table_by_name("Items")?;
    assert_eq!(table.get_column_by_name("Count")?.id(), &column);
    assert_eq!(
        table.get_row(&row)?.get_cell(&column),
        Some(&Cell::Valid(Value::Int(7)))
    );
    assert_eq!(database.next_table_id, next_table_id);
    assert_eq!(
        database.delete_table(&deleted),
        Err(DatabaseError::TableNotFound)
    );
    Ok(())
}

#[test]
fn delete_table_rejects_unknown_id_without_changing_state() -> Result<(), DatabaseError> {
    let mut database = Database::new("Shop")?;
    let retained = database.create_table("Items")?;
    let next_table_id = database.next_table_id;

    assert_eq!(
        database.delete_table(&TableId::new(999)),
        Err(DatabaseError::TableNotFound)
    );

    assert_eq!(
        database
            .tables()
            .map(|table| table.id())
            .collect::<Vec<_>>(),
        [retained]
    );
    assert_eq!(database.get_table_by_name("Items")?.id(), retained);
    assert_eq!(database.next_table_id, next_table_id);
    Ok(())
}

#[test]
fn delete_table_frees_the_name_but_does_not_reuse_the_id() -> Result<(), Box<dyn std::error::Error>>
{
    let mut database = Database::new("Shop")?;
    let deleted = database.create_table("Items")?;
    let table = database.get_table_mut(&deleted)?;
    let column = table.create_column("Count", DataType::Int)?;
    table.add_row([(column, Value::Int(7))])?;

    database.delete_table(&deleted)?;
    let created = database.create_table("Items")?;

    assert!(created > deleted);
    assert!(matches!(
        database.get_table(&deleted),
        Err(DatabaseError::TableNotFound)
    ));
    let table = database.get_table_by_name("Items")?;
    assert_eq!(table.id(), created);
    assert_eq!(table.columns().count(), 0);
    assert_eq!(table.rows().count(), 0);
    Ok(())
}
