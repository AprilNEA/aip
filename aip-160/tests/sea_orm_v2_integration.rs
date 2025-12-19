#![cfg(feature = "sea-orm-v2")]
use aip_160::{ToSeaOrmConditionV2, parse_filter};
use sea_orm_v2::{
    Database, DatabaseConnection, DbBackend, DbErr, QueryOrder, QuerySelect, QueryTrait, Set,
    entity::prelude::*,
};

// Define test entity
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "users", crate = "sea_orm_v2")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub email: String,
    pub age: i32,
    pub active: bool,
    pub score: f64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

// Define UUID test entity for PostgreSQL-specific tests
mod uuid_entity {
    use sea_orm_v2::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "documents", crate = "sea_orm_v2")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub id: Uuid,
        pub title: String,
        pub created_at: DateTimeWithTimeZone,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

// Define entity with enum column for PostgreSQL-specific tests
mod enum_entity {
    use sea_orm_v2::entity::prelude::*;

    #[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
    #[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "status_type")]
    pub enum Status {
        #[sea_orm(string_value = "pending")]
        Pending,
        #[sea_orm(string_value = "active")]
        Active,
        #[sea_orm(string_value = "completed")]
        Completed,
    }

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "tasks", crate = "sea_orm_v2")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub name: String,
        pub status: Status,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

// Helper function to setup test database
async fn setup_test_db() -> Result<DatabaseConnection, DbErr> {
    let db = Database::connect("sqlite::memory:").await?;

    // Create table
    let create_table_sql = r#"
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            email TEXT NOT NULL,
            age INTEGER NOT NULL,
            active BOOLEAN NOT NULL,
            score REAL NOT NULL
        )
    "#;

    db.execute_unprepared(create_table_sql).await?;

    Ok(db)
}

// Helper function to seed test data
async fn seed_data(db: &DatabaseConnection) -> Result<(), DbErr> {
    let users = vec![
        (1, "Alice", "alice@example.com", 25, true, 95.5),
        (2, "Bob", "bob@example.com", 30, true, 87.3),
        (3, "Charlie", "charlie@test.com", 22, false, 76.8),
        (4, "David", "david@example.com", 35, true, 91.2),
        (5, "Eve", "eve@test.com", 28, false, 82.4),
        (6, "Frank", "frank@example.com", 42, true, 88.9),
    ];

    for (id, name, email, age, active, score) in users {
        let user = ActiveModel {
            id: Set(id),
            name: Set(name.to_string()),
            email: Set(email.to_string()),
            age: Set(age),
            active: Set(active),
            score: Set(score),
        };
        Entity::insert(user).exec(db).await?;
    }

    Ok(())
}

#[tokio::test]
async fn test_simple_equality_filter() -> Result<(), Box<dyn std::error::Error>> {
    let db = setup_test_db().await?;
    seed_data(&db).await?;

    let filter = parse_filter("name = \"Alice\"")?;
    let condition = filter.to_condition::<Entity>()?;

    let users = Entity::find().filter(condition).all(&db).await?;

    assert_eq!(users.len(), 1);
    assert_eq!(users[0].name, "Alice");
    assert_eq!(users[0].age, 25);

    Ok(())
}

#[tokio::test]
async fn test_number_comparison() -> Result<(), Box<dyn std::error::Error>> {
    let db = setup_test_db().await?;
    seed_data(&db).await?;

    let filter = parse_filter("age > 30")?;
    let condition = filter.to_condition::<Entity>()?;

    let users = Entity::find().filter(condition).all(&db).await?;

    assert_eq!(users.len(), 2); // David (35) and Frank (42)
    assert!(users.iter().all(|u| u.age > 30));

    Ok(())
}

#[tokio::test]
async fn test_and_expression() -> Result<(), Box<dyn std::error::Error>> {
    let db = setup_test_db().await?;
    seed_data(&db).await?;

    let filter = parse_filter("age > 25 AND active = true")?;
    let condition = filter.to_condition::<Entity>()?;

    let users = Entity::find().filter(condition).all(&db).await?;

    assert_eq!(users.len(), 3); // Bob, David, Frank
    assert!(users.iter().all(|u| u.age > 25 && u.active));

    Ok(())
}

#[tokio::test]
async fn test_or_expression() -> Result<(), Box<dyn std::error::Error>> {
    let db = setup_test_db().await?;
    seed_data(&db).await?;

    let filter = parse_filter("name = \"Alice\" OR name = \"Bob\"")?;
    let condition = filter.to_condition::<Entity>()?;

    let users = Entity::find().filter(condition).all(&db).await?;

    assert_eq!(users.len(), 2);
    assert!(users.iter().any(|u| u.name == "Alice"));
    assert!(users.iter().any(|u| u.name == "Bob"));

    Ok(())
}

// Test that UUID columns generate proper CAST in SQL
#[test]
fn test_uuid_cast_in_sql() -> Result<(), Box<dyn std::error::Error>> {
    // Test equality
    let filter = parse_filter("id = \"550e8400-e29b-41d4-a716-446655440000\"")?;
    let condition = filter.to_condition::<uuid_entity::Entity>()?;
    let query = uuid_entity::Entity::find().filter(condition);
    let sql = query.build(DbBackend::Postgres).to_string();

    assert!(
        sql.contains("CAST"),
        "SQL should contain CAST for UUID: {}",
        sql
    );
    assert!(
        sql.contains("uuid"),
        "SQL should cast to uuid type: {}",
        sql
    );

    Ok(())
}

// Test that timestamp columns generate proper CAST in SQL
#[test]
fn test_timestamp_cast_in_sql() -> Result<(), Box<dyn std::error::Error>> {
    let filter = parse_filter("created_at > \"2024-01-01T00:00:00Z\"")?;
    let condition = filter.to_condition::<uuid_entity::Entity>()?;
    let query = uuid_entity::Entity::find().filter(condition);
    let sql = query.build(DbBackend::Postgres).to_string();

    assert!(
        sql.contains("CAST"),
        "SQL should contain CAST for timestamp: {}",
        sql
    );
    assert!(
        sql.contains("timestamptz"),
        "SQL should cast to timestamptz type: {}",
        sql
    );

    Ok(())
}

// Test that enum columns generate proper CAST in SQL
#[test]
fn test_enum_cast_in_sql() -> Result<(), Box<dyn std::error::Error>> {
    let filter = parse_filter("status = \"active\"")?;
    let condition = filter.to_condition::<enum_entity::Entity>()?;
    let query = enum_entity::Entity::find().filter(condition);
    let sql = query.build(DbBackend::Postgres).to_string();

    assert!(
        sql.contains("CAST"),
        "SQL should contain CAST for enum: {}",
        sql
    );
    assert!(
        sql.contains("status_type"),
        "SQL should cast to enum type name: {}",
        sql
    );

    Ok(())
}
