use sqlx::{SqlitePool, FromRow};
use anyhow::Result;
use log::{info, debug, error};
use std::{time::SystemTime, vec};
use shared::Message;

const DB_URL: &str = "sqlite://sqlite.db";

#[allow(dead_code)]
#[derive(Clone, FromRow, Debug)]
struct DbMessage {
    time: i64, 
    client: String,
    message: Vec<u8>
}

#[allow(dead_code)]
#[derive(Clone, FromRow, Debug)]
struct LastClientOnlinePresence {
    time: i64,
    client: String,
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
    std::thread::sleep(std::time::Duration::from_millis(20));   // todo: now idea why this is needed; running like "cargo test -- --show-output --test-threads=1"
    Ok(())
}

async fn create_tables(db_url: &str) -> Result<()> {
    let db = SqlitePool::connect(db_url).await?;
    let result = sqlx::query("CREATE TABLE Messages (time INTEGER, client VARCHAR(250) NOT NULL, message blob NOT NULL);").execute(&db).await.unwrap();
    debug!("Create user table result: {:?}", result);
    let result = sqlx::query("CREATE TABLE LastOnline (time INTEGER, client VARCHAR(250) NOT NULL PRIMARY KEY);").execute(&db).await.unwrap();
    debug!("Create last online result: {:?}", result);
    db.close().await;
    Ok(())
}

pub async fn store_message(user_name: &str, message: &Message) {
    if let Err(e) = insert_message(DB_URL, &user_name, &message).await {
        error!("Error inserting message to DB: {}", e);                     // note: probably good reason to exit program gracefully
    }
}

async fn insert_message(db_url: &str, client: &str, message: &Message) -> Result<()> {
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

// pub async fn get_messages(without_client: Option<&str>) -> Result<Vec<(std::time::SystemTime, String, Message)>> {
//     get_messages_from_db(DB_URL, without_client).await
// }
// async fn get_messages_from_db(db_url: &str, without_client: Option<&str>) -> Result<Vec<(std::time::SystemTime, String, Message)>> {
//     let query = match without_client {
//         Some(client) =>
//             sqlx::query_as::<_, DbMessage>("select * from Messages where client != (?)").bind(client),
//         None => 
//             sqlx::query_as::<_, DbMessage>("select * from Messages"),
//     };
//     let db = SqlitePool::connect(db_url).await?;
//     let res = 
//         query
//         .fetch_all(&db)
//         .await?
//         .iter()
//         .map(|row| {
//             let message = Message::deserialize(&row.message).unwrap();
//             (SystemTime::UNIX_EPOCH + std::time::Duration::from_millis(row.time as u64), row.client.to_owned(), message)
//         })
//         .collect();
//     db.close().await;
//     Ok(res)
// }

// async fn update_last_online(db_url: &str, client: &str) -> Result<()> {
//     let time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?.as_millis() as i64;

//     let db = SqlitePool::connect(db_url).await?;
//     let result = sqlx::query("INSERT OR REPLACE INTO LastOnline (time, client) VALUES (?, ?);")
//         .bind(time)
//         .bind(client)
//         .execute(&db).await.unwrap();
//     debug!("Update last online result: {:?}", result);
//     db.close().await;
//     Ok(())
// }

async fn get_last_online_time(db_url: &str, client: &str) -> Result<Option<i64>> {
    let db = SqlitePool::connect(db_url).await?;
    let res = 
        sqlx::query_as::<_, LastClientOnlinePresence>("select * from LastOnline where client = (?)")
        .bind(client)
        .fetch_optional(&db)
        .await?;
    db.close().await;
    // match res {
    //     Some(LastClientOnlinePresence{time, ..}) => Ok(Some(SystemTime::UNIX_EPOCH + std::time::Duration::from_millis(time as u64))),
    //     None => Ok(None)
    // }
    match res {
        Some(LastClientOnlinePresence{time, ..}) => Ok(Some(time)),
        None => Ok(None)
    }
}

pub async fn get_all_last_online_data() -> Vec<(String, std::time::SystemTime)> {
    match get_all_last_online_data_priv(DB_URL).await {
        Err(e) => { error!("Error getting users's last seen from DB: {}", e);
                    vec![]
        },
        Ok(ret) => ret
    }

}
async fn get_all_last_online_data_priv(db_url: &str) -> Result<Vec<(String, std::time::SystemTime)>> {
    let db = SqlitePool::connect(db_url).await?;
    let res = 
        sqlx::query_as::<_, LastClientOnlinePresence>("select * from LastOnline")
        .fetch_all(&db)
        .await?;
    db.close().await;
    let res = res.into_iter()
        .map(|row| (row.client, SystemTime::UNIX_EPOCH + std::time::Duration::from_millis(row.time as u64)))
        .collect();
    Ok(res)

}

pub async fn update_online_users(users: &[String]) {
    if let Err(e) = update_online_users_priv(DB_URL, users).await {
        error!("Error updating online users in DB: {}", e);                     // note: probably good reason to exit program gracefully
    }
}

async fn update_online_users_priv(db_url: &str, users: &[String]) -> Result<()> {
    // note: 
    // let result = sqlx::query("UPDATE LastOnline set time = (?) WHERE client in (?);")
    //     .bind(time)
    //     .bind(users.join(","))  // note: not very safe...
    //     .execute(&db).await.unwrap();
    // doesn't work well - no user is updated; probably that join works differently than I expect (and AI as well)

    let time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?.as_millis() as i64;

    let db = SqlitePool::connect(db_url).await?;
    for user in users {
        let result = sqlx::query("INSERT OR REPLACE INTO LastOnline (time, client) VALUES (?, ?);")
            .bind(time)
            .bind(user)  // note: not very safe...
            .execute(&db).await.unwrap();
        debug!("Update for user '{:?}' result: {:?}", user, result);
    }
    db.close().await;
    Ok(())
}

pub async fn get_missing_messages(user: &str) -> Vec<Message> {
    match get_missing_messages_priv(DB_URL, user).await {
        Err(e) => { 
            error!("Error when getting missing messages from DB for user {}: {}", user, e);
            return vec![]
        },
        Ok(messages) => messages
    }
}
async fn get_missing_messages_priv(db_url: &str, user: &str) -> Result<Vec<Message>> {
    let user_last_online_time = get_last_online_time(db_url, user).await?;
    
    // user not registered yet, don't display him anything from the past...
    if matches!(user_last_online_time, None) {
        return Ok(vec![]);
    }
    let user_last_online_time = user_last_online_time.unwrap();

    let db = SqlitePool::connect(db_url).await?;
    let result = sqlx::query_as::<_, DbMessage>("SELECT * from Messages WHERE time > (?) and client != (?) order by time, client; ")
            .bind(user_last_online_time)
            .bind(user)
            .fetch_all(&db)
            .await?
            .iter()
            .map(|row| {
                Message::deserialize(&row.message).unwrap()
            })
            .collect();
    debug!("Update for user '{:?}' result: {:?}", user, result);

    db.close().await;
    Ok(result)
}

#[cfg(test)]
mod test {
    use super::*;
    //use itertools::Itertools;

    const DB_URL_TESTING: &str = "sqlite://testing_sqlite_a3b094/sqlite_test.db";
    const DB_URL_DIR: &str = "testing_sqlite_a3b094"; // matches the db_url_testing

    fn raw_query(db_url: &str, query: &str) {
        let db = tokio_test::block_on(SqlitePool::connect(db_url)).unwrap();
        tokio_test::block_on(sqlx::query(query).execute(&db)).unwrap();
        tokio_test::block_on(db.close());
    }

    #[test]
    fn test_create_db() {
        let file_path = DB_URL_TESTING.replace("sqlite://", "");
        let path = std::path::Path::new(&file_path);
        if path.exists() {
            std::fs::remove_dir_all(DB_URL_DIR).unwrap();
        }
        std::fs::create_dir(DB_URL_DIR).unwrap();

        tokio_test::block_on(create_if_needed(DB_URL_TESTING)).unwrap();

        assert!(path.exists());

        println!("DB created at {}", file_path);
    }

    #[test]
    fn test_insert_message() {
        test_create_db();
        let msg = Message::Text { from: "".into(), content: "message".into() };
        let msg2 = Message::File{ from: "".into(), name: "file".into(), content: "content".into()};
        tokio_test::block_on(insert_message(DB_URL_TESTING, "test user", &msg)).unwrap();
        tokio_test::block_on(insert_message(DB_URL_TESTING, "test user2", &msg2)).unwrap();

        println!("user inserted: test_user");
    }

    // #[test]
    // fn test_insert_and_get_without() {
    //     test_create_db();
    //     let msg = Message::Text { from: "".into(), content: "message".into() };
    //     let msg2 = Message::File{ from: "".into(), name: "file".into(), content: "content".into()};
    //     tokio_test::block_on(insert_message(DB_URL_TESTING, "test user", &msg)).unwrap();
    //     tokio_test::block_on(insert_message(DB_URL_TESTING, "test user", &msg2)).unwrap();
    //     tokio_test::block_on(insert_message(DB_URL_TESTING, "test user2", &msg2)).unwrap();

    //     let rows = tokio_test::block_on(get_messages_from_db(DB_URL_TESTING, Some("test user"))).unwrap();
    //     assert_eq!(rows.len(), 1);
    //     assert_eq!(rows[0].1, "test user2");
    //     assert!(matches!(&rows[0].2, Message::File{from:_,name:_,content:_}));

    //     println!("get without: test_user");
    // }

    // #[test]
    // fn test_insert_and_get_all() {
    //     test_create_db();
    //     let msg = Message::Text { from: "".into(), content: "message".into() };
    //     let msg2 = Message::File{ from: "".into(), name: "file".into(), content: "content".into()};
    //     tokio_test::block_on(insert_message(DB_URL_TESTING, "test user", &msg)).unwrap();
    //     tokio_test::block_on(insert_message(DB_URL_TESTING, "test user", &msg2)).unwrap();
    //     tokio_test::block_on(insert_message(DB_URL_TESTING, "test user2", &msg2)).unwrap();

    //     let rows = tokio_test::block_on(get_messages_from_db(DB_URL_TESTING, None)).unwrap();
    //     assert_eq!(rows.len(), 3);
    //     let clients = rows.iter()
    //         .map(|(_,client,_)| client)
    //         .sorted()
    //         .unique()
    //         .collect::<Vec<_>>();
    //     assert_eq!(clients[0], "test user");
    //     assert_eq!(clients[1], "test user2");

    //     println!("get all: test_user; clients: {:?}", clients);
    //     //std::thread::sleep(std::time::Duration::from_secs(5));
    // }

    // #[test]
    // fn test_user_presence_update_works_if_this_is_first_visit() {
    //     test_create_db();

    //     tokio_test::block_on(update_last_online(DB_URL_TESTING, "test user")).unwrap();

    //     let last_online = tokio_test::block_on(get_last_online(DB_URL_TESTING, "test user")).unwrap();
    //     assert!(last_online.is_some());
    //     let last_online = last_online.unwrap();
    //     assert_eq!(last_online.client, "test user");
    //     assert!(last_online.time <= SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis() as i64);
    // }

    // #[test]
    // fn test_user_presence_update_works_if_this_is_repeated_visit() {
    //     // setup
    //     test_create_db();
    //     raw_query(DB_URL_TESTING, "INSERT INTO LastOnline (time, client) VALUES (10, 'test user');");

    //     //act 
    //     tokio_test::block_on(update_last_online(DB_URL_TESTING, "test user")).unwrap();

    //     // verify
    //     let last_online = tokio_test::block_on(get_last_online(DB_URL_TESTING, "test user")).unwrap().unwrap();
    //     let expected_time_at_least = (SystemTime::now() - std::time::Duration::from_secs(10)).duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis() as i64;
    //     println!("Last online: {:?}", last_online);

    //     assert_eq!(last_online.client, "test user");
    //     assert!(last_online.time > expected_time_at_least);
    // }

    #[test]
    fn test_user_presence_update_for_some_users() {
        // setup
        test_create_db();
        raw_query(DB_URL_TESTING, "INSERT INTO LastOnline (time, client) VALUES (10, 'test user');");
        raw_query(DB_URL_TESTING, "INSERT INTO LastOnline (time, client) VALUES (10, 'test user2');");
        raw_query(DB_URL_TESTING, "INSERT INTO LastOnline (time, client) VALUES (10, 'test user3');");

        // act
        tokio_test::block_on(update_online_users_priv(DB_URL_TESTING, &["test user".into(), "test user3".into()])).unwrap();

        // verify
        let u1 = tokio_test::block_on(get_last_online_time(DB_URL_TESTING, "test user")).unwrap().unwrap();
        let u2 = tokio_test::block_on(get_last_online_time(DB_URL_TESTING, "test user2")).unwrap().unwrap();
        let u3 = tokio_test::block_on(get_last_online_time(DB_URL_TESTING, "test user3")).unwrap().unwrap();
        println!("Times: {} {} {}", u1, u2, u3);
        let expected_time_at_least = (SystemTime::now() - std::time::Duration::from_secs(10)).duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis() as i64;

        assert_eq!(u2, 10);
        assert!(u1 > expected_time_at_least);
        assert!(u3 > expected_time_at_least);
    }

    #[test]
    fn test_user_presence_update_works_even_for_nonexisting_user() {
        // setup
        test_create_db();
        raw_query(DB_URL_TESTING, "INSERT INTO LastOnline (time, client) VALUES (10, 'test user');");

        // act
        tokio_test::block_on(update_online_users_priv(DB_URL_TESTING, &["test user2".into()])).unwrap();

        // verify
        let u1 = tokio_test::block_on(get_last_online_time(DB_URL_TESTING, "test user")).unwrap().unwrap();
        let u2 = tokio_test::block_on(get_last_online_time(DB_URL_TESTING, "test user2")).unwrap().unwrap();
        println!("Times: {} {}", u1, u2);
        let expected_time_at_least = (SystemTime::now() - std::time::Duration::from_secs(10)).duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis() as i64;
        
        assert_eq!(u1, 10);
        assert!(u2 > expected_time_at_least);
    }

    #[test]
    fn test_get_only_missing_messages_for_given_user() {
        // setup - insert messages
        test_create_db();
        let msg_u1_t = Message::Text { from: "test user".into(), content: "message".into() };
        let msg_u2_f = Message::File{ from: "test user2".into(), name: "file".into(), content: "content".into()};
        let msg_u2_t1 = Message::Text { from: "test user2".into(), content: "another message".into()};
        let msg_u2_t2 = Message::Text { from: "test user2".into(), content: "last message".into()};
        let msg_u3_t1 = Message::Text { from: "test user3".into(), content: "user3 message".into()};
        tokio_test::block_on(insert_message(DB_URL_TESTING, "test user", &msg_u1_t)).unwrap();
        tokio_test::block_on(insert_message(DB_URL_TESTING, "test user2", &msg_u2_f)).unwrap();
        // setup - update users
        tokio_test::block_on(update_online_users_priv(DB_URL_TESTING, &["test user".into(), "test user2".into()])).unwrap();
        // setup - wait and add another messages
        std::thread::sleep(std::time::Duration::from_millis(50));
        tokio_test::block_on(insert_message(DB_URL_TESTING, "test user2", &msg_u2_t1)).unwrap();
        tokio_test::block_on(insert_message(DB_URL_TESTING, "test user2", &msg_u2_t2)).unwrap();
        tokio_test::block_on(insert_message(DB_URL_TESTING, "test user3", &msg_u3_t1)).unwrap();

        // act
        let missing_messages = tokio_test::block_on(get_missing_messages_priv(DB_URL_TESTING, "test user")).unwrap();

        // verify
        assert_eq!(missing_messages.len(), 3);
        assert_eq!(&missing_messages[0], &msg_u2_t1);
        assert_eq!(&missing_messages[1], &msg_u2_t2);
        assert_eq!(&missing_messages[2], &msg_u3_t1);
    }

    #[test]
    fn test_missing_messages_are_empty_for_unknown_user() {
        // setup - insert messages
        test_create_db();
        let msg_u1_t = Message::Text { from: "test user".into(), content: "message".into() };
        let msg_u2_f = Message::File{ from: "test user2".into(), name: "file".into(), content: "content".into()};
        tokio_test::block_on(insert_message(DB_URL_TESTING, "test user", &msg_u1_t)).unwrap();
        tokio_test::block_on(insert_message(DB_URL_TESTING, "test user2", &msg_u2_f)).unwrap();
        // setup - update users
        tokio_test::block_on(update_online_users_priv(DB_URL_TESTING, &["test user".into(), "test user2".into()])).unwrap();

        // act
        let missing_messages = tokio_test::block_on(get_missing_messages_priv(DB_URL_TESTING, "test user3")).unwrap();

        // verify
        assert!(missing_messages.is_empty());
    }
}