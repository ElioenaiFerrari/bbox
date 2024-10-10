use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Voter {
    pub id: String,
    pub first_name: String,
    pub last_name: String,
    pub mother_name: String,
    pub father_name: String,
    pub birth_date: String,
}

impl Voter {
    pub fn build(
        first_name: String,
        last_name: String,
        mother_name: String,
        father_name: String,
        birth_date: String,
    ) -> Voter {
        Voter {
            id: Uuid::now_v7().to_string(),
            first_name,
            last_name,
            mother_name,
            father_name,
            birth_date,
        }
    }
    pub async fn create<'a>(&self, conn: &'a SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO voters (id, first_name, last_name, mother_name, father_name, birth_date)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&self.id)
        .bind(&self.first_name)
        .bind(&self.last_name)
        .bind(&self.mother_name)
        .bind(&self.father_name)
        .bind(&self.birth_date)
        .execute(conn)
        .await?;

        Ok(())
    }
}
