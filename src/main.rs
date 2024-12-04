use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_postgres::NoTls;

#[derive(Clone)]
struct AppState {
    db_client: Arc<Mutex<tokio_postgres::Client>>,
}

// Usuario struct para serializar/deserializar JSON
#[derive(Serialize, Deserialize)]
struct Usuario {
    id_usuario: i32,
    nombre: String,
    apellido: String,
    email: String,
}

// Telefono struct para serializar/deserializar JSON
#[derive(Serialize, Deserialize)]
struct Telefono {
    id_telf: i32,
    marca: String,
    modelo: String,
    precio: f64,
}

#[tokio::main]
async fn main() {
    // Conectar a la base de datos
    let (client, connection) =
        tokio_postgres::connect("host=localhost user=postgres password=1504 dbname=Base_de_Datos_CRUD", NoTls)
            .await
            .expect("No se pudo conectar a la base de datos");

            println!("Conectado a la base de datos con éxito!");

    // Lanzar la conexión en segundo plano
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Error en la conexión: {}", e);
        }
    });

    // Crear el estado compartido
    let state = AppState {
        db_client: Arc::new(Mutex::new(client)),
    };

    // Configurar las rutas
    let app = Router::new()
        .route("/usuarios", post(crear_usuario).get(obtener_usuarios))
        .route("/usuarios/:id", get(obtener_usuario).put(actualizar_usuario).delete(eliminar_usuario))
        .with_state(state);

    // Ejecutar el servidor
    println!("Iniciando el servidor...");

    // Ejecutar el servidor
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();

    // Imprimir mensaje si el servidor está corriendo
    println!("Servidor corriendo en http://localhost:3000");
}

// Crear un usuario
async fn crear_usuario(
    State(state): State<AppState>,
    Json(payload): Json<Usuario>,
) -> Json<String> {
    let query = "INSERT INTO usuario (id_usuario, nombre, apellido, email) VALUES ($1, $2, $3, $4)";
    let client = state.db_client.lock().await;

    if let Err(e) = client.execute(query, &[&payload.id_usuario, &payload.nombre, &payload.apellido, &payload.email]).await {
        return Json(format!("Error al insertar usuario: {}", e));
    }

    Json("Usuario creado con éxito".to_string())
}

// Obtener todos los usuarios
async fn obtener_usuarios(State(state): State<AppState>) -> Json<Vec<Usuario>> {
    let query = "SELECT id_usuario, nombre, apellido, email FROM usuario";
    let client = state.db_client.lock().await;

    let rows = match client.query(query, &[]).await {
        Ok(rows) => rows,
        Err(_) => return Json(vec![]),
    };

    let usuarios = rows
        .iter()
        .map(|row| Usuario {
            id_usuario: row.get(0),
            nombre: row.get(1),
            apellido: row.get(2),
            email: row.get(3),
        })
        .collect();

    Json(usuarios)
}

// Obtener un usuario por ID
async fn obtener_usuario(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Json<Option<Usuario>> {
    let query = "SELECT id_usuario, nombre, apellido, email FROM usuario WHERE id_usuario = $1";
    let client = state.db_client.lock().await;

    let row = client.query_opt(query, &[&id]).await.unwrap();

    if let Some(row) = row {
        Json(Some(Usuario {
            id_usuario: row.get(0),
            nombre: row.get(1),
            apellido: row.get(2),
            email: row.get(3),
        }))
    } else {
        Json(None)
    }
}

// Actualizar un usuario
async fn actualizar_usuario(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(payload): Json<Usuario>,
) -> Json<String> {
    let query = "UPDATE usuario SET nombre = $1, apellido = $2, email = $3 WHERE id_usuario = $4";
    let client = state.db_client.lock().await;

    if let Err(e) = client
        .execute(query, &[&payload.nombre, &payload.apellido, &payload.email, &id])
        .await
    {
        return Json(format!("Error al actualizar usuario: {}", e));
    }

    Json("Usuario actualizado con éxito".to_string())
}

// Eliminar un usuario
async fn eliminar_usuario(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Json<String> {
    let query = "DELETE FROM usuario WHERE id_usuario = $1";
    let client = state.db_client.lock().await;

    if let Err(e) = client.execute(query, &[&id]).await {
        return Json(format!("Error al eliminar usuario: {}", e));
    }

    Json("Usuario eliminado con éxito".to_string())
}
