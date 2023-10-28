use axum::Json;
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{
    routing::get,
    Router,
};
use azure_data_cosmos::prelude::*;
use futures::StreamExt;
use google_oauth::AsyncClient;
use hyper::StatusCode;
use rand::Rng;
use rand::distributions::{Alphanumeric, DistString};
use uuid::Uuid;
use std::env;
use std::net::{SocketAddr,IpAddr, Ipv4Addr};
use std::sync::Arc;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct AuthBody {
    id_token: String
}

struct AppState {
    user_collection_client: CollectionClient
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct UserEntry {
    id: String,
    access_token: String
}

impl azure_data_cosmos::CosmosEntity for UserEntry {
    type Entity = String;

    fn partition_key(&self) -> Self::Entity {
        self.id.clone()
    }
}

struct AppError(anyhow::Error);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}
// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

fn generate_token() -> String {
    let mut rng = rand::thread_rng();
    
    Alphanumeric.sample_string(&mut rng, 32)
}

// async fn debug_cosmos_db(State(state): State<Arc<AppState>>) -> Result<(), AppError> {
//     let user_data = UserEntry {
//         id: Uuid::new_v4(),
//         email: "test".to_owned(),
//         access_token: "123".to_owned(),
//     };

//     println!("Creating document...");

//     let err = state.user_collection_client.create_document(user_data).is_upsert(true).await;
//     println!("{err:?}");

//     Ok(())
// }

/// Handle login via OAuth (currently only Google). The email is used as the user's identifier.
/// Registers the user if necessary, and generates a new random token to use for stateless authentication.
/// The token is stored in the user's record, which any authenticated endpoint can check when a request arrives.
async fn handle_auth(State(state): State<Arc<AppState>>, Json(payload): Json<AuthBody>) -> Result<(), AppError> {
    println!("Auth started");
    let id_token = payload.id_token;
    let client_id_key = "GOOGLE_CLIENT_ID";
    let client_id: String = env::var(client_id_key)?.parse().expect("Google client ID must be filled!");
    println!("Client id: {client_id}");
    
    let client = AsyncClient::new(client_id);

    let data = client.validate_id_token(id_token).await;
    match &data {
        Ok(data) => {
            // https://fly.io/blog/api-tokens-a-tedious-survey/ random tokens are a reasonable choice for a simple auth system like this that doesn't require policies
            println!("ok: {:?}", data);
            // TODO handle new user

            let email = data.email.to_owned().unwrap();
            // Save the new token
            let access_token = generate_token();
            let user_data = UserEntry {
                id: email,
                access_token,
            };

            state.user_collection_client.create_document(user_data).is_upsert(true).await?;
            println!("User saved.");

        },
        Err(e) => println!("{:?}", e),
    };
    // TODO return our own JWT
    Ok(())
}

async fn get_collection_client() -> azure_core::Result<CollectionClient> {
    // CosmosDB configuration
    let primary_key = std::env::var("COSMOS_PRIMARY_KEY").expect("Set env variable COSMOS_PRIMARY_KEY first!");
    let account = std::env::var("COSMOS_ACCOUNT").expect("Set env variable COSMOS_ACCOUNT first!");

    let main_db = std::env::var("COSMOS_DB").expect("Specify the main DB name first!");
    let users_collection = std::env::var("USERS_TABLE").expect("Specify the name of the users collection!");


    let authorization_token = AuthorizationToken::primary_from_base64(&primary_key)?;

    let client = CosmosClient::builder(account, authorization_token)
        .cloud_location(CloudLocation::Emulator { address: "localhost".to_owned(), port: 8081 })
        .build();

    println!("Client built");

    // Attempt creating the DB and the required collections if they're not present already
    match client.create_database(&main_db).await {
        Ok(_) => println!("{main_db} database created successfully."),
        Err(_) => println!("{main_db} database exists already.")
    }

    let database_client = client.database_client(main_db);

    match database_client.create_collection(&users_collection, "/id").await {
        Ok(_) => println!("{users_collection} collection created successfully."),
        Err(_) => println!("{users_collection} collection exists already.")
    }

    let collection_client = database_client
        .collection_client(users_collection.clone());

    Ok(collection_client)
}

#[tokio::main]
async fn main() -> azure_core::Result<()> {
    let collection_client = get_collection_client().await.unwrap();
        
    let shared_state = Arc::new(AppState { user_collection_client: collection_client });
    

    // build our application with a single route
    let app = Router::new()
        .route("/api/auth", post(handle_auth))
        .with_state(shared_state);

    let port_key = "FUNCTIONS_CUSTOMHANDLER_PORT";
    let port: u16 = match env::var(port_key) {
        Ok(val) => val.parse().expect("Custom Handler port is not a number!"),
        Err(_) => 3000,
    };

    println!("Ready!");
    // run it with hyper on localhost:3000
    axum::Server::bind(&SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port))
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}