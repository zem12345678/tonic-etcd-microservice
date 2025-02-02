use tonic::transport::Channel;
use etcd_client::{Client,GetOptions};

// 引入生成的 gRPC 代码
pub mod example {
    tonic::include_proto!("example");
}

use example::{
    example_service_client::ExampleServiceClient,
    HelloRequest
};

async fn discover_service(
    etcd_client: &mut Client,
    service_name: &str
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let resp = etcd_client.get(service_name, Some(GetOptions::default())).await?;

   if let Some(kv) =resp.kvs().first() {
       let service_address = String::from_utf8(kv.value().to_vec())?;
       Ok(Some(service_address))
   } else {
       Ok(None)

   }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ETCD 客户端
    let mut etcd_client = Client::connect(["localhost:2379"], None).await?;

    // 服务发现
    if let  Some(service_address) = discover_service(&mut etcd_client, "example_service").await? {
        // 创建 gRPC 客户端
        let channel = Channel::from_shared(format!("http://{}", service_address))?
            .connect()
            .await?;

        let mut client = ExampleServiceClient::new(channel);

        // 调用服务
        let request = tonic::Request::new(HelloRequest {
            name: "Rust gRPC Client".to_string(),
        });

        let response = client.say_hello(request).await?;
        println!("Response: {:?}", response.into_inner().message);
    } else {
        println!("No service address found");
    }



    Ok(())
}