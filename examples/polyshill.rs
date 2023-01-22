/// This example provided by mattf_#6820@discord, aka k#yeet🚀 polyshill on the Rust Programming Language Community Server
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::{broadcast, mpsc, oneshot};

struct Broker(mpsc::Sender<BrokerMessage>);

async fn actor(mut recv: mpsc::Receiver<BrokerMessage>) {
    let mut senders: HashMap<TypeId, Arc<dyn Any + Send + Sync>> = HashMap::new();
    while let Some(message) = recv.recv().await {
        match message {
            BrokerMessage::Subscribe {
                item_type,
                func_table,
                receiver_recv,
            } => {
                let sender = Arc::clone(
                    senders
                        .entry(item_type)
                        .or_insert_with(func_table.make_sender),
                );
                let receiver = (func_table.get_receiver_from_sender)(&*sender);
                receiver_recv.send(receiver).unwrap();
            }
            BrokerMessage::Send { item, send_item } => {
                let sender = &senders[&(&*item).type_id()];
                send_item(&**sender, item);
            }
            BrokerMessage::Close => {
                senders.clear();
            }
        }
    }
}

impl Broker {
    pub fn new() -> Self {
        let (actor_send, actor_recv) = mpsc::channel(8);
        tokio::spawn(actor(actor_recv));
        Self(actor_send)
    }

    pub async fn close(&self) {
        self.0.send(BrokerMessage::Close).await.unwrap();
    }

    pub async fn subscribe<T: Clone + Send + 'static>(&self) -> broadcast::Receiver<T> {
        fn make_sender<T: Clone + Send + 'static>() -> Arc<dyn Any + Send + Sync> {
            Arc::new(broadcast::channel::<T>(8).0)
        }

        fn get_receiver_from_sender<T: Clone + Send + 'static>(
            sender: &(dyn Any + Send + Sync),
        ) -> Box<dyn Any + Send> {
            let sender = sender.downcast_ref::<broadcast::Sender<T>>().unwrap();
            let receiver = sender.subscribe();
            Box::new(receiver)
        }

        let (receiver_send, receiver_recv) = oneshot::channel();
        self.0
            .send(BrokerMessage::Subscribe {
                item_type: TypeId::of::<T>(),
                func_table: &SubscribeFuncTable {
                    make_sender: make_sender::<T>,
                    get_receiver_from_sender: get_receiver_from_sender::<T>,
                },
                receiver_recv: receiver_send,
            })
            .await
            .unwrap();
        *receiver_recv.await.unwrap().downcast().unwrap()
    }

    pub async fn publish<T: Send + Sync + 'static>(&self, item: T) {
        fn send_item<T: 'static>(
            sender: &(dyn Any + Send + Sync),
            item: Box<dyn Any + Send + Sync>,
        ) {
            let sender = sender.downcast_ref::<broadcast::Sender<T>>().unwrap();
            let item = *item.downcast::<T>().unwrap();
            _ = sender.send(item);
        }

        self.0
            .send(BrokerMessage::Send {
                item: Box::new(item),
                send_item: send_item::<T>,
            })
            .await
            .unwrap();
    }
}

struct SubscribeFuncTable {
    make_sender: fn() -> Arc<dyn Any + Send + Sync>,
    get_receiver_from_sender: fn(&(dyn Any + Send + Sync)) -> Box<dyn Any + Send>,
}

impl std::fmt::Debug for SubscribeFuncTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SubscribeFuncTable")
            .field("make_sender", &(self.make_sender as usize as *const ()))
            .field(
                "get_receiver_from_sender",
                &(self.get_receiver_from_sender as usize as *const ()),
            )
            .finish()
    }
}

enum BrokerMessage {
    Subscribe {
        item_type: TypeId,
        func_table: &'static SubscribeFuncTable,
        receiver_recv: oneshot::Sender<Box<dyn Any + Send>>,
    },
    Send {
        item: Box<dyn Any + Send + Sync>,
        send_item: fn(&(dyn Any + Send + Sync), Box<dyn Any + Send + Sync>),
    },
    Close,
}

impl std::fmt::Debug for BrokerMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Subscribe {
                item_type,
                func_table,
                receiver_recv,
            } => f
                .debug_struct("Subscribe")
                .field("item_type", item_type)
                .field("func_table", func_table)
                .field("receiver_recv", receiver_recv)
                .finish(),
            Self::Send { item, send_item } => f
                .debug_struct("Send")
                .field("item", item)
                .field("send_item", &(*send_item as usize as *const ()))
                .finish(),
            Self::Close => f.debug_struct("Close").finish(),
        }
    }
}

//#[tokio::main(flavor = "current_thread")]
#[tokio::main]
async fn main() {
    let broker = Broker::new();
    let mut recv = broker.subscribe::<i32>().await;
    broker.publish::<i32>(3).await;
    let result = recv.recv().await.unwrap();
    assert_eq!(result, 3);
    println!("{}", result);

    let mut first_task_sub = broker.subscribe::<i64>().await;
    let mut second_task_sub = broker.subscribe::<i64>().await;

    let one = tokio::spawn(async move {
        println!("First task spawned.");
        while let Ok(result) = first_task_sub.recv().await {
            println!("First task got {}", result);
        }
        println!("First task got `None` from the channel. It is closed.");
    });

    let two = tokio::spawn(async move {
        println!("Second task spawned.");
        while let Ok(result) = second_task_sub.recv().await {
            println!("Second task got {}", result);
        }
        println!("Second task got `None` from the channel. It is closed.");
    });

    for i in 1..=10 {
        broker.publish::<i64>(i).await;
    }
    broker.close().await;
    
    let _ = tokio::join!(one, two);
}
