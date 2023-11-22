use std::{
    collections::HashMap,
    env, fs,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
};

use axum::{
    extract::State,
    routing::post,
    Json, Router, TypedHeader, headers::{authorization::Bearer, Authorization},
};
use azure_data_cosmos::prelude::{
    CollectionClient,
};
use reqwest;
use serde::{Deserialize};
use shared::{get_collection_client, get_user_document, AppError, index_documents, IndexAction, UserSearchData};

struct AppState {
    search_endpoint: String,
    search_index_name: String,
    search_admin_key: String,
    openai_api_key: String,
    openai_base_url: String,
    openai_model: String,
    collection_client: CollectionClient,
}

#[derive(Deserialize, Debug)]
struct OpenAIResponse {
    object: String,
    data: Vec<OpenAIEmbeddings>,
    model: String,
    usage: OpenAIUsage,
}

#[derive(Deserialize, Debug)]
struct OpenAIEmbeddings {
    object: String,
    index: u32,
    embedding: Vec<f64>,
}

#[derive(Deserialize, Debug)]
struct OpenAIUsage {
    prompt_tokens: u32,
    total_tokens: u32,
}

#[derive(Deserialize)]
struct TextEmbeddingsBody {
    description: String,
}

/// Handler for text based data (e.g. a brief description written by the user of his interests)
async fn generate_embeddings(
    auth_header: TypedHeader<Authorization<Bearer>>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<TextEmbeddingsBody>,
) -> Result<Json<()>, AppError> {
    println!("Generate Embeddings start");
    let mut user_document = get_user_document(auth_header.token(), &state.collection_client).await?;

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

    // let openai_response = res.json::<OpenAIResponse>().await?;
    let file = fs::File::open("example_openai_response.json").expect("file should open read only");
    let openai_response: OpenAIResponse =
        serde_json::from_reader(file).expect("file should be proper JSON");

    let embeddings = &openai_response.data.first().unwrap().embedding;

    //let user_description = UserEntry { description: payload.description.clone(), description_embeddings: embeddings.clone()};

    println!("Body:\n{:?}", openai_response);

    // Save the data in the DB, both original (for user facing purposes) and vector data (in the indexed column)

    user_document.description = Some(payload.description.clone());
    user_document.description_embeddings = Some(embeddings.clone());

    state
        .collection_client
        .document_client(user_document.id.clone(), &user_document.id)?
        .replace_document(user_document.clone())
        .await?;

    let res = index_documents(&state.search_endpoint, &state.search_index_name, &state.search_admin_key, &[
        IndexAction {
            action_type: shared::IndexActionType::MergeOrUpload,
            user_document: UserSearchData::from(user_document),
        }
    ]).await;
    println!("Res: {:?}", res);
    Ok(Json(()))
}

#[tokio::main]
async fn main() {
    let collection_client = get_collection_client().await.unwrap();

    let search_endpoint =
        std::env::var("SEARCH_ENDPOINT").expect("Set env variable SEARCH_ENDPOINT first!");
    let search_index_name =
        std::env::var("SEARCH_INDEX_NAME").expect("Set env variable SEARCH_INDEX_NAME first!");
    let search_admin_key =
        std::env::var("SEARCH_ADMIN_KEY").expect("Set env variable SEARCH_ADMIN_KEY first!");
    let openai_api_key =
        std::env::var("OPENAI_API_KEY").expect("Set env variable OPENAI_API_KEY first!");
    let openai_base_url =
        std::env::var("OPENAI_BASE_URL").expect("Set env variable OPENAI_BASE_URL first!");
    let openai_model = std::env::var("OPENAI_MODEL").expect("Set env variable OPENAI_MODEL first!");

    let shared_state = Arc::new(AppState {
        search_endpoint,
        search_index_name,
        search_admin_key,
        openai_api_key,
        openai_base_url,
        openai_model,
        collection_client,
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
