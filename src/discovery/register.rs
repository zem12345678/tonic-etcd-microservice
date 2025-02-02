// use etcd_client::{ Client, LeaseGrantResponse,LeaseKeepAliveResponse};
// struct Register {
//     etcd_addr:(str),
//     dial_timeout:i64,
//     lease:LeaseGrantResponse,
//     keep_alive: LeaseKeepAliveResponse,
//     // srvInfo Server
//     server_ttl:i64,
//     etcd_client:&mut Client,
// }
//
// impl Register {
//     fn new(etcd_addr:(str), dial_timeout:i64) -> Self {
//         // ETCD 客户端
//         self.etcd_client = Client::connect(["localhost:2379"], None).await?;
//         self.dial_timeout = dial_timeout;
//
//     }
//     fn register(&self) -> Box<dyn std::error::Error>{
//
//         let lease = self.etcd_client.lease_client().grant(self.server_ttl, None).await?;
//
//         self.etcd_client.put(service_name, etcd_addr, Some(PutOptions::new().with_lease(lease.id())))
//             .await?;
//
//         let(mut lease_keeper,  lease_keep_alive_stream) = self.etcd_client.lease_keep_alive(lease_id).await?;
//
//         lease_keeper.keep_alive().await?;
//
//         self.keep_alive = lease_keep_alive_stream.clone();
//
//         tokio::spawn(async move {
//             loop {
//                 match self.keep_alive.message().await {
//                     Ok(Some(_)) => {
//                         // 如果有消息，表示租约仍然有效，可以继续续约
//                         lease_keeper.keep_alive().await.unwrap();
//                         println!("Service keeper updated");
//                     }
//                     Ok(None) => {
//                         // 如果没有收到消息，表示连接断开，需要重新连接
//                         self.etcd_client.put(service_name, self.etcd_addr, Some(PutOptions::new().with_lease(self.lease.id())))
//                             .await?;
//                         println!("Lease keep-alive stream closed, reconnecting...");
//                         break;
//                     }
//                     Err(e) => {
//                         // 处理错误，连接可能会断开
//                         eprintln!("Error in keep-alive stream: {:?}", e);
//                         break;
//                     }
//                 }
//
//                 // 等待一段时间再进行续约
//                 tokio::time::sleep(std::time::Duration::from_secs(5)).await;
//             }
//         });
//     }
// }