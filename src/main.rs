use actix_web::{
    rt,
    web::{self, Data},
    App, HttpRequest, HttpResponse, HttpServer,
};
use actix_ws::AggregatedMessage;
use bbox::{Candidate, Candidature, CandidaturePosition, Party, Vote, Voter};
use dotenv::dotenv;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde::Deserialize;
use sqlx::SqlitePool;
use tokio_stream::StreamExt;
use uuid::Uuid;
use validator::{Validate, ValidationError};

async fn establish_connection() -> Result<SqlitePool, sqlx::Error> {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqlitePool::connect(&database_url).await
}

fn validate_uuid(uuid: &str) -> Result<(), ValidationError> {
    match Uuid::parse_str(uuid) {
        Ok(_) => Ok(()),
        Err(_) => Err(ValidationError::new("invalid uuid")),
    }
}

#[derive(Debug, Validate, Deserialize)]
struct VoteRequest {
    #[validate(custom(function = "validate_uuid"))]
    pub voter_id: String,
    #[validate(length(min = 1))]
    pub candidature_code: String,
}

async fn ws(
    req: HttpRequest,
    stream: web::Payload,
    state: Data<State>,
) -> Result<HttpResponse, actix_web::Error> {
    let (res, mut session, stream) = actix_ws::handle(&req, stream)?;

    let candidatures = Candidature::list(&state.conn).await.unwrap();
    let value = serde_json::to_string(&candidatures).unwrap();
    session.text(value).await.unwrap();

    let mut stream = stream
        .aggregate_continuations()
        // aggregate continuation frames up to 1MiB
        .max_continuation_size(2_usize.pow(20));

    // start task but don't wait for it
    rt::spawn(async move {
        // receive messages from websocket
        while let Some(msg) = stream.next().await {
            match msg {
                Ok(AggregatedMessage::Text(text)) => {
                    let vote_request = serde_json::from_str::<VoteRequest>(&text).unwrap();
                    match vote_request.validate() {
                        Ok(_) => {}
                        Err(reason) => {
                            session.text(reason.to_string()).await.unwrap();
                            continue;
                        }
                    }
                    match Vote::build(
                        &state.conn,
                        vote_request.voter_id,
                        vote_request.candidature_code,
                    )
                    .await
                    {
                        Ok(vote) => match vote.create(&state.conn).await {
                            Ok(_) => {
                                println!("vote created");
                            }
                            Err(reason) => {
                                println!("error on create vote: {}", reason);
                                continue;
                            }
                        },
                        Err(reason) => {
                            println!("error on create vote: {}", reason);
                            continue;
                        }
                    }
                    session.text(text).await.unwrap();
                }

                Ok(AggregatedMessage::Binary(bin)) => {
                    // echo binary message
                    session.binary(bin).await.unwrap();
                }

                Ok(AggregatedMessage::Ping(msg)) => {
                    // respond to PING frame with PONG frame
                    session.pong(&msg).await.unwrap();
                }

                _ => {}
            }
        }
    });

    // respond immediately with response connected to WS session
    Ok(res)
}

#[derive(Debug, Clone)]
struct State {
    pub conn: SqlitePool,
}

fn generate_random_string(length: usize) -> String {
    let mut rng = thread_rng();
    (0..length)
        .map(|_| rng.sample(Alphanumeric)) // Generate random alphanumeric characters
        .map(char::from) // Convert bytes to char
        .collect() // Collect into a String
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
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

    // use rng
    let candidature = Candidature {
        id: Uuid::now_v7().to_string(),
        party_id: party.id,
        candidate_id: candidate.id,
        code: generate_random_string(8),
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

    println!(
        "voter created: {}, candidature created: {}",
        voter.id, candidature.code
    );

    // let vote = Vote::build(&conn, voter.id, candidature.code).await?;

    // vote.create(&conn).await?;

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(State { conn: conn.clone() }))
            .route("/ws", web::get().to(ws))
    })
    .bind(("0.0.0.0", 4000))?
    .run()
    .await
}
