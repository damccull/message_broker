use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::mpsc::Sender, error::Report,
};

use tokio::sync::{mpsc, oneshot};

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
        match &message {
            MessageBrokerControlMessage::Publish(m) => {
                dbg!(m);
            }
            MessageBrokerControlMessage::Subscribe {
                type_id,
                respond_to,
            } => {
                dbg!(&message,&type_id);
                let (tx,rx) = mpsc::channel(10);
                respond_to.send(Box::new(rx));
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
        let boxed = Box::new(message);
        let x = self
            .sender
            .send(MessageBrokerControlMessage::Publish(boxed))
            .await;

        if x.is_err() {
            anyhow::bail!("Unable to send the message to the actor: {:?}", x);
        }

        Ok(())
    }

    pub async fn subscribe(&self, message_type: TypeId) -> Result<(), anyhow::Error> {
        let (tx, rx) = oneshot::channel();
        let x = self
            .sender
            .send(MessageBrokerControlMessage::Subscribe {
                type_id: TypeId::of::<TestEvent>(),
                respond_to: tx,
            })
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
    Publish(Box<dyn Any + Send + Sync>),
    Subscribe {
        type_id: TypeId,
        respond_to: oneshot::Sender<mpsc::Receiver<Box<dyn Any + Send + Sync>>>,
    },
}

#[derive(Debug)]
pub enum TestEvent {
    A,
    B,
}

#[cfg(test)]
mod tests {}
