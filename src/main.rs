use memcache;
use memcache::MemcacheError;
use tokio::net::UdpSocket;
use tokio::runtime::*;

const NUMS: u32 = 1000000;
const MEMCACHED_SERVER: &str = "127.0.0.1";

fn validate_server(server: &memcache::Client) -> std::result::Result<(), MemcacheError> {
    // flush the database:
    server.flush()?;

    // set a string value:
    server.set("foo", "bar", 0)?;

    // retrieve from memcached:
    let value: Option<String> = server.get("foo")?;
    assert_eq!(value, Some(String::from("bar")));

    println!("memcached server works, and set with foo bar keypair");
    Ok(())
}

fn wrap_get_command(key: String, seq: u16) -> Vec<u8> {
    let mut bytes: Vec<u8> = vec![0, 0, 0, 1, 0, 0];
    let mut command = format!("get {}\r\n", key).into_bytes();
    let mut seq_bytes = seq.to_be_bytes().to_vec();
    seq_bytes.append(&mut bytes);
    seq_bytes.append(&mut command);
    // println!("bytes: {:?}", seq_bytes);
    seq_bytes
}

async fn send_get_commands() {
    let socket = UdpSocket::bind("0.0.0.0:0").await.unwrap();
    let mut seq = 0u16;

    for _ in 0..NUMS {
        let command = wrap_get_command("foo".to_string(), seq);
        seq = seq.wrapping_add(1);
        socket
            .send_to(&command[..], format!("{}:11211", MEMCACHED_SERVER))
            .await
            .expect("failed to send");
        let mut buf = [0; 1024];
        if let Ok((amt, _)) = socket.recv_from(&mut buf).await {
            // println!("amt: {}", amt);
        }
    }
}

fn main() {
    let server = memcache::connect(format!(
        "memcache+udp://{}:{}?timeout=10",
        MEMCACHED_SERVER, "11211"
    ))
    .unwrap();

    validate_server(&server).unwrap();

    let rt = Runtime::new().unwrap();
    //
    rt.block_on(async move { send_get_commands().await });
    // std::thread::sleep(std::time::Duration::from_secs(10));
}
