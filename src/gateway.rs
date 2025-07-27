use std::{fs, future::Future};
use tokio::{
    sync::{mpsc, oneshot},
    task::JoinHandle,
    time::{self, Duration},
};

pub trait GatewayApi {
    fn read(&self) -> impl Future<Output = Vec<String>> + Send;
    fn subscribe_sse(&self) -> impl Future<Output = mpsc::Receiver<Vec<String>>> + Send;
    fn get_sse_count(&self) -> impl Future<Output = usize> + Send;
}

impl GatewayApi for mpsc::Sender<GatewayMessage> {
    async fn read(&self) -> Vec<String> {
        let (tx, rx) = oneshot::channel();
        self.send(GatewayMessage::Read(tx)).await.unwrap();
        rx.await.unwrap()
    }

    async fn subscribe_sse(&self) -> mpsc::Receiver<Vec<String>> {
        let (tx, rx) = oneshot::channel();
        self.send(GatewayMessage::SubscribeSse(tx)).await.unwrap();
        rx.await.unwrap()
    }

    async fn get_sse_count(&self) -> usize {
        let (tx, rx) = oneshot::channel();
        self.send(GatewayMessage::GetSseCount(tx)).await.unwrap();
        rx.await.unwrap()
    }
}

pub struct Gateway {
    tx: mpsc::Sender<GatewayMessage>,
    _handle: JoinHandle<()>,
}

impl Gateway {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(100);
        let mut actor = GatewayActor::new(rx);
        let handle = tokio::spawn(async move {
            actor.run().await;
        });

        Self {
            tx,
            _handle: handle,
        }
    }

    pub fn api(&self) -> impl GatewayApi + Send + Sync + Clone + 'static {
        self.tx.clone()
    }
}

pub enum GatewayMessage {
    Read(oneshot::Sender<Vec<String>>),
    SubscribeSse(oneshot::Sender<mpsc::Receiver<Vec<String>>>),
    GetSseCount(oneshot::Sender<usize>),
}

// For backwards compatibility
pub type Request = oneshot::Sender<Vec<String>>;

struct GatewayActor {
    recv: mpsc::Receiver<GatewayMessage>,
    data: Vec<String>,
    sse_senders: Vec<mpsc::Sender<Vec<String>>>,
}

impl GatewayActor {
    pub fn new(recv: mpsc::Receiver<GatewayMessage>) -> Self {
        let data = Self::load_data_from_file();
        Self {
            recv,
            data,
            sse_senders: Vec::new(),
        }
    }

    fn load_data_from_file() -> Vec<String> {
        let content = fs::read_to_string("data.txt").unwrap();
        let lines: Vec<String> = content
            .lines()
            .map(|line| line.trim().to_string())
            .collect();

        assert_eq!(lines.len(), 500);

        lines
    }

    pub async fn run(&mut self) {
        let mut interval = time::interval(Duration::from_millis(500));

        loop {
            tokio::select! {
                Some(msg) = self.recv.recv() => {
                    self.handle_message(msg).await;
                }
                _ = interval.tick() => {
                    self.push_to_sse_streams().await;
                }
                else => break,
            }
        }
    }

    async fn handle_message(&mut self, msg: GatewayMessage) {
        match msg {
            GatewayMessage::Read(sender) => {
                let _ = sender.send(self.data.clone());
            }
            GatewayMessage::SubscribeSse(sender) => {
                let (tx, rx) = mpsc::channel(10);
                self.sse_senders.push(tx);
                let _ = sender.send(rx);
            }
            GatewayMessage::GetSseCount(sender) => {
                let _ = sender.send(self.sse_senders.len());
            }
        }
    }

    async fn push_to_sse_streams(&mut self) {
        let data = self.data.clone();
        let mut still_connected = Vec::new();

        for sender in self.sse_senders.drain(..) {
            if sender.send(data.clone()).await.is_ok() {
                // Stream is still connected
                still_connected.push(sender);
            }
            // If send fails, the receiver was dropped, so we don't re-add it
        }

        self.sse_senders = still_connected;
    }
}
