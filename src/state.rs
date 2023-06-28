use anyhow::{anyhow, Result};
use std::{
    collections::HashMap,
    hash::Hash,
    net::TcpStream,
    sync::{Arc, Mutex},
};

use crate::{pipeline::Pipeline, tcp_connections};

#[allow(dead_code)]
pub struct Frame {
    pub data: bytes::Bytes,
}

#[allow(dead_code)]
pub struct Input {
    port: u16,
    input_id: u32,
}

#[allow(dead_code)]
pub struct Output {
    port: u16,
    output_id: u32,
}

#[allow(dead_code)]
pub struct PendingConnection {
    pub port: u16,
    pub tcp_stream: TcpStream,
}

pub struct SyncHashMap<K, V>(std::sync::Mutex<HashMap<K, V>>);

impl<K, V> SyncHashMap<K, V>
where
    K: Eq + PartialEq + Hash,
{
    pub fn new() -> Self {
        Self(Mutex::new(HashMap::new()))
    }

    pub fn insert(&self, key: K, value: V) {
        let mut map = self.0.lock().unwrap();
        map.insert(key, value);
    }

    pub fn remove(&self, key: &K) -> Option<V> {
        let mut map = self.0.lock().unwrap();
        map.remove(key)
    }
}

impl<K, V> SyncHashMap<K, V>
where
    K: Eq + PartialEq + Hash,
    V: Clone,
{
    pub fn get_cloned(&self, key: &K) -> Option<V> {
        let map = self.0.lock().unwrap();
        map.get(key).map(Clone::clone)
    }
}

pub struct State {
    pub inputs: SyncHashMap<u32, Input>,
    pub outputs: SyncHashMap<u32, Output>,
    pub pending_connections: SyncHashMap<u16, PendingConnection>,
    pub pipeline: Arc<Pipeline>,
}

impl State {
    pub fn new(pipeline: Arc<Pipeline>) -> State {
        State {
            inputs: SyncHashMap::new(),
            outputs: SyncHashMap::new(),
            pending_connections: SyncHashMap::new(),
            pipeline,
        }
    }

    pub fn add_output(&self, port: u16, output_id: u32) -> Result<()> {
        let pending_output = self
            .pending_connections
            .remove(&port)
            .ok_or_else(|| anyhow!("no pending connection for port {}", port))?;
        self.outputs.insert(output_id, Output { port, output_id });

        let sender = tcp_connections::listen_on_output(pending_output.tcp_stream);
        self.pipeline.add_output(output_id, sender);
        Ok(())
    }

    pub fn add_input(&self, port: u16, input_id: u32) -> Result<()> {
        let pending_input = self
            .pending_connections
            .remove(&port)
            .ok_or_else(|| anyhow!("no pending connection for port {}", port))?;
        self.inputs.insert(input_id, Input { port, input_id });

        self.pipeline.add_input(input_id);
        tcp_connections::listen_on_input(pending_input.tcp_stream, self.pipeline.clone(), input_id);
        Ok(())
    }
}
