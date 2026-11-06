use datanium_core::database::{Cell, CellError, DataType, Database, RowId, TableError, Value};

#[test]
fn inserts_and_reads_owned_and_borrowed_names() -> Result<(), Box<dyn std::error::Error>> {
    let mut db = Database::new("Shop")?;
    let table_id = db.create_table("Items")?;
    let table = db.get_table_mut(&table_id)?;
    let text = table.create_column("Name", DataType::Text)?;
    let int = table.create_column("Count", DataType::Int)?;
    let boolean = table.create_column("Active", DataType::Bool)?;
    let a = table.add_row([
        (text, Value::Text("Anna".into())),
        (int, Value::Int(28)),
        (boolean, Value::Bool(true)),
    ])?;
    let b = table.add_row_by_column_names([("Name", Value::Null)])?;
    let c = table.add_row_by_column_names([(String::from("Name"), Value::Text("Bob".into()))])?;
    assert_ne!(a, b);
    assert_ne!(b, c);
    assert_eq!(
        table.get_row(&a)?.get_cell(&text),
        Some(&Cell::Valid(Value::Text("Anna".into())))
    );
    assert_eq!(
        table.get_row(&a)?.get_cell(&int),
        Some(&Cell::Valid(Value::Int(28)))
    );
    assert_eq!(
        table.get_row(&a)?.get_cell(&boolean),
        Some(&Cell::Valid(Value::Bool(true)))
    );
    for column in [text, int, boolean] {
        assert_eq!(
            table.get_row(&b)?.get_cell(&column),
            Some(&Cell::Valid(Value::Null))
        );
    }
    let added = table.create_column("Later", DataType::Text)?;
    assert_eq!(
        table.get_row(&a)?.get_cell(&added),
        Some(&Cell::Valid(Value::Null))
    );
    assert_eq!(
        table.get_row(&c)?.get_cell(&text),
        Some(&Cell::Valid(Value::Text("Bob".into())))
    );
    assert!(matches!(
        table.get_row(&RowId::new(999)),
        Err(TableError::RowNotFound)
    ));
    let other = db.create_table("Other")?;
    let foreign_column = db
        .get_table_mut(&other)?
        .create_column("Other", DataType::Text)?;
    let empty = db.get_table_mut(&other)?.add_row([])?;
    assert_eq!(
        db.get_table(&other)?.get_row(&empty)?.get_cell(&added),
        None
    );
    assert_eq!(
        db.get_table(&other)?
            .get_row(&empty)?
            .get_cell(&foreign_column),
        Some(&Cell::Valid(Value::Null))
    );
    Ok(())
}

#[test]
fn type_matrix_preserves_original_values_and_accepts_null() -> Result<(), Box<dyn std::error::Error>>
{
    let mut db = Database::new("Shop")?;
    let id = db.create_table("Items")?;
    let table = db.get_table_mut(&id)?;
    for (name, expected) in [
        ("Number", DataType::Int),
        ("Text", DataType::Text),
        ("Flag", DataType::Bool),
    ] {
        let column = table.create_column(name, expected)?;
        for (value, actual) in [
            (Value::Int(7), DataType::Int),
            (Value::Text("7".into()), DataType::Text),
            (Value::Bool(true), DataType::Bool),
        ] {
            let row = table.add_row([(column, value)])?;
            let original = match actual {
                DataType::Int => Value::Int(7),
                DataType::Text => Value::Text("7".into()),
                DataType::Bool => Value::Bool(true),
            };
            let wanted = if actual == expected {
                Cell::Valid(original)
            } else {
                Cell::Error {
                    value: original,
                    error: CellError::TypeMismatch { expected, actual },
                }
            };
            assert_eq!(table.get_row(&row)?.get_cell(&column), Some(&wanted));
        }
        let null = table.add_row([(column, Value::Null)])?;
        assert_eq!(
            table.get_row(&null)?.get_cell(&column),
            Some(&Cell::Valid(Value::Null))
        );
    }
    Ok(())
}

#[test]
fn columnless_rows_can_be_inserted_and_later_backfilled() -> Result<(), Box<dyn std::error::Error>>
{
    let mut db = Database::new("Shop")?;
    let id = db.create_table("Items")?;
    let table = db.get_table_mut(&id)?;
    let row = table.add_row([])?;
    let column = table.create_column("Name", DataType::Text)?;
    assert_eq!(
        table.get_row(&row)?.get_cell(&column),
        Some(&Cell::Valid(Value::Null))
    );
    Ok(())
}
