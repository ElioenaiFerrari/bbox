use chrono::Datelike;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{FromRow, Row, SqlitePool};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum CandidaturePosition {
    #[serde(rename = "Presidente")]
    President,
    #[serde(rename = "Vice-Presidente")]
    VicePresident,
    #[serde(rename = "Governador")]
    Governor,
    #[serde(rename = "Vice-Governador")]
    ViceGovernor,
    #[serde(rename = "Senador")]
    Senator,
    #[serde(rename = "Deputado Federal")]
    FederalDeputy,
    #[serde(rename = "Deputado Estadual")]
    StateDeputy,
    #[serde(rename = "Prefeito")]
    Mayor,
    #[serde(rename = "Vice-Prefeito")]
    ViceMayor,
    #[serde(rename = "Vereador")]
    Councilor,
    #[serde(rename = "Ministro")]
    Minister,
    #[serde(rename = "Secretário")]
    Secretary,
}

impl ToString for CandidaturePosition {
    fn to_string(&self) -> String {
        match self {
            CandidaturePosition::President => "Presidente".to_string(),
            CandidaturePosition::VicePresident => "Vice-Presidente".to_string(),
            CandidaturePosition::Governor => "Governador".to_string(),
            CandidaturePosition::ViceGovernor => "Vice-Governador".to_string(),
            CandidaturePosition::Senator => "Senador".to_string(),
            CandidaturePosition::FederalDeputy => "Deputado Federal".to_string(),
            CandidaturePosition::StateDeputy => "Deputado Estadual".to_string(),
            CandidaturePosition::Mayor => "Prefeito".to_string(),
            CandidaturePosition::ViceMayor => "Vice-Prefeito".to_string(),
            CandidaturePosition::Councilor => "Vereador".to_string(),
            CandidaturePosition::Minister => "Ministro".to_string(),
            CandidaturePosition::Secretary => "Secretário".to_string(),
        }
    }
}

impl From<String> for CandidaturePosition {
    fn from(position: String) -> CandidaturePosition {
        match position.as_str() {
            "Presidente" => CandidaturePosition::President,
            "Vice-Presidente" => CandidaturePosition::VicePresident,
            "Governador" => CandidaturePosition::Governor,
            "Vice-Governador" => CandidaturePosition::ViceGovernor,
            "Senador" => CandidaturePosition::Senator,
            "Deputado Federal" => CandidaturePosition::FederalDeputy,
            "Deputado Estadual" => CandidaturePosition::StateDeputy,
            "Prefeito" => CandidaturePosition::Mayor,
            "Vice-Prefeito" => CandidaturePosition::ViceMayor,
            "Vereador" => CandidaturePosition::Councilor,
            "Ministro" => CandidaturePosition::Minister,
            "Secretário" => CandidaturePosition::Secretary,
            _ => CandidaturePosition::Councilor,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Candidature {
    pub id: String,
    pub party_id: String,
    pub candidate_id: String,
    pub image_url: String,
    pub code: String,
    pub year: i32,
    pub position: CandidaturePosition,
}

impl Candidature {
    pub fn build(
        party_id: String,
        candidate_id: String,
        code: String,
        position: CandidaturePosition,
        image_url: String,
    ) -> Candidature {
        let current_year = chrono::Local::now().year();
        Candidature {
            id: Uuid::now_v7().to_string(),
            party_id,
            candidate_id,
            code,
            position,
            image_url,
            year: current_year,
        }
    }
    pub async fn create<'a>(&self, conn: &'a SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO candidatures (id, party_id, candidate_id, code, position, year)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&self.id)
        .bind(&self.party_id)
        .bind(&self.candidate_id)
        .bind(&self.code)
        .bind(&self.position.to_string())
        .bind(&self.year)
        .execute(conn)
        .await?;

        Ok(())
    }

    pub async fn list<'a>(
        conn: &'a SqlitePool,
        position: CandidaturePosition,
    ) -> Result<Vec<Value>, sqlx::Error> {
        let current_year = chrono::Local::now().year();
        let rows = sqlx::query(
            r#"
            SELECT
                cu.id,
                cu.party_id,
                cu.candidate_id,
                cu.code,
                cu.position,
                cu.year,
                cu.image_url,
                p.name,
                p.acronym,
                p.name,
                ca.first_name,
                ca.last_name
            FROM
                candidatures cu
            JOIN
                parties p ON p.id = cu.party_id
            JOIN
                candidates ca ON ca.id = cu.candidate_id
            WHERE
                position = ? AND
                year = ?
            "#,
        )
        .bind(position.to_string())
        .bind(current_year)
        .fetch_all(conn)
        .await?;

        let mut cs = Vec::new();
        for row in rows {
            let value = json!({
                "candidature": {
                    "id": row.get::<String, usize>(0),
                    "party_id": row.get::<String, usize>(1),
                    "candidate_id": row.get::<String, usize>(2),
                    "code": row.get::<String, usize>(3),
                    "position": row.get::<String, usize>(4),
                    "year": row.get::<i32, _>(5),
                    "image_url": row.get::<String, usize>(6),
                },
                "party": {
                    "name": row.get::<String, usize>(7),
                    "acronym": row.get::<String, usize>(8),
                },
                "candidate": {
                    "first_name": row.get::<String, usize>(9),
                    "last_name": row.get::<String, usize>(10),

                },
            });

            cs.push(value);
        }

        Ok(cs)
    }
}
