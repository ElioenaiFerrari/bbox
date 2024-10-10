use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Party {
    pub id: String,
    pub name: String,
    pub description: String,
    pub acronym: String,
}

impl Party {
    pub fn build(name: String, description: String, acronym: String) -> Party {
        Party {
            id: Uuid::now_v7().to_string(),
            name,
            description,
            acronym,
        }
    }
    pub async fn create<'a>(&self, conn: &'a SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO parties (id, name, description, acronym)
            VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(&self.id)
        .bind(&self.name)
        .bind(&self.description)
        .bind(&self.acronym)
        .execute(conn)
        .await?;

        Ok(())
    }
}
