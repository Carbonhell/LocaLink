use std::{net::{SocketAddr, IpAddr, Ipv4Addr}, env, collections::HashMap, sync::Arc, fs};

use axum::{Router, Json, routing::post, response::{IntoResponse, Response}, extract::State};
use azure_data_cosmos::prelude::{CollectionClient, CosmosClient, AuthorizationToken, CloudLocation, Query, Param};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use reqwest;
use futures::StreamExt;

struct AppState {
    openai_api_key: String,
    openai_base_url: String,
    openai_model: String,
    collection_client: CollectionClient
}

#[derive(Deserialize, Debug)]
struct OpenAIResponse {
    object: String,
    data: Vec<OpenAIEmbeddings>,
    model: String,
    usage: OpenAIUsage
}

#[derive(Deserialize, Debug)]
struct OpenAIEmbeddings {
    object: String,
    index: u32,
    embedding: Vec<f64>
}

#[derive(Deserialize, Debug)]
struct OpenAIUsage {
    prompt_tokens: u32,
    total_tokens: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct UserEntry {
    id: String,
    access_token: String,
    description: Option<String>,
    description_embeddings: Option<Vec<f64>>,
}

// TODO make shared
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
// ---

#[derive(Deserialize)]
struct TextEmbeddingsBody {
    description: String
}

// TODO add auth!
/// Handler for text based data (e.g. a brief description written by the user of his interests)
async fn generate_embeddings(State(state): State<Arc<AppState>>, Json(payload): Json<TextEmbeddingsBody>) -> Result<(), AppError> {
    println!("Call start");
    //convert data to embeddings through OpenAI Ada model
    let mut map = HashMap::new();
    map.insert("input", payload.description.clone());
    map.insert("model", state.openai_model.clone());

    // let client = reqwest::Client::new();
    // println!("Posting {}, with token {} and input {}", state.openai_base_url, state.openai_api_key, payload.description);
    // let res = client.post(&state.openai_base_url)
    //     //.query(&[("api-version", "API_VERSION")]) only used in Azure OpenAI
    //     .header("Authorization", format!("Bearer {}", state.openai_api_key))
    //     .header("Content-Type", "application/json")
    //     .json(&map)
    //     .send()
    //     .await?;
    // println!("Status: {}", res.status());
    // println!("Headers:\n{:#?}", res.headers());

    // let openai_response = res.json::<OpenAIResponse>().await?; // TODO figure out response structure... https://learn.microsoft.com/en-us/azure/ai-services/openai/reference#embeddings
    let file = fs::File::open("example_openai_response.json").expect("file should open read only");
    let openai_response: OpenAIResponse = serde_json::from_reader(file).expect("file should be proper JSON");

    let embeddings = &openai_response.data.first().unwrap().embedding;

    //let user_description = UserEntry { description: payload.description.clone(), description_embeddings: embeddings.clone()};

    println!("Body:\n{:?}", openai_response);

    // Save the data in the DB, both original (for user facing purposes) and vector data (in the indexed column)
    let query = state.collection_client.query_documents(
        Query::with_params(
            "SELECT * FROM users AS u WHERE u.access_token = @token".to_owned(),
            vec![Param::new("@token".into(), "3qaPDBow6OnLgwhDsZeN03SuSE9pdWdx")]
        )
    )
        .query_cross_partition(true) //TODO deep dive and figure out how to do in partition queries for this case
        .max_item_count(1);
    let mut stream = query.into_stream::<UserEntry>();
    // The stream's needed because the response is pageable, in case of a large amount of documents.
    if let Some(query_response) = stream.next().await {
        println!("User document found, {:?}", query_response);
        // In this page, the documents are under results
        let query_response = query_response?;
        let (user_document, _) = query_response.results.first().unwrap();
        let mut user_document = user_document.clone();
        user_document.description = Some(payload.description.clone());
        user_document.description_embeddings = Some(embeddings.clone());

        state.collection_client.document_client(user_document.id.clone(), &user_document.id)?
            .replace_document(user_document).await?;

    }

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
async fn main() {
    let collection_client = get_collection_client().await.unwrap();

    let openai_api_key = std::env::var("OPENAI_API_KEY").expect("Set env variable OPENAI_API_KEY first!");
    let openai_base_url = std::env::var("OPENAI_BASE_URL").expect("Set env variable OPENAI_BASE_URL first!");
    let openai_model = std::env::var("OPENAI_MODEL").expect("Set env variable OPENAI_MODEL first!");

    let shared_state = Arc::new(AppState {
        openai_api_key,
        openai_base_url,
        openai_model,
        collection_client
    });

    // build our application with a single route
    let app = Router::new()
        .route("/api/generate_embeddings", post(generate_embeddings))
        .with_state(shared_state);

    let port_key = "FUNCTIONS_CUSTOMHANDLER_PORT";
    let port: u16 = match env::var(port_key) {
        Ok(val) => val.parse().expect("Custom Handler port is not a number!"),
        Err(_) => 3000,
    };

    // run it with hyper on localhost:3000
    axum::Server::bind(&SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port))
        .serve(app.into_make_service())
        .await
        .unwrap();
}