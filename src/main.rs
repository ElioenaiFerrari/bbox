use bbox::{Candidate, Candidature, CandidaturePosition, Party, Vote, Voter};
use dotenv::dotenv;
use sqlx::SqlitePool;
use uuid::Uuid;

async fn establish_connection() -> Result<SqlitePool, sqlx::Error> {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqlitePool::connect(&database_url).await
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let conn = establish_connection()
        .await
        .expect("Failed to connect to database");

    let party = Party {
        id: Uuid::now_v7().to_string(),
        name: "Partido da Causa Operária".to_string(),
        description: "Partido da Causa Operária".to_string(),
        acronym: "PCO".to_string(),
    };
    if let Err(reason) = party.create(&conn).await {
        println!("error on create party: {}", reason);
    }

    let candidate = Candidate {
        id: Uuid::now_v7().to_string(),
        first_name: "José".to_string(),
        last_name: "Silva".to_string(),
    };
    if let Err(reason) = candidate.create(&conn).await {
        println!("error on create candidate: {}", reason);
    }

    let candidature = Candidature {
        id: Uuid::now_v7().to_string(),
        party_id: party.id,
        candidate_id: candidate.id,
        code: Uuid::now_v7().to_string(),
        position: CandidaturePosition::President,
    };
    if let Err(reason) = candidature.create(&conn).await {
        println!("error on create candidature: {}", reason);
    }

    let voter = Voter {
        id: Uuid::now_v7().to_string(),
        first_name: "Maria".to_string(),
        last_name: "Silva".to_string(),
        mother_name: "Ana".to_string(),
        father_name: "José".to_string(),
        birth_date: "01/01/2000".to_string(),
    };
    if let Err(reason) = voter.create(&conn).await {
        println!("error on create voter: {}", reason);
    }

    let vote = Vote::build(&conn, voter.id, candidature.code).await?;

    vote.create(&conn).await?;

    Ok(())
}
