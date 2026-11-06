use crate::database::DatabaseError;

#[test]
fn database_errors_expose_messages_through_standard_error_trait() {
    let cases = [
        (
            DatabaseError::TableIdExhausted,
            "table ID counter is exhausted",
        ),
        (
            DatabaseError::TableAlreadyExists,
            "a table with this name already exists",
        ),
        (DatabaseError::InvalidDatabaseName, "invalid database name"),
        (DatabaseError::InvalidTableName, "invalid table name"),
        (DatabaseError::TableNotFound, "table not found"),
    ];

    for (error, expected_message) in cases {
        let standard_error: &dyn std::error::Error = &error;

        assert_eq!(standard_error.to_string(), expected_message);
        assert!(standard_error.source().is_none());
    }
}
