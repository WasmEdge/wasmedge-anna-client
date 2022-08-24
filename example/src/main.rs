use std::time::Duration;

use wasmedge_anna_client::{Client, ClientConfig};

#[tokio::main(flavor = "current_thread")]
async fn main() -> eyre::Result<()> {
    set_up_logger()?;

    let mut client = Client::new(ClientConfig {
        routing_ip: "127.0.0.1".parse().unwrap(),
        routing_port_base: 12340,
        routing_threads: 1,
        timeout: Duration::from_secs(10),
    })?;

    // put the value
    let time = format!("{}", chrono::Utc::now());
    client.put_lww("time".into(), time.into()).await?;
    println!("Successfully PUT `time`");

    // sleep 1 second
    tokio::time::sleep(Duration::from_secs(1)).await;

    // get the value
    let bytes = client.get_lww("time".into()).await?;
    let value = String::from_utf8(bytes)?;
    println!("Successfully GET `time`: {}", value);

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
