//! Simple WebSocket server for testing real-time synchronization

use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, WebSocketStream};
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

type PeerMap = Arc<RwLock<HashMap<String, tokio::sync::mpsc::UnboundedSender<Value>>>>;

#[tokio::main]
async fn main() {
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(addr).await.expect("Failed to bind");
    println!("WebSocket server listening on: {}", addr);

    let peers: PeerMap = Arc::new(RwLock::new(HashMap::new()));

    while let Ok((stream, addr)) = listener.accept().await {
        println!("New connection from: {}", addr);
        let peers = peers.clone();
        tokio::spawn(handle_connection(stream, peers));
    }
}

async fn handle_connection(stream: TcpStream, peers: PeerMap) {
    let ws_stream = accept_async(stream).await.expect("Failed to accept");
    handle_websocket(ws_stream, peers).await;
}

async fn handle_websocket(ws_stream: WebSocketStream<TcpStream>, peers: PeerMap) {
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    
    // Create a channel for this peer
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Value>();
    
    // Generate a unique peer ID
    let peer_id = uuid::Uuid::new_v4().to_string();
    
    // Store this peer
    {
        let mut peers_map = peers.write().await;
        peers_map.insert(peer_id.clone(), tx);
    }
    
    println!("Peer {} connected. Total peers: {}", peer_id, peers.read().await.len());
    
    // Broadcast presence to all other peers
    broadcast_to_others(&peers, &peer_id, json!({
        "type": "presence",
        "replica_id": peer_id,
        "timestamp": chrono::Utc::now().to_rfc3339()
    })).await;
    
    // Handle incoming messages from this peer
    let peers_clone = peers.clone();
    let peer_id_clone = peer_id.clone();
    
    let mut ws_sender_clone = ws_sender.clone();
    let mut ws_receiver_clone = ws_receiver.clone();
    
    // Spawn task to handle incoming WebSocket messages
    tokio::spawn(async move {
        while let Some(msg) = ws_receiver_clone.next().await {
            if let Ok(msg) = msg {
                if let tokio_tungstenite::tungstenite::Message::Text(text) = msg {
                    if let Ok(data) = serde_json::from_str::<Value>(&text) {
                        handle_message(&peers_clone, &peer_id_clone, data).await;
                    }
                }
            }
        }
    });
    
    // Spawn task to forward messages from our channel to WebSocket
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let Ok(json_str) = serde_json::to_string(&msg) {
                if let Err(e) = ws_sender_clone.send(tokio_tungstenite::tungstenite::Message::Text(json_str)).await {
                    println!("Failed to send message to peer {}: {}", peer_id, e);
                    break;
                }
            }
        }
    });
    
    // Keep connection alive
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
        
        // Send heartbeat
        let heartbeat = json!({
            "type": "heartbeat",
            "replica_id": peer_id,
            "timestamp": chrono::Utc::now().to_rfc3339()
        });
        
        if let Ok(json_str) = serde_json::to_string(&heartbeat) {
            if let Err(_) = ws_sender.send(tokio_tungstenite::tungstenite::Message::Text(json_str)).await {
                break;
            }
        }
    }
    
    // Clean up when connection is closed
    {
        let mut peers_map = peers.write().await;
        peers_map.remove(&peer_id);
    }
    
    println!("Peer {} disconnected. Total peers: {}", peer_id, peers.read().await.len());
    
    // Broadcast departure to other peers
    broadcast_to_others(&peers, &peer_id, json!({
        "type": "departure",
        "replica_id": peer_id,
        "timestamp": chrono::Utc::now().to_rfc3339()
    })).await;
}

async fn handle_message(peers: &PeerMap, from_peer: &str, message: Value) {
    if let Some(msg_type) = message.get("type").and_then(|v| v.as_str()) {
        match msg_type {
            "sync" => {
                // Broadcast sync message to all other peers
                broadcast_to_others(peers, from_peer, message).await;
                
                // Send acknowledgment back to sender
                if let Some(peer_tx) = peers.read().await.get(from_peer) {
                    let ack = json!({
                        "type": "ack",
                        "key": message.get("key"),
                        "replica_id": from_peer
                    });
                    let _ = peer_tx.send(ack);
                }
            }
            "presence" => {
                // Broadcast presence to all other peers
                broadcast_to_others(peers, from_peer, message).await;
            }
            "heartbeat" => {
                // Just log heartbeat, no need to broadcast
                println!("Heartbeat from peer: {}", from_peer);
            }
            _ => {
                println!("Unknown message type: {} from peer: {}", msg_type, from_peer);
            }
        }
    }
}

async fn broadcast_to_others(peers: &PeerMap, exclude_peer: &str, message: Value) {
    let peers_map = peers.read().await;
    
    for (peer_id, tx) in peers_map.iter() {
        if peer_id != exclude_peer {
            if let Err(e) = tx.send(message.clone()) {
                println!("Failed to send message to peer {}: {}", peer_id, e);
            }
        }
    }
}
