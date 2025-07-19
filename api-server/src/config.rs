use redis::{
    aio::Connection,Client
};

pub async fn get_client()->Connection{
    let client=Client::open("redis://127.0.0.1/").unwrap();
    client.get_async_connection().await.unwrap()
}



