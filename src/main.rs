use std::time::Duration;

use anyhow::{Context, Ok};
use axum::{Json, Router, extract::State, http::StatusCode, routing::{get, post}, serve};
use dotenvy::{dotenv, var};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row, postgres::PgPoolOptions};

#[tokio::main]
async fn main()->anyhow::Result<()>{

    // Checking if dotenv from dotenvy is working rightly
    // dotenv().ok();

    //Accessing the field, named {DATABASE_URL} in .env file 
    // let database_url = var("DATABASE_URL").unwrap();
    let database_url = var("DATABASE_URL").context("Database needs to be set")?;

    // SQL/Database connection
    let pool = PgPoolOptions::new().max_connections(10)
    .acquire_timeout(Duration::from_secs(5))
    .connect(&database_url)
    .await
    .context("failed to connect to database")?;


    // Making migration
    sqlx::migrate!("./migrations").run(&pool).await.context("Error migrating")?;

    
    //Printing the accessed variable
    // println!("database {}", database_url);

    // For sqlx-cli is not [add] is [install], thus cargo install sqlx-cli


    //SQL
    // let sql = "SELECT 'RustTaskStoreDb' as name";

    //Difining a differed statement
    // let que = sqlx::query(sql).fetch_one(&pool).await?;

    // println!("database connection {:?}", que.get::<String, _>("name"));
    // 
    let app = Router::new()
    .route("/", get(async || "Hi john"))
    // .route("/task", post(create_task))
    .route("/task", post(create_task_with_body))
    .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.
    context("Failed to listen")?;

    // serve(listener, app).await.unwrap();
    serve(listener, app).await?;

    Ok(())


    // let appx = Router::new()
    //            .route("/", get(root))
    //            .route("/users", get(list).post(create))
    //            .route("/users/{id}", get(show).delete(destory))
    //            .route("/files/{*rest}", get(any_depth))
    //            .nest("/api", api_router())
    //            .fallback(not_found);
    
    // let listenerx = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    // axum::serve(listenerx, app).await.unwrap();

}




//Creating the handlers(functions) for each endpoints

async fn create_task(State(pool): State<PgPool>, Json(data): 
Json<TaskRequestBody>) -> StatusCode {

    let task = sqlx::query("INSERT INTO task (title) VALUES ($1) RETURNING id ")
    .bind(&data.title)
    .fetch_one(&pool)
    .await
    .context("Error inserting record to db").unwrap();

    let task_id = task.get::<i32, _>("id");
    println!("Task {}", task_id);

    StatusCode::CREATED

}



// Returning create_task with status and taskresponsebody

async fn create_task_with_body(State(pool): State<PgPool>, Json(data): 
Json<TaskRequestBody>) -> (StatusCode, Json<TaskResponseBody>) {

    let task = sqlx::query("INSERT INTO task (title) VALUES ($1) RETURNING id ")
    .bind(&data.title)
    .fetch_one(&pool)
    .await
    .context("Error inserting record to db").unwrap();

    let task_id = task.get::<i32, _>("id");
    println!("Task {}", task_id);

    (
        StatusCode::CREATED, 
        Json(
            TaskResponseBody 
            { 
                message: "task created successflly".to_string(), 
                id: task_id 
            }
        )
    )

}



async fn read_task(State(pool):State<PgPool>)->Json<TaskReadResponse>
{
    let tasks = sqlx::query("SELECT * FROM task WHERE id = $1")
    .fetch_all(&pool)
    .await
    .context("failed to read task").unwrap();

    let tasks_rows = tasks.into_iter().map(|row| TaskState{
        title :row.get::<String, _>("title"),
        id:row.get::<i32, _>("id")
    }).collect();
    Json(TaskReadResponse { title: "Fetched task successfully".to_string(), task: tasks_rows })
}



// struct in this context serves as model or dto that is to be passed through requesting client


#[derive(Deserialize, Serialize)]
struct TaskRequestBody {
    title:String

}


#[derive(Deserialize,Serialize)]
struct TaskResponseBody {
    message:String,
    id:i32
}


#[derive(Deserialize, Serialize)]
struct TaskState {
    title:String,
    id: i32
}


#[derive(Deserialize, Serialize)]
struct TaskReadResponse {
    title:String,
    task:Vec<TaskState>
}
//Set up database

