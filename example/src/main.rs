use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};

use wasmedge_anna_client::{redis_like, Client, ClientConfig};

#[tokio::main(flavor = "current_thread")]
async fn main() -> eyre::Result<()> {
    set_up_logger()?;

    let config = ClientConfig {
        routing_ip: "127.0.0.1".parse().unwrap(),
        routing_port_base: 12340,
        routing_threads: 1,
        timeout: Duration::from_secs(10),
    };

    test_put_get_lww(config.clone()).await?;
    test_transaction(config.clone()).await?;
    test_redis_like_client(config).await?;

    Ok(())
}

#[allow(unused)]
async fn test_put_get_lww(config: ClientConfig) -> eyre::Result<()> {
    log::info!("test_put_get_lww");

    let mut client = Client::new(config)?;

    let time = format!("{}", chrono::Utc::now());
    client.put_lww("time".into(), time.into()).await?;
    log::info!("Successfully PUT `time`");

    let bytes = client.get_lww("time".into()).await?;
    let value = String::from_utf8(bytes)?;
    log::info!("Successfully GET `time`: {}", value);

    Ok(())
}

#[allow(unused)]
async fn test_transaction(config: ClientConfig) -> eyre::Result<()> {
    log::info!("test_transaction");

    let mut client = Client::new(config)?;

    let mut tx = client.begin_transaction();
    let time = format!("{}", chrono::Utc::now());
    tx.put_lww("time".into(), time.into()).await?;
    let bytes = tx.get_lww("time".into()).await?;
    let value = String::from_utf8(bytes)?;
    log::info!("Successfully GET `time` in transaction: {}", value);
    tx.commit().await?;

    let bytes = client.get_lww("time".into()).await?;
    let value = String::from_utf8(bytes)?;
    log::info!("Successfully GET `time` after transaction: {}", value);

    Ok(())
}

#[allow(unused)]
async fn test_redis_like_client(config: ClientConfig) -> eyre::Result<()> {
    log::info!("test_redis_like_client");

    let client = redis_like::Client::open(config)?;
    let mut con = client.get_async_connection().await?;
    con.set("my_key", 42i32).await?;

    let val: i32 = con.get("my_key").await?;
    assert_eq!(val, 42i32);

    con.set_nx("my_key", "should not be set").await?;
    let val: i32 = con.get("my_key").await?;
    assert_eq!(val, 42i32);

    con.set_nx("hello", "world").await?;
    let val: String = con.get("hello").await?;
    assert_eq!(val, "world");

    let mut hash_set1 = HashSet::new();
    hash_set1.insert(b"hello".to_vec());
    hash_set1.insert(b"world".to_vec());
    con.s_add("hash_set_key", hash_set1.clone()).await?;

    let mut hash_set2 = HashSet::new();
    hash_set2.insert(b"anna".to_vec());
    con.s_add("hash_set_key", hash_set2.clone()).await?;

    let val = con.s_get("hash_set_key").await?;

    hash_set1.extend(hash_set2);
    assert_eq!(val, hash_set1);

    let mut hash_map1 = HashMap::new();
    hash_map1.insert("key1".to_string(), b"value1".to_vec());
    hash_map1.insert("key2".to_string(), b"value2".to_vec());
    con.h_mset("hash_map_key", hash_map1.clone()).await?;

    let mut hash_map2 = HashMap::new();
    hash_map2.insert("key3".to_string(), b"value3".to_vec());
    con.h_mset("hash_map_key", hash_map2.clone()).await?;

    let val = con.h_getall("hash_map_key").await?;

    hash_map1.extend(hash_map2);
    assert_eq!(val, hash_map1);

    let val = con.inc("inc_key", 10).await?;
    assert_eq!(val, 10);
    let val = con.inc("inc_key", 0).await?;
    assert_eq!(val, 10);
    let val = con.inc("inc_key", -15).await?;
    assert_eq!(val, -5);

    log::info!("Success!");
    Ok(())
}

fn set_up_logger() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .chain(std::io::stdout())
        .apply()?;
    Ok(())
}
