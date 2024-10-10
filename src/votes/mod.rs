use std::env;

use chrono::Datelike;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::{Candidature, CandidaturePosition};

#[derive(Debug, Serialize, Deserialize)]
pub struct Vote {
    pub id: String,
    pub voter_id: String,
    pub candidature_id: String,
    pub candidature_position: CandidaturePosition,
    pub hash: String,
    pub previous_hash: String,
    pub year: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl Vote {
    pub async fn build<'a>(
        conn: &'a SqlitePool,
        voter_id: String,
        code: String,
    ) -> Result<Self, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT
                id,
                party_id,
                candidate_id,
                code,
                position
            FROM
                candidatures WHERE code = ?
            "#,
        )
        .bind(&code)
        .fetch_one(conn)
        .await;

        if let Err(reason) = row {
            return Err(reason);
        }

        let row = row?;
        let position: String = row.get(4);
        let candidature = Candidature {
            id: row.get(0),
            party_id: row.get(1),
            candidate_id: row.get(2),
            code: row.get(3),
            position: CandidaturePosition::from(position),
        };
        let current_year = chrono::Utc::now().year();
        let row = sqlx::query(
            r#"
            SELECT
                id,
                voter_id,
                candidature_id,
                hash,
                year,
                previous_hash,
                created_at
            FROM
                votes
            WHERE
                voter_id = ? AND
                candidature_position = ? AND
                year = ?
            "#,
        )
        .bind(&voter_id)
        .bind(&candidature.position.to_string())
        .bind(&current_year)
        .fetch_one(conn)
        .await;

        if let Ok(row) = row {
            return Ok(Vote {
                id: row.get(0),
                voter_id: row.get(1),
                candidature_id: row.get(2),
                candidature_position: CandidaturePosition::Councilor,
                hash: row.get(3),
                year: row.get(4),
                previous_hash: row.get(5),
                created_at: row.get(6),
            });
        }

        // get last vote from database
        let row = sqlx::query(
            r#"
            SELECT
                id,
                voter_id,
                candidature_id,
                hash,
                previous_hash,
                created_at
            FROM
                votes
            ORDER BY
                created_at DESC
            LIMIT 1
            "#,
        )
        .fetch_one(conn)
        .await?;

        let last_vote = Vote {
            id: row.get(0),
            voter_id: row.get(1),
            candidature_id: row.get(2),
            candidature_position: CandidaturePosition::Councilor,
            hash: row.get(3),
            previous_hash: row.get(4),
            created_at: row.get(5),
            year: 0,
        };

        let secret_key = env::var("SECRET_KEY").expect("SECRET_KEY must be set");
        let mut mac = Hmac::<Sha256>::new_from_slice(secret_key.as_bytes()).unwrap();
        mac.update(voter_id.as_bytes());
        mac.update(candidature.id.as_bytes());
        mac.update(last_vote.hash.as_bytes());
        mac.update(last_vote.created_at.to_string().as_bytes());

        let result = mac.finalize();
        let code_bytes = result.into_bytes();
        let hash = hex::encode(code_bytes);

        let vote = Vote {
            id: Uuid::now_v7().to_string(),
            voter_id,
            candidature_id: candidature.id,
            candidature_position: candidature.position,
            hash,
            previous_hash: last_vote.hash,
            year: current_year,
            created_at: chrono::Utc::now(),
        };

        println!("{:?}", vote);

        // insert vote

        Ok(vote)
    }

    pub async fn create<'a>(&self, conn: &'a SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO votes (id, voter_id, candidature_id, candidature_position, hash, previous_hash, year, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&self.id)
        .bind(&self.voter_id)
        .bind(&self.candidature_id)
        .bind(&self.candidature_position.to_string())
        .bind(&self.hash)
        .bind(&self.previous_hash)
        .bind(&self.year)
        .bind(&self.created_at)
        .execute(conn)
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dotenv::dotenv;

    fn setup() {
        let _ = dotenv().ok();
    }

    #[test]
    fn test_build() {
        setup();
        let voter_id = "voter_id".to_string();
        let candidature_id = "candidature_id".to_string();
        let previous_hash = "previous_hash".to_string();
    }
}
