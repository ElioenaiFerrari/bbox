use chrono::Datelike;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row, SqlitePool};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum CandidaturePosition {
    President,
    VicePresident,
    Governor,
    ViceGovernor,
    Senator,
    FederalDeputy, // Representa deputados federais
    StateDeputy,   // Representa deputados estaduais
    Mayor,
    ViceMayor,
    Councilor, // Substitui Councilman e Councilwoman, Alderman e Alderwoman
    Minister,  // Representa ministros
    Secretary, // Representa secretários (por exemplo, de estado)
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
    ) -> Candidature {
        let current_year = chrono::Local::now().year();
        Candidature {
            id: Uuid::now_v7().to_string(),
            party_id,
            candidate_id,
            code,
            position,
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
    ) -> Result<Vec<Candidature>, sqlx::Error> {
        let current_year = chrono::Local::now().year();
        let rows = sqlx::query(
            r#"
            SELECT
                id,
                party_id,
                candidate_id,
                code,
                position,
                year
            FROM
                candidatures
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
            let position: String = row.get(4);
            let c = Candidature {
                id: row.get(0),
                party_id: row.get(1),
                candidate_id: row.get(2),
                code: row.get(3),
                position: CandidaturePosition::from(position),
                year: row.get(5),
            };
            cs.push(c);
        }

        Ok(cs)
    }
}
