use crate::database::{
    Cell, CellWarning, ColumnId, Constraint, DataType, Database, TableId, Value,
};

#[test]
fn adding_not_null_warns_on_existing_null_cells() -> Result<(), Box<dyn std::error::Error>> {
    let mut database = Database::new("Shop")?;
    let table_id = database.create_table("Items")?;
    let table = database.get_table_mut(&table_id)?;
    let count = table.create_column("Count", DataType::Int)?;
    let note = table.create_column("Note", DataType::Text)?;
    let missing_count = table.add_row([(count, Value::Null)])?;
    let valid_count = table.add_row([(count, Value::Int(7))])?;

    database.add_constraint(&table_id, Constraint::NotNull { column: count })?;

    let table = database.get_table(&table_id)?;
    let row = table.get_row(&missing_count)?;
    assert_eq!(
        row.get_cell(&count),
        Some(&Cell::Warning {
            value: Value::Null,
            warning: CellWarning::NotNull,
        })
    );
    assert_eq!(row.get_cell(&note), Some(&Cell::Valid(Value::Null)));
    assert_eq!(
        table.get_row(&valid_count)?.get_cell(&count),
        Some(&Cell::Valid(Value::Int(7)))
    );
    Ok(())
}

#[test]
fn not_null_insert_warns_for_explicit_and_omitted_nulls() -> Result<(), Box<dyn std::error::Error>>
{
    let mut database = Database::new("Shop")?;
    let table_id = database.create_table("Items")?;
    let count = database
        .get_table_mut(&table_id)?
        .create_column("Count", DataType::Int)?;
    database.add_constraint(&table_id, Constraint::NotNull { column: count })?;
    let table = database.get_table_mut(&table_id)?;

    let explicit = table.add_row([(count, Value::Null)])?;
    let omitted = table.add_row([])?;

    for id in [explicit, omitted] {
        assert_eq!(
            table.get_row(&id)?.get_cell(&count),
            Some(&Cell::Warning {
                value: Value::Null,
                warning: CellWarning::NotNull,
            })
        );
    }
    Ok(())
}

#[test]
fn not_null_update_revalidates_the_changed_cell() -> Result<(), Box<dyn std::error::Error>> {
    let mut database = Database::new("Shop")?;
    let table_id = database.create_table("Items")?;
    let table = database.get_table_mut(&table_id)?;
    let count = table.create_column("Count", DataType::Int)?;
    let note = table.create_column("Note", DataType::Text)?;
    database.add_constraint(&table_id, Constraint::NotNull { column: count })?;
    let table = database.get_table_mut(&table_id)?;
    let row = table.add_row([(count, Value::Int(7))])?;
    let warning = Cell::Warning {
        value: Value::Null,
        warning: CellWarning::NotNull,
    };

    table.update_row(&row, [(count, Value::Null)])?;
    assert_eq!(table.get_row(&row)?.get_cell(&count), Some(&warning));

    table.update_row(&row, [(note, Value::Text("Restock".into()))])?;
    assert_eq!(table.get_row(&row)?.get_cell(&count), Some(&warning));

    table.update_row(&row, [(count, Value::Int(12))])?;
    assert_eq!(
        table.get_row(&row)?.get_cell(&count),
        Some(&Cell::Valid(Value::Int(12)))
    );
    Ok(())
}

#[test]
fn adding_not_null_rejects_unknown_table_or_column_without_changing_cells()
-> Result<(), Box<dyn std::error::Error>> {
    let mut database = Database::new("Shop")?;
    let table_id = database.create_table("Items")?;
    let table = database.get_table_mut(&table_id)?;
    let count = table.create_column("Count", DataType::Int)?;
    let existing = table.add_row([])?;

    assert!(
        database
            .add_constraint(&TableId::new(999), Constraint::NotNull { column: count })
            .is_err()
    );
    assert!(
        database
            .add_constraint(
                &table_id,
                Constraint::NotNull {
                    column: ColumnId::new(999),
                },
            )
            .is_err()
    );

    let table = database.get_table_mut(&table_id)?;
    let inserted = table.add_row([])?;
    for id in [existing, inserted] {
        assert_eq!(
            table.get_row(&id)?.get_cell(&count),
            Some(&Cell::Valid(Value::Null))
        );
    }
    Ok(())
}
