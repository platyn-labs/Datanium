use datanium_core::database::{Cell, DataType, Database, Value};

#[test]
fn iterators_are_empty_initially_and_follow_ids_instead_of_names()
-> Result<(), Box<dyn std::error::Error>> {
    let mut db = Database::new("Shop")?;
    assert_eq!(db.tables().count(), 0);
    let first = db.create_table("Zebra")?;
    let second = db.create_table("Alpha")?;
    db.rename_table(&first, "Renamed")?;
    assert_eq!(
        db.tables().map(|table| table.id()).collect::<Vec<_>>(),
        vec![first, second]
    );
    let table = db.get_table_mut(&first)?;
    assert_eq!(table.columns().count(), 0);
    assert_eq!(table.rows().count(), 0);
    let z = table.create_column("Z", DataType::Int)?;
    let a = table.create_column("A", DataType::Text)?;
    assert_eq!(
        table
            .columns()
            .map(|column| *column.id())
            .collect::<Vec<_>>(),
        vec![z, a]
    );
    assert_eq!(
        table
            .columns()
            .map(|column| column.name())
            .collect::<Vec<_>>(),
        vec!["Z", "A"]
    );
    let r1 = table.add_row([(z, Value::Int(8))])?;
    let r2 = table.add_row([(z, Value::Int(2))])?;
    let rows = table.rows().collect::<Vec<_>>();
    assert_eq!(
        rows.iter().map(|(id, _)| *id).collect::<Vec<_>>(),
        vec![r1, r2]
    );
    assert_eq!(rows[0].1.get_cell(&z), Some(&Cell::Valid(Value::Int(8))));
    assert_eq!(rows[1].1.get_cell(&z), Some(&Cell::Valid(Value::Int(2))));
    assert_eq!(db.get_table(&second)?.columns().count(), 0);
    assert_eq!(db.get_table(&second)?.rows().count(), 0);
    Ok(())
}
