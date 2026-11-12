use super::*;
use crate::database::Database;

#[test]
fn primary_key_is_stored_as_one_rule_without_synthetic_not_null_or_unique_definitions()
-> Result<(), Box<dyn std::error::Error>> {
    let mut database = Database::new("Shop")?;
    let table_id = database.create_table("Items")?;
    let key = database
        .get_table_mut(&table_id)?
        .create_column("Code", DataType::Int)?;

    for _ in 0..2 {
        database.add_constraint(&table_id, Constraint::PrimaryKey { columns: vec![key] })?;
    }

    let table = database.get_table(&table_id)?;
    assert!(matches!(
        table.constraints.as_slice(),
        [Constraint::PrimaryKey { columns }] if columns == &[key]
    ));
    Ok(())
}

#[test]
fn explicit_not_null_is_stored_independently_of_a_primary_key()
-> Result<(), Box<dyn std::error::Error>> {
    let mut database = Database::new("Shop")?;
    let table_id = database.create_table("Items")?;
    let key = database
        .get_table_mut(&table_id)?
        .create_column("Code", DataType::Int)?;
    database.add_constraint(&table_id, Constraint::PrimaryKey { columns: vec![key] })?;

    for _ in 0..2 {
        database.add_constraint(&table_id, Constraint::NotNull { column: key })?;
    }

    let table = database.get_table(&table_id)?;
    assert_eq!(table.constraints.len(), 2);
    assert!(table.constraints.iter().any(|constraint| {
        matches!(constraint, Constraint::NotNull { column } if *column == key)
    }));
    assert!(table.constraints.iter().any(|constraint| {
        matches!(constraint, Constraint::PrimaryKey { columns } if columns == &[key])
    }));
    Ok(())
}
