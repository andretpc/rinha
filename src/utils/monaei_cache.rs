use futures::channel::oneshot::{self, Sender};
use nohash_hasher::IntMap;
use speedy::{Readable, Writable};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::io::{BufReader, BufWriter};
use tokio::net::tcp::OwnedWriteHalf;
use tokio::net::TcpStream;
use tokio::sync::RwLock;

#[derive(Debug, Readable, Writable)]
pub struct IncrementOpts {
    pub lower_limit: Option<i32>,
    pub upper_limit: Option<i32>,
    pub create: Option<bool>,
}

#[derive(Debug, Readable, Writable)]
pub enum MonaeicacheCommand<'a> {
    Set(&'a str, Value),
    Get(&'a str),
    Remove(&'a str),
    Increment(&'a str, i32, Option<IncrementOpts>),
    AddToSet(&'a str, Value, Option<usize>),
}

#[derive(Debug, Readable, Writable)]
pub struct Incrementable {
    pub value: i32,
    pub lower_limit: Option<i32>,
    pub upper_limit: Option<i32>,
}

impl TryInto<Incrementable> for Value {
    type Error = Error;

    fn try_into(self) -> Result<Incrementable, Self::Error> {
        match self {
            Self::Inc(inc) => Ok(inc),
            _ => Err(Error::TypeError),
        }
    }
}

impl TryInto<i32> for Value {
    type Error = Error;

    fn try_into(self) -> Result<i32, Self::Error> {
        match self {
            Self::Int(value) => Ok(value),
            _ => Err(Error::TypeError),
        }
    }
}

impl TryInto<bool> for Value {
    type Error = Error;

    fn try_into(self) -> Result<bool, Self::Error> {
        match self {
            Self::Bool(value) => Ok(value),
            _ => Err(Error::TypeError),
        }
    }
}

impl TryInto<u32> for Value {
    type Error = Error;

    fn try_into(self) -> Result<u32, Self::Error> {
        match self {
            Self::Nat(value) => Ok(value),
            _ => Err(Error::TypeError),
        }
    }
}

impl TryInto<String> for Value {
    type Error = Error;

    fn try_into(self) -> Result<String, Self::Error> {
        match self {
            Self::String(value) => Ok(value),
            _ => Err(Error::TypeError),
        }
    }
}

impl TryInto<Vec<Value>> for Value {
    type Error = Error;

    fn try_into(self) -> Result<Vec<Value>, Self::Error> {
        match self {
            Self::Set(value) => Ok(value),
            _ => Err(Error::TypeError),
        }
    }
}

impl Into<Value> for &str {
    fn into(self) -> Value {
        Value::String(self.to_string())
    }
}

impl Into<Value> for bool {
    fn into(self) -> Value {
        Value::Bool(self)
    }
}

impl Into<Value> for i32 {
    fn into(self) -> Value {
        Value::Int(self)
    }
}

impl Into<Value> for u32 {
    fn into(self) -> Value {
        Value::Nat(self)
    }
}

#[derive(Debug, Readable, Writable)]
pub enum Value {
    None,
    String(String),
    Nat(u32),
    Int(i32),
    Bool(bool),
    Inc(Incrementable),
    Bin(Vec<u8>),
    Set(Vec<Self>),
}

pub struct MonaeicacheClient {
    reqs: Arc<RwLock<IntMap<usize, Sender<Vec<u8>>>>>,
    message_id: AtomicUsize,
    writer: RwLock<BufWriter<OwnedWriteHalf>>,
}

#[derive(Readable, Writable, Debug, thiserror::Error)]
pub enum Error {
    #[error("Limit reached")]
    LimitReached,
    #[error("Type error")]
    TypeError,
    #[error("Not found")]
    NotFound,
}

impl Value {
    pub fn is_none(&self) -> bool {
        match self {
            Self::None => true,
            _ => false,
        }
    }

    pub fn as_str(&self) -> Option<&String> {
        match self {
            Self::String(str) => Some(str),
            _ => None,
        }
    }

    pub fn as_set(&self) -> Option<&Vec<Self>> {
        match self {
            Self::Set(set) => Some(set),
            _ => None,
        }
    }

    pub fn as_set_mut(&mut self) -> Option<&mut Vec<Self>> {
        match self {
            Self::Set(set) => Some(set),
            _ => None,
        }
    }

    pub fn as_inc_mut(&mut self) -> Option<&mut Incrementable> {
        match self {
            Self::Inc(inc) => Some(inc),
            _ => None,
        }
    }

    pub fn as_inc(&self) -> Option<&Incrementable> {
        match self {
            Self::Inc(inc) => Some(inc),
            _ => None,
        }
    }
}

impl MonaeicacheClient {
    async fn send<'a>(&self, command: MonaeicacheCommand<'a>) -> Vec<u8> {
        let message_id = self.message_id.fetch_add(1, Ordering::Relaxed);

        let message = self.build_message(message_id, command);

        let (tx, rx) = oneshot::channel::<Vec<u8>>();

        self.reqs.write().await.insert(message_id, tx);
        let mut writer = self.writer.write().await;
        writer.write(&message).await.unwrap();
        writer.flush().await.unwrap();

        rx.await.unwrap()
    }

    pub async fn set<V>(&self, key: &str, value: V) -> Value
    where
        V: Into<Value>,
    {
        Value::read_from_buffer(&self.send(MonaeicacheCommand::Set(key, value.into())).await)
            .unwrap_or(Value::None)
    }

    pub async fn get(&self, key: &str) -> Value {
        Value::read_from_buffer(&mut self.send(MonaeicacheCommand::Get(key)).await)
            .unwrap_or(Value::None)
    }

    pub async fn add_to_set<V>(&self, key: &str, value: V, size: Option<usize>)
    where
        V: Into<Value>,
    {
        self.send(MonaeicacheCommand::AddToSet(key, value.into(), size))
            .await;
    }

    pub async fn increment(
        &self,
        key: &str,
        value: i32,
        opts: Option<IncrementOpts>,
    ) -> Result<Value, Error> {
        let mut res = self
            .send(MonaeicacheCommand::Increment(key, value, opts))
            .await;
        speedy::Readable::read_from_buffer(&mut res).unwrap()
    }

    pub fn build_message(&self, message_id: usize, command: MonaeicacheCommand) -> Vec<u8> {
        let command_bytes = command.write_to_vec().unwrap();

        let message = [
            &message_id.to_le_bytes(),
            &command_bytes.len().to_le_bytes() as &[u8],
            &command_bytes,
        ]
        .concat();

        message
    }

    pub async fn new() -> Self {
        let stream = TcpStream::connect("0.0.0.0:9998").await.unwrap();

        let (reader, writer) = stream.into_split();

        let mut reader = BufReader::new(reader);
        let writer = BufWriter::new(writer);
        let reqs: Arc<RwLock<IntMap<usize, Sender<Vec<u8>>>>> =
            Arc::new(RwLock::new(IntMap::default()));

        let clone = reqs.clone();

        tokio::spawn(async move {
            loop {
                let mut req_id = [0; 8];
                match reader.read_exact(&mut req_id).await {
                    Ok(bytes_read) => {
                        if bytes_read == 0 {
                            break;
                        }

                        let mut length_buffer = [0; 8];
                        reader
                            .read_exact(&mut length_buffer)
                            .await
                            .expect("Error reading length");

                        let length = usize::from_le_bytes(length_buffer) as usize;

                        let mut data_buffer: Vec<u8> = vec![0; length];
                        reader
                            .read_exact(&mut data_buffer)
                            .await
                            .expect("Error reading data");

                        if let Some(tx) = clone.write().await.remove(&usize::from_le_bytes(req_id))
                        {
                            tx.send(data_buffer.to_vec()).unwrap();
                        }
                    }
                    Err(err) => {
                        eprintln!("Error reading from socket: {}", err);
                        break;
                    }
                }
            }
        });

        Self {
            reqs,
            message_id: AtomicUsize::new(0),
            writer: RwLock::new(writer),
        }
    }
}
