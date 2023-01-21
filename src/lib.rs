use std::{any::{TypeId, Any}, collections::HashMap, sync::{Arc, mpsc::Sender}};

use tokio::sync::mpsc;

pub struct Actor {
    receiver: mpsc::Receiver<MessageBrokerControlMessage>,
    subscriptions: HashMap<TypeId, Vec<Sender<Box<dyn Any + Send + Sync>>>>,
}

impl Actor {
    fn new(receiver: mpsc::Receiver<MessageBrokerControlMessage>) -> Self {
        Self {
            receiver,
            subscriptions: HashMap::default(),
        }
    }

    fn handle_message(&self, message: MessageBrokerControlMessage) {
        match message {
            MessageBrokerControlMessage::Publish(m) => {
                dbg!(m);
            }
            MessageBrokerControlMessage::Subscribe => {
                dbg!(message);
            }
        }
    }
}

pub struct Handle {
    sender: mpsc::Sender<MessageBrokerControlMessage>,
}

impl Handle {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(10);
        let actor = Actor::new(rx);

        tokio::spawn(run_actor(actor));

        Self { sender: tx }
    }

    pub async fn publish<MessageType>(&self, message: MessageType) -> Result<(), anyhow::Error>
    where
        MessageType: Any + Send + Sync,
    {
        let arced = Arc::new(message);
        let x = self
            .sender
            .send(MessageBrokerControlMessage::Publish(arced))
            .await;

        if x.is_err() {
            anyhow::bail!("Unable to send the message to the actor: {:?}", x);
        }

        Ok(())
    }

    pub async fn subscribe(&self) -> Result<(), anyhow::Error> {
        let x = self
            .sender
            .send(MessageBrokerControlMessage::Subscribe)
            .await;

        if x.is_err() {
            anyhow::bail!("Unable to send the message to the actor: {:?}", x);
        }

        Ok(())
    }
}

pub async fn run_actor(mut actor: Actor) {
    while let Some(msg) = actor.receiver.recv().await {
        actor.handle_message(msg);
    }
}

#[derive(Debug)]
pub enum MessageBrokerControlMessage {
    Publish(Arc<dyn Any + Send + Sync>),
    Subscribe,
}

#[cfg(test)]
mod tests {}
