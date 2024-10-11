use actix_cors::Cors;
use actix_web::{
    get, rt,
    web::{self, Data, Query},
    App, HttpRequest, HttpResponse, HttpServer,
};
use actix_ws::AggregatedMessage;
use bbox::{Candidate, Candidature, CandidaturePosition, Party, Vote, Voter};
use chrono::Datelike;
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
    #[validate(length(min = 1))]
    pub candidature_position: String,
}

#[derive(Debug, Deserialize, Clone)]
struct GetCandidatureQuery {
    pub position: CandidaturePosition,
}

#[get("/candidatures")]
async fn get_candidatures(
    state: Data<State>,
    query: Query<GetCandidatureQuery>,
) -> Result<HttpResponse, actix_web::Error> {
    let candidatures = Candidature::list(&state.conn, query.position.clone())
        .await
        .unwrap();
    Ok(HttpResponse::Ok().json(candidatures))
}

#[derive(Debug, Deserialize)]
struct VoteQuery {
    pub candidature_position: CandidaturePosition,
}

#[get("/votes")]
async fn get_votes(
    state: Data<State>,
    query: Query<VoteQuery>,
) -> Result<HttpResponse, actix_web::Error> {
    let votes = Vote::list(&state.conn, query.candidature_position.clone())
        .await
        .unwrap();
    Ok(HttpResponse::Ok().json(votes))
}

#[get("/ws")]
async fn ws(
    req: HttpRequest,
    stream: web::Payload,
    state: Data<State>,
) -> Result<HttpResponse, actix_web::Error> {
    let (res, mut session, stream) = actix_ws::handle(&req, stream)?;

    // let candidatures = Candidature::list(&state.conn).await.unwrap();
    // let value = serde_json::to_string(&candidatures).unwrap();
    // session.text(value).await.unwrap();

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
                    let vote_request = match serde_json::from_str::<VoteRequest>(&text) {
                        Ok(vote_request) => vote_request,
                        Err(reason) => {
                            session.text(reason.to_string()).await.unwrap();
                            continue;
                        }
                    };

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
                        CandidaturePosition::from(vote_request.candidature_position),
                    )
                    .await
                    {
                        Ok(vote) => match vote.create(&state.conn).await {
                            Ok(_) => {
                                session.text("vote created".to_string()).await.unwrap();
                            }
                            Err(reason) => {
                                session.text(reason.to_string()).await.unwrap();
                                continue;
                            }
                        },
                        Err(reason) => {
                            session.text(reason.to_string()).await.unwrap();
                            continue;
                        }
                    }
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

    let party1 = Party {
        id: Uuid::now_v7().to_string(),
        name: "Partido Social Democrático".to_string(),
        description: "Partido Social Democrático".to_string(),
        acronym: "PSD".to_string(),
    };

    let party2 = Party {
        id: Uuid::now_v7().to_string(),
        name: "Partido Comunista Brasileiro".to_string(),
        description: "Partido Comunista Brasileiro".to_string(),
        acronym: "PCB".to_string(),
    };

    if let Err(reason) = party1.create(&conn).await {
        println!("error on create party: {}", reason);
    }

    if let Err(reason) = party2.create(&conn).await {
        println!("error on create party: {}", reason);
    }

    let candidate1 = Candidate {
        id: Uuid::now_v7().to_string(),
        first_name: "João".to_string(),
        last_name: "Silva".to_string(),
    };

    let candidate2 = Candidate {
        id: Uuid::now_v7().to_string(),
        first_name: "Maria".to_string(),
        last_name: "Silva".to_string(),
    };

    if let Err(reason) = candidate1.create(&conn).await {
        println!("error on create candidate: {}", reason);
    }

    if let Err(reason) = candidate2.create(&conn).await {
        println!("error on create candidate: {}", reason);
    }

    // use rng
    let candidature1 = Candidature {
        id: Uuid::now_v7().to_string(),
        party_id: party1.id,
        candidate_id: candidate1.id,
        image_url: "https://media.gazetadopovo.com.br/2024/07/23194251/Jair-Bolsonaro-Arquivo-Carolina-Antunes-PR-960x540.jpg".to_string(),
        code: generate_random_string(8),
        position: CandidaturePosition::President,
        year: chrono::Utc::now().year(),
    };

    let candidature2 = Candidature {
        id: Uuid::now_v7().to_string(),
        party_id: party2.id,
        candidate_id: candidate2.id,
        image_url: "https://static.poder360.com.br/2024/10/lula-entrevista-fortaleza-848x477.png"
            .to_string(),
        code: generate_random_string(8),
        position: CandidaturePosition::President,
        year: chrono::Utc::now().year(),
    };

    if let Err(reason) = candidature1.create(&conn).await {
        println!("error on create candidature: {}", reason);
    }

    if let Err(reason) = candidature2.create(&conn).await {
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

    // let vote = Vote::build(&conn, voter.id, candidature.code).await?;

    // vote.create(&conn).await?;

    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    .allow_any_header()
                    .allow_any_method()
                    .allow_any_origin(),
            )
            .app_data(Data::new(State { conn: conn.clone() }))
            .service(ws)
            .service(
                web::scope("/api/v1")
                    .service(get_candidatures)
                    .service(get_votes),
            )
    })
    .bind(("0.0.0.0", 4000))?
    .run()
    .await
}
