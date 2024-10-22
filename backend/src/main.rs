#[macro_use]
extern crate rocket;

use qr_builder::create_qr_for_id;
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::{http::Status, response::status::Custom, State};
use rocket_cors::{AllowedOrigins, CorsOptions};
use tokio::fs::File;
use tokio_postgres::{Client, NoTls};

mod qr_builder;

#[derive(Serialize, Deserialize, Clone)]
struct Vehicle {
    id: Option<i32>,
    vehicle_type: String,
    manufacturer: String,
    model: String,
    price: String,
    data: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct QrCodeImage {
    char_image: String,
}

#[post("/api/vehicles", data = "<vehicle>")]
async fn add_vehicle(
    conn: &State<Client>,
    vehicle: Json<Vehicle>,
) -> Result<Json<Vec<Vehicle>>, Custom<String>> {
    execute_query(
        conn,
        "INSERT INTO vehicles (vehicle_type, manufacturer, model, price, data) VALUES ($1, $2, $3, $4, $5)",
        &[&vehicle.vehicle_type, &vehicle.manufacturer, &vehicle.model, &vehicle.price, &vehicle.data],
    )
    .await?;
    get_vehicles(conn).await
}

#[get("/api/vehicles")]
async fn get_vehicles(conn: &State<Client>) -> Result<Json<Vec<Vehicle>>, Custom<String>> {
    get_vehicles_from_db(conn).await.map(Json)
}

async fn get_vehicles_from_db(client: &Client) -> Result<Vec<Vehicle>, Custom<String>> {
    let vehicles = client
        .query(
            "SELECT id, vehicle_type, manufacturer, model, price, data FROM vehicles",
            &[],
        )
        .await
        .map_err(|e| Custom(Status::InternalServerError, e.to_string()))?
        .iter()
        .map(|row| Vehicle {
            id: Some(row.get(0)),
            vehicle_type: row.get(1),
            manufacturer: row.get(2),
            model: row.get(3),
            price: row.get(4),
            data: row.get(5),
        })
        .collect::<Vec<Vehicle>>();
    Ok(vehicles)
}

#[get("/api/vehicles/<id>")]
async fn get_vehicle_info(conn: &State<Client>, id: i32) -> Result<Json<Vehicle>, Custom<String>> {
    get_vehicle_info_from_db(conn, id).await.map(Json)
}
async fn get_vehicle_info_from_db(client: &Client, id: i32) -> Result<Vehicle, Custom<String>> {
    let vehicles = client
        .query(
            "SELECT * FROM vehicles WHERE id = $1",
            &[&id],
        )
        .await
        .map_err(|e| Custom(Status::InternalServerError, e.to_string()))?
        .iter()
        .map(|row| Vehicle {
            id: Some(row.get(0)),
            vehicle_type: row.get(1),
            manufacturer: row.get(2),
            model: row.get(3),
            price: row.get(4),
            data: row.get(5),
        })
        .collect::<Vec<Vehicle>>();
    match vehicles.len() {
        0 => Err(Custom(Status::InternalServerError, "Vehicle with ID not found in the database".to_string())),
        1 => Ok(vehicles[0].clone()),
        _ => {Err(Custom(Status::InternalServerError, "Something is definitely wrong with the database".to_string()))}
    }

}

#[get("/api/vehicles/search/<model>")]
async fn fuzzy_search_vehicles(
    conn: &State<Client>,
    model: &str,
) -> Result<Json<Vec<Vehicle>>, Custom<String>> {
    fuzzy_search_from_db(conn, model).await.map(Json)
}

async fn fuzzy_search_from_db(
    client: &Client,
    model: &str,
) -> Result<Vec<Vehicle>, Custom<String>> {
    let vehicles = client
        .query("SELECT id, vehicle_type, manufacturer, model, price, data FROM vehicles ORDER BY LEVENSHTEIN(model, $1) ASC LIMIT 10", &[&model])
        .await
        .map_err(|e| Custom(Status::InternalServerError, e.to_string()))?
        .iter()
        .map(|row| Vehicle {
            id: Some(row.get(0)),
            vehicle_type: row.get(1),
            manufacturer: row.get(2),
            model: row.get(3),
            price: row.get(4),
            data: row.get(5),
        })
        .collect::<Vec<Vehicle>>();
    Ok(vehicles)
}

#[get("/api/vehicles/search/unique/<column>")]
async fn search_unique_cols_vehicles(
    conn: &State<Client>,
    column: &str,
) -> Result<Json<Vec<String>>, Custom<String>> {
    search_unique_from_db(conn, column).await.map(Json)
}

async fn search_unique_from_db(
    client: &Client,
    col_name: &str,
) -> Result<Vec<String>, Custom<String>> {
    let query = format!(
        "SELECT DISTINCT {} FROM vehicles",
        match col_name {
            c if c == "vehicle_type"
                || c == "manufacturer"
                || c == "model"
                || c == "price"
                || c == "data" =>
            {
                c
            }
            _ => {
                return Err(Custom(
                    Status::InternalServerError,
                    "Unmatched column name given".to_string(),
                ));
            }
        }
    );
    let unique_rows = client
        .query(&query, &[])
        .await
        .map_err(|e| Custom(Status::InternalServerError, e.to_string()))?
        .iter()
        .map(|row| {
            row.get(0)
        })
        .collect::<Vec<String>>();

    Ok(unique_rows)
}

#[put("/api/vehicles/<id>", data = "<vehicle>")]
async fn update_vehicle(
    conn: &State<Client>,
    id: i32,
    vehicle: Json<Vehicle>,
) -> Result<Json<Vec<Vehicle>>, Custom<String>> {
    execute_query(
        conn,
        "UPDATE vehicles SET vehicle_type = $1, manufacturer = $2, model = $3, price = $4, data = $5 WHERE id = $6",
        &[&vehicle.vehicle_type, &vehicle.manufacturer, &vehicle.model, &vehicle.price, &vehicle.data, &id],
    )
    .await?;
    get_vehicles(conn).await
}

#[delete("/api/vehicles/<id>")]
async fn delete_vehicle(conn: &State<Client>, id: i32) -> Result<Status, Custom<String>> {
    execute_query(conn, "DELETE from vehicles WHERE id = $1", &[&id]).await?;
    Ok(Status::NoContent)
}

#[get("/api/vehicles/qr/<id>")]
async fn generate_qr_vehicle(
    conn: &State<Client>,
    id: i32,
) -> Result<Option<File>, Custom<String>> {
    let res = conn.query(
        "SELECT EXISTS(SELECT * FROM vehicles WHERE id = $1)",
        &[&id],
    )
    .await
    .map_err(|e| Custom(Status::InternalServerError, e.to_string()))?
    .iter()
    .map(|row| {let res:bool = row.get(0); println!("exists?: {}", res.clone()); res})
    .collect::<Vec<bool>>();
    match res[0] {
        true => {},
        false => {return Err(Custom(Status::InternalServerError, "Entry with such id doesn't exist".to_string()))}
    }

    let path = create_qr_for_id(id);
    match File::open(path).await {
        Ok(f) => Ok(Some(f)),
        Err(e) => Err(Custom(Status::InternalServerError, e.to_string())),
    }
}

async fn execute_query(
    client: &Client,
    query: &str,
    params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
) -> Result<u64, Custom<String>> {
    client
        .execute(query, params)
        .await
        .map_err(|e| Custom(Status::InternalServerError, e.to_string()))
}

#[launch]
async fn rocket() -> _ {
    let (client, connection) = tokio_postgres::connect(
        "host=localhost user=postgres password=postgres dbname=postgres",
        NoTls,
    )
    .await
    .expect("Failed to connect to Postgres");

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Failed to connect to Postgres {}", e);
        }
    });
    client
        .execute(
            "CREATE TABLE IF NOT EXISTS vehicles (
                id SERIAL PRIMARY KEY,
                vehicle_type TEXT NOT NULL,
                manufacturer TEXT NOT NULL,
                model TEXT NOT NULL,
                price TEXT NOT NULL,
                data TEXT NOT NULL
        )",
            &[],
        )
        .await
        .expect("Failed to create table");
    let cors = CorsOptions::default()
        .allowed_origins(AllowedOrigins::all())
        .to_cors()
        .expect("Error while building CORS");
    rocket::build()
        .manage(client)
        .mount(
            "/",
            routes![
                add_vehicle,
                get_vehicles,
                update_vehicle,
                delete_vehicle,
                fuzzy_search_vehicles,
                search_unique_cols_vehicles,
                generate_qr_vehicle,
                get_vehicle_info,
            ],
        )
        .attach(cors)
}
