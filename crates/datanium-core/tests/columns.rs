use datanium_core::database::{DataType, Database, DatabaseError, TableError};

#[test]
fn public_api_creates_and_reads_columns() -> Result<(), Box<dyn std::error::Error>> {
    let mut database = Database::new("Shop")?;
    let table_id = database.create_table("Items")?;
    let table = database.get_table_mut(&table_id)?;
    let id = table.create_column("Name", DataType::Text)?;
    assert_eq!(table.get_column_by_name(String::from("Name"))?.id(), &id);
    let borrowed = String::from("Name");
    assert_eq!(table.get_column_by_name(&borrowed)?.id(), &id);
    assert_eq!(borrowed, "Name");
    let lower = table.create_column("name", DataType::Int)?;
    assert_ne!(id, lower);
    assert_eq!(table.get_column(&id)?.name(), "Name");
    assert_eq!(table.get_column_by_name("Name")?.id(), &id);
    assert_eq!(table.get_column(&lower)?.data_type(), &DataType::Int);
    assert!(matches!(
        table.get_column_by_name("Missing"),
        Err(TableError::ColumnNotFound)
    ));
    assert!(matches!(
        table.get_column_by_name("invalid name"),
        Err(TableError::ColumnNotFound)
    ));
    let max_name = "a".repeat(64);
    database
        .get_table_mut_by_name("Items")?
        .create_column(&max_name, DataType::Bool)?;
    database.rename_table(&table_id, "Products")?;
    assert_eq!(
        database.get_table(&table_id)?.get_column(&id)?.name(),
        "Name"
    );
    assert!(matches!(
        database.get_table_mut_by_name("Items"),
        Err(DatabaseError::TableNotFound)
    ));
    let other = database.create_table("Other")?;
    assert!(matches!(
        database.get_table(&other)?.get_column(&id),
        Err(TableError::ColumnNotFound)
    ));
    let mut empty = Database::new("Empty")?;
    assert!(matches!(
        empty.get_table_mut(&table_id),
        Err(DatabaseError::TableNotFound)
    ));
    Ok(())
}

#[test]
fn table_errors_expose_standard_error_messages() {
    for error in [
        TableError::ColumnIdExhausted,
        TableError::RowIdExhausted,
        TableError::RowNotFound,
        TableError::ColumnAlreadyExists,
        TableError::InvalidColumnName,
        TableError::ColumnNotFound,
        TableError::EmptyConstraintColumns,
    ] {
        let standard: &dyn std::error::Error = &error;
        assert!(!standard.to_string().is_empty());
        assert!(standard.source().is_none());
    }
}
