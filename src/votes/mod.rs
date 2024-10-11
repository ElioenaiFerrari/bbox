use std::{collections::HashMap, env};

use anyhow::anyhow;
use chrono::Datelike;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
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
        candidature_position: CandidaturePosition,
    ) -> Result<Self, anyhow::Error> {
        let row = sqlx::query(
            r#"
            SELECT
                id,
                party_id,
                candidate_id,
                code,
                position,
                year
            FROM
                candidatures WHERE code = ? AND
                position = ?
            "#,
        )
        .bind(&code)
        .bind(&candidature_position.to_string())
        .fetch_one(conn)
        .await;

        if let Err(reason) = row {
            return Err(anyhow!("candidature not found: {}", reason));
        }

        let row = row?;
        let candidature = Candidature {
            id: row.get(0),
            party_id: row.get(1),
            candidate_id: row.get(2),
            code: row.get(3),
            position: candidature_position,
            year: row.get(5),
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

        if let Ok(_) = row {
            return Err(anyhow!(
                "voter already voted for this position in this year"
            ));
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

    // get group by candidate name and count votes
    pub async fn list<'a>(
        conn: &'a SqlitePool,
        candidature_position: CandidaturePosition,
    ) -> Result<Vec<Value>, sqlx::Error> {
        let current_year = chrono::Utc::now().year();
        let rows = sqlx::query(
            r#"
            SELECT
                c.first_name,
                c.last_name,
                c.id,
                ca.code,
                ca.position,
                p.name,
                p.acronym,
                p.id,
                COUNT(v.id) as votes
            FROM
                votes v
            JOIN
                candidatures ca ON v.candidature_id = ca.id
            JOIN
                candidates c ON ca.candidate_id = c.id
            JOIN
                parties p ON ca.party_id = p.id
            WHERE
                v.candidature_position = ? AND
                v.year = ?
            GROUP BY
                ca.id
            "#,
        )
        .bind(candidature_position.to_string())
        .bind(current_year)
        .fetch_all(conn)
        .await?;

        let mut cs = Vec::new();

        for row in rows {
            let value = json!(
                {
                    "candidate": {
                        "first_name": row.get::<String, usize>(0),
                        "last_name": row.get::<String, usize>(1),
                        "id": row.get::<String, usize>(2),
                    },
                    "candidature": {
                        "code": row.get::<String, usize>(3),
                        "position": row.get::<String, usize>(4),
                    },
                    "party": {
                        "name": row.get::<String, usize>(5),
                        "acronym": row.get::<String, usize>(6),
                        "id": row.get::<String, usize>(7),
                    },
                    "votes": row.get::<i64, usize>(8),
                }
            );

            cs.push(value);
        }

        Ok(cs)
    }
}
