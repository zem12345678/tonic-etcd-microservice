use tonic::{Request, Response, Status, transport::Server};
use etcd_client::{Client, LeaseKeepAliveStream, LeaseKeeper, PutOptions};
use tokio::signal;
use tokio::signal::unix::signal;

// 引入生成的 gRPC 代码
pub mod example {
    tonic::include_proto!("example");
}

use example::{
    example_service_server::{ExampleService, ExampleServiceServer},
    HelloRequest, HelloReply
};

#[derive(Default)]
pub struct ExampleServiceImpl {}

#[tonic::async_trait]
impl ExampleService for ExampleServiceImpl {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>
    ) -> Result<Response<HelloReply>, Status> {
        let name = request.into_inner().name;
        Ok(Response::new(HelloReply {
            message: format!("Hello, {}!", name)
        }))
    }
}

async fn register_service(
    etcd_client: &mut Client,
    service_name: &str,
    service_addr: &str,
    lease_id: i64,
) -> Result<(LeaseKeeper,LeaseKeepAliveStream), Box<dyn std::error::Error>> {
    etcd_client.put(service_name, service_addr, Some(PutOptions::new().with_lease(lease_id)))
        .await?;

    let(mut lease_keeper,  lease_keep_alive_stream) = etcd_client.lease_keep_alive(lease_id).await?;
    lease_keeper.keep_alive().await?;
    Ok((lease_keeper,lease_keep_alive_stream))

}

async fn etcd_delete (etcd_client: &mut Client,key: &str, lease_id: i64) {
    etcd_client.delete(key, None).await;
    etcd_client.lease_revoke(lease_id).await;

}
async fn etcd_watch(etcd_client: &mut Client,service_name: &str) {
    let (mut watcher,mut watch_stream) = etcd_client.watch(service_name, None).await.unwrap();
    println!("{}",watcher.watch_id());
    while let Some(resp) = watch_stream.message().await.unwrap() {

        if resp.created() {
            println!("watcher created: {}", resp.watch_id());
        }

        if resp.canceled() {
            println!("watch canceled: {}", resp.watch_id());
        }

        for event in resp.events() {
            if let Some(kv) = event.kv() {

            }
            match event.event_type() {
                etcd_client::EventType::Put => {
                    println!("Service updated: {:?}", event.kv());
                }
                etcd_client::EventType::Delete => {
                    watcher.cancel_by_id(resp.watch_id()).await;
                    println!("Service deleted: {:?}", event.kv());
                }
                _ => {}
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051";

    // ETCD 客户端
    let mut etcd_client = Client::connect(["localhost:2379"], None).await?;
    let lease = etcd_client.lease_client().grant(10, None).await?;
    let lease_id = lease.id();
    // 服务注册
    let (mut lease_keeper, mut lease_keep_alive_stream) = register_service(
        &mut etcd_client,
        "example_service",
        &addr,
        lease_id,
    ).await?;

    tokio::spawn(async move {
        loop {
            match lease_keep_alive_stream.message().await {
                Ok(Some(_)) => {
                    // 如果有消息，表示租约仍然有效，可以继续续约
                    lease_keeper.keep_alive().await.unwrap();
                    println!("Service keeper updated");
                }
                Ok(None) => {
                    // 如果没有收到消息，表示连接断开，需要重新连接
                    println!("Lease keep-alive stream closed, reconnecting...");
                    break;
                }
                Err(e) => {
                    // 处理错误，连接可能会断开
                    eprintln!("Error in keep-alive stream: {:?}", e);
                    break;
                }
            }

            // 等待一段时间再进行续约
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }
    });
    tokio::spawn(async move {
        etcd_watch(&mut etcd_client, "example_service").await;
    });
    println!("Server starting on {}", addr);

    // 服务实现
    let service = ExampleServiceImpl::default();

    // 启动 gRPC 服务
    Server::builder()
        .add_service(ExampleServiceServer::new(service))
        .serve(addr.parse().unwrap())
        .await?;



    // 使用select!同时处理服务运行和信号
    // tokio::select! {
    // _ = signal::ctrl_c() => {
    //     println!("Received termination signal");
    //     etcd_delete(&mut etcd_client, "example_service", lease_id).await;
    //     }
    // }
    Ok(())
}