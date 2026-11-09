use datanium_core::database::{Cell, CellWarning, DataType, Database, Value};

#[test]
fn partial_updates_preserve_omitted_cells_other_rows_and_identity()
-> Result<(), Box<dyn std::error::Error>> {
    let mut db = Database::new("Shop")?;
    let id = db.create_table("Items")?;
    let table = db.get_table_mut(&id)?;
    let name = table.create_column("Name", DataType::Text)?;
    let count = table.create_column("Count", DataType::Int)?;
    let row = table.add_row([(name, Value::Text("kept".into())), (count, Value::Int(1))])?;
    let other = table.add_row([(count, Value::Int(2))])?;
    table.update_row(&row, [(count, Value::Int(3))])?;
    table.update_row(&row, [])?;
    assert_eq!(
        table.get_row(&row)?.get_cell(&name),
        Some(&Cell::Valid(Value::Text("kept".into())))
    );
    assert_eq!(
        table.get_row(&row)?.get_cell(&count),
        Some(&Cell::Valid(Value::Int(3)))
    );
    assert_eq!(
        table.get_row(&other)?.get_cell(&count),
        Some(&Cell::Valid(Value::Int(2)))
    );
    assert_eq!(
        table.rows().map(|(id, _)| id).collect::<Vec<_>>(),
        vec![row, other]
    );
    table.update_row(&row, [(count, Value::Null), (name, Value::Null)])?;
    for column in [count, name] {
        assert_eq!(
            table.get_row(&row)?.get_cell(&column),
            Some(&Cell::Valid(Value::Null))
        );
    }
    table.update_row(&row, [(count, Value::Int(4))])?;
    assert_eq!(
        table.get_row(&row)?.get_cell(&count),
        Some(&Cell::Valid(Value::Int(4)))
    );
    Ok(())
}

#[test]
fn update_preserves_a_type_warning_until_the_value_is_corrected()
-> Result<(), Box<dyn std::error::Error>> {
    let mut db = Database::new("Shop")?;
    let id = db.create_table("Items")?;
    let table = db.get_table_mut(&id)?;
    let count = table.create_column("Count", DataType::Int)?;
    let name = table.create_column("Name", DataType::Text)?;
    let row = table.add_row([(count, Value::Int(1))])?;
    let warning = Cell::Warning {
        value: Value::Text("12 Stück".into()),
        warning: CellWarning::TypeMismatch {
            expected: DataType::Int,
            actual: DataType::Text,
        },
    };

    table.update_row(&row, [(count, Value::Text("12 Stück".into()))])?;
    assert_eq!(table.get_row(&row)?.get_cell(&count), Some(&warning));

    table.update_row(&row, [(name, Value::Text("Book".into()))])?;
    assert_eq!(table.get_row(&row)?.get_cell(&count), Some(&warning));

    table.update_row(&row, [(count, Value::Int(12))])?;
    assert_eq!(
        table.get_row(&row)?.get_cell(&count),
        Some(&Cell::Valid(Value::Int(12)))
    );
    Ok(())
}

#[test]
fn columnless_row_accepts_empty_update() -> Result<(), Box<dyn std::error::Error>> {
    let mut db = Database::new("Shop")?;
    let id = db.create_table("Items")?;
    let table = db.get_table_mut(&id)?;
    let row = table.add_row([])?;
    table.update_row(&row, [])?;
    assert_eq!(table.rows().count(), 1);
    Ok(())
}
