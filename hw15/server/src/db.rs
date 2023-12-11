use sqlx::{SqlitePool, FromRow};
use anyhow::Result;
use log::{info, debug};
use std::time::SystemTime;
//use itertools::Itertools;

const DB_URL: &str = "sqlite://sqlite.db";

#[derive(Clone, FromRow, Debug)]
struct Message {
    time: i64, 
    client: String,
    message: Vec<u8>
}

pub async fn ensure_db_exists() -> Result<()> {
    create_if_needed(DB_URL).await
}

async fn create_if_needed(db_url: &str) -> Result<()> {
    use sqlx::{Sqlite, migrate::MigrateDatabase};

    if !Sqlite::database_exists(db_url).await.unwrap_or(false) {
        debug!("Creating database {}", db_url);
        match Sqlite::create_database(db_url).await {
            Ok(_) => info!("Create db success"),
            Err(error) => panic!("error: {}", error),
        }

        create_tables(db_url).await?;
    } else {
        debug!("Database already exists");
    }
    Ok(())
}

async fn create_tables(db_url: &str) -> Result<()> {
    let db = SqlitePool::connect(db_url).await?;
    let result = sqlx::query("CREATE TABLE Messages (time INTEGER, client VARCHAR(250) NOT NULL, message blob NOT NULL);").execute(&db).await.unwrap();
    debug!("Create user table result: {:?}", result);
    db.close().await;
    Ok(())
}


pub async fn insert(client: &str, message: &shared::Message) -> Result<()> {
    insert_message(DB_URL, client, message).await
}
async fn insert_message(db_url: &str, client: &str, message: &shared::Message) -> Result<()> {
    let message_blob = message.serialize()?;
    let time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?.as_millis() as i64;

    let db = SqlitePool::connect(db_url).await?;
    let result = sqlx::query("INSERT INTO Messages (time, client, message) VALUES (?, ?, ?);")
        .bind(time)
        .bind(client)
        .bind(message_blob)
        .execute(&db).await.unwrap();
    debug!("Insert result: {:?}", result);
    db.close().await;
    Ok(())
}

pub async fn get_messages(without_client: Option<&str>) -> Result<Vec<(std::time::SystemTime, String, shared::Message)>> {
    get_messages_from_db(DB_URL, without_client).await
}
async fn get_messages_from_db(db_url: &str, without_client: Option<&str>) -> Result<Vec<(std::time::SystemTime, String, shared::Message)>> {
    let query = match without_client {
        Some(client) =>
            sqlx::query_as::<_, Message>("select * from Messages where client != (?)").bind(client),
        None => 
            sqlx::query_as::<_, Message>("select * from Messages"),
    };
    let db = SqlitePool::connect(db_url).await?;
    let res = 
        query
        .fetch_all(&db)
        .await?
        .iter()
        .map(|row| {
            let message = shared::Message::deserialize(&row.message).unwrap();
            (SystemTime::UNIX_EPOCH + std::time::Duration::from_millis(row.time as u64), row.client.to_owned(), message)
        })
        .collect();
    db.close().await;
    Ok(res)
}


#[cfg(test)]
mod test {
    use super::*;

    const DB_URL_TESTING: &str = "sqlite://testing/sqlite_test.db";

    #[test]
    fn test_create_db() {
        let file_path = DB_URL_TESTING.replace("sqlite://", "");
        let path = std::path::Path::new(&file_path);
        if path.exists() {
            std::fs::remove_file(path).unwrap();
        }

        tokio_test::block_on(create_if_needed(DB_URL_TESTING)).unwrap();

        assert!(path.exists());

        println!("DB created at {}", file_path);
    }

    #[test]
    fn test_insert_message() {
        test_create_db();
        let msg = shared::Message::Text("message".into());
        let msg2 = shared::Message::File{ name: "file".into(), content: "content".into()};
        tokio_test::block_on(insert_message(DB_URL_TESTING, "test user", &msg)).unwrap();
        tokio_test::block_on(insert_message(DB_URL_TESTING, "test user2", &msg2)).unwrap();

        println!("user inserted: test_user");
    }

    #[test]
    fn test_insert_and_get_without() {
        test_create_db();
        let msg = shared::Message::Text("message".into());
        let msg2 = shared::Message::File{ name: "file".into(), content: "content".into()};
        tokio_test::block_on(insert_message(DB_URL_TESTING, "test user", &msg)).unwrap();
        tokio_test::block_on(insert_message(DB_URL_TESTING, "test user", &msg2)).unwrap();
        tokio_test::block_on(insert_message(DB_URL_TESTING, "test user2", &msg2)).unwrap();

        let rows = tokio_test::block_on(get_messages_from_db(DB_URL_TESTING, Some("test user"))).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].1, "test user2");
        assert!(matches!(&rows[0].2, shared::Message::File{name,content}));

        println!("get without: test_user");
    }

    #[test]
    fn test_insert_and_get_all() {
        test_create_db();
        let msg = shared::Message::Text("message".into());
        let msg2 = shared::Message::File{ name: "file".into(), content: "content".into()};
        tokio_test::block_on(insert_message(DB_URL_TESTING, "test user", &msg)).unwrap();
        tokio_test::block_on(insert_message(DB_URL_TESTING, "test user", &msg2)).unwrap();
        tokio_test::block_on(insert_message(DB_URL_TESTING, "test user2", &msg2)).unwrap();

        let rows = tokio_test::block_on(get_messages_from_db(DB_URL_TESTING, None)).unwrap();
        assert_eq!(rows.len(), 3);
        // let mut clients = rows.iter().map(|(_,client,_)| client).collect::<Vec<_>>();
        // clients.sort();
        // clients.dedup();
        let clients = rows.iter()
            .map(|(_,client,_)| client)
            .sorted()
            .unique()
            .collect::<Vec<_>>();
        assert_eq!(clients[0], "test user");
        assert_eq!(clients[1], "test user2");

        println!("get all: test_user; clients: {:?}", clients);
    }
}