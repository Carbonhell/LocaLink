use std::{
    env,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
};

use axum::{extract::State, routing::post, Json, Router, TypedHeader, headers::{Authorization, authorization::Bearer}};
use azure_data_cosmos::prelude::CollectionClient;
use serde::{Deserialize, Serialize};
use shared::{get_collection_client, get_user_document, AppError, Point, get_user_document_by_id};

// in future, this can be replaced by partners positions so that businesses can pay us to act as a meeting point
static POIS: [[f64; 2]; 1] = [
    [41.07539627931235, 14.332490805848085]
];

struct AppState {
    collection_client: CollectionClient,
    search_endpoint: String,
    search_index_name: String,
}

#[derive(Deserialize)]
struct MeetBody {
    target_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct MeetResponse {
    poi: Point,
}

async fn meet(
    auth_header: TypedHeader<Authorization<Bearer>>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<MeetBody>,
) -> Result<Json<MeetResponse>, AppError> {
    println!("Called with {}", payload.target_id);
    let mut user_document =
        get_user_document(auth_header.token(), &state.collection_client).await?;

    if user_document
        .matches
        .iter()
        .find(|matched_user| matched_user.id == payload.target_id)
        .is_none() {
        return Err(AppError::NotFoundError);
    }
    let target_user_document = get_user_document_by_id(&*payload.target_id, &state.collection_client).await?;

    if user_document.location.is_none() || target_user_document.location.is_none() {
        return Err(AppError::MissingLocationData);
    }
    // We just need a rough estimate to find a valid POI to use, no need to consider the spherical form of the earth and street vs air distance
    let user_lat_lng = user_document.location.clone().unwrap().coordinates;
    let target_lat_lng = target_user_document.location.clone().unwrap().coordinates;
    let (avg_lat, avg_lng) = ((user_lat_lng[0] + target_lat_lng[0]) / 2.,
                              (user_lat_lng[1] + target_lat_lng[1]) / 2.);

    let poi = POIS
        .iter()
        .reduce(|best_match, element| {
        if (avg_lat - element[0]) + (avg_lng - element[1]) < (avg_lat - best_match[0]) + (avg_lng - best_match[1]) {
            return element;
        }
        best_match
    })
        .unwrap();

    println!("Point chosen: {:?}", poi);
    Ok(Json(MeetResponse{poi: Point::new(poi[0], poi[1])}))
}

#[tokio::main]
async fn main() {
    let collection_client = get_collection_client().await.unwrap();
    let search_endpoint =
        std::env::var("SEARCH_ENDPOINT").expect("Set env variable SEARCH_ENDPOINT first!");
    let search_index_name =
        std::env::var("SEARCH_INDEX_NAME").expect("Set env variable SEARCH_INDEX_NAME first!");

    let shared_state = Arc::new(AppState { collection_client, search_endpoint, search_index_name });

    // build our application with a single route
    let app = Router::new()
        .route("/api/meet", post(meet))
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
