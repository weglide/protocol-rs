use std::future::Future;
use tokio::{
    sync::{mpsc, oneshot},
    task::JoinHandle,
};

pub trait GatewayApi {
    fn read(&self) -> impl Future<Output = Vec<String>> + Send;
}

impl GatewayApi for mpsc::Sender<Request> {
    async fn read(&self) -> Vec<String> {
        let (tx, rx) = oneshot::channel();
        self.send(Request(tx)).await.unwrap();
        rx.await.unwrap()
    }
}

pub struct Gateway {
    tx: mpsc::Sender<Request>,
    _handle: JoinHandle<()>,
}

impl Gateway {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(1);
        let mut actor = GatewayActor::new(rx);
        let handle = tokio::spawn(async move {
            actor.run().await;
        });

        Self { tx, _handle: handle }
    }

    pub fn api(&self) -> impl GatewayApi + Send + Sync + Clone + 'static {
        self.tx.clone()
    }
}

pub struct Request(oneshot::Sender<Vec<String>>);

struct GatewayActor {
    recv: mpsc::Receiver<Request>,
    data: Vec<String>,
}

impl GatewayActor {
    pub fn new(recv: mpsc::Receiver<Request>) -> Self {
        Self { recv, data: vec![] }
    }

    pub async fn run(&mut self) {
        while let Some(Request(send)) = self.recv.recv().await {
            send.send(self.data.clone()).unwrap();
        }
    }
}
