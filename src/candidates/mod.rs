use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Candidate {
    pub id: String,
    pub first_name: String,
    pub last_name: String,
}

impl Candidate {
    pub fn build(first_name: String, last_name: String) -> Candidate {
        Candidate {
            id: Uuid::now_v7().to_string(),
            first_name,
            last_name,
        }
    }
    pub async fn create<'a>(&self, conn: &'a SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO candidates (id, first_name, last_name)
            VALUES (?, ?, ?)
            "#,
        )
        .bind(&self.id)
        .bind(&self.first_name)
        .bind(&self.last_name)
        .execute(conn)
        .await?;

        Ok(())
    }
}
