#[macro_use]
extern crate warp;
extern crate postgres;
extern crate r2d2;
extern crate r2d2_postgres;
extern crate uuid;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

use uuid::Uuid;
use warp::Filter;
use r2d2_postgres::{TlsMode, PostgresConnectionManager};
use std::env;

type Db = PostgresConnectionManager;

#[derive(Serialize, Deserialize, Debug)]
struct ApiKey {
    id: Uuid,
    key: String,
    token: String,
    username: String,
}

fn check(pg_pool: r2d2::Pool<r2d2_postgres::PostgresConnectionManager>, get_access_token: String) -> Result<impl warp::Reply, warp::Rejection> {
    let mut api_key = ApiKey { 
        id: Uuid::nil(), 
        key: String::new(), 
        token: String::new(), 
        username: String::new(), 
    };

    let pg_conn = pg_pool.get().unwrap();
    for row in &pg_conn.query("SELECT id, key, username, token FROM radius_api_keys WHERE token = $1", &[&get_access_token]).unwrap() {
        api_key = ApiKey {
            id: row.get("id"),
            key: row.get("key"),
            token: row.get("token"), 
            username: row.get("username"),
        };
    }
    let json_api_key = serde_json::to_string(&api_key);
    println!("{:?}", json_api_key);
    Ok(warp::reply::json(&api_key))
}

fn main() {
    let pg_url = match env::var("PG_URL") {
        Ok(pg_url) => { 
             println!("pg_url: {}", pg_url);
             pg_url
        },
        Err(e) => panic!("Couldn't read PG_URL ({})", e),
    };

    let pg_pool = r2d2::Pool::new(Db::new(pg_url, TlsMode::None).unwrap()).unwrap();
    let db = warp::any().map(move || pg_pool.clone());

    let access_token = path!("cdr" / "access_token" / String)
        .map(|access_token| format!("{}", access_token));
    
    let api_cdr = warp::get2()
        .and(db)
        .and(access_token)
        .and_then(check);

    let route = api_cdr;

    warp::serve(route)
        .run( ([127, 0, 0, 1], 8080) );
}

