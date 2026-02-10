//! Network module for multiplayer functionality
//! Provides WebSocket client/server implementation for real-time gameplay

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, RwLock};
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};
use futures_util::{SinkExt, StreamExt};
use uuid::Uuid;
use anyhow::Result;

/// Represents different network messages
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum NetworkMessage {
    /// Authentication request
    Auth { username: String, password: String },
    /// Authentication response
    AuthResponse { success: bool, token: Option<String>, user_id: Option<Uuid> },
    /// Player joined lobby
    PlayerJoined { user_id: Uuid, username: String },
    /// Player left lobby
    PlayerLeft { user_id: Uuid },
    /// Game state update (sync)
    GameStateUpdate {
        player_id: Uuid,
        score: u32,
        combo: u32,
        accuracy: f64,
        health: f32,
    },
    /// Hit event
    HitEvent {
        player_id: Uuid,
        circle_id: u32,
        score: u16,
        timestamp: f64,
    },
    /// Miss event
    MissEvent {
        player_id: Uuid,
        circle_id: u32,
        timestamp: f64,
    },
    /// Game start signal
    GameStart { seed: u64 },
    /// Game end signal
    GameEnd { winner_id: Uuid, final_scores: HashMap<Uuid, u32> },
    /// Chat message
    Chat { user_id: Uuid, username: String, message: String },
    /// Lobby update
    LobbyUpdate { players: Vec<PlayerInfo> },
    /// Error message
    Error { message: String },
    /// Heartbeat
    Heartbeat,
}

/// Player information for lobby display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerInfo {
    pub user_id: Uuid,
    pub username: String,
    pub is_ready: bool,
    pub score: u32,
    pub combo: u32,
    pub accuracy: f64,
    pub rank: u32,
}

impl Default for PlayerInfo {
    fn default() -> Self {
        Self {
            user_id: Uuid::new_v4(),
            username: "Player".to_string(),
            is_ready: false,
            score: 0,
            combo: 0,
            accuracy: 0.0,
            rank: 0,
        }
    }
}

/// WebSocket client for connecting to multiplayer server
pub struct GameClient {
    sender: mpsc::UnboundedSender<NetworkMessage>,
    receiver: Arc<Mutex<mpsc::UnboundedReceiver<NetworkMessage>>>,
}

impl GameClient {
    /// Create a new game client
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self {
            sender: tx,
            receiver: Arc::new(Mutex::new(rx)),
        }
    }

    /// Connect to a multiplayer server
    pub async fn connect(&self, server_url: &str) -> Result<()> {
        let url = url::Url::parse(server_url)?;
        let (ws_stream, _) = tokio_tungstenite::connect_async(url).await?;

        let (mut write, mut read) = ws_stream.split();
        let sender = self.sender.clone();

        // Task to send messages to server
        tokio::spawn(async move {
            while let Some(msg) = sender.recv() {
                let json = serde_json::to_string(&msg).unwrap();
                if write.send(Message::Text(json)).await.is_err() {
                    break;
                }
            }
        });

        // Task to receive messages from server
        tokio::spawn(async move {
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        if let Ok(network_msg) = serde_json::from_str::<NetworkMessage>(&text) {
                            // TODO: Forward to game state
                        }
                    }
                    Ok(Message::Close(_)) => break,
                    Err(e) => eprintln!("WebSocket error: {}", e),
                    _ => {}
                }
            }
        });

        Ok(())
    }

    /// Send a network message
    pub fn send(&self, message: NetworkMessage) -> Result<()> {
        self.sender.send(message)?;
        Ok(())
    }

    /// Try to receive a message (non-blocking)
    pub fn try_recv(&self) -> Option<NetworkMessage> {
        self.receiver.lock().unwrap().try_recv().ok()
    }
}

/// Multiplayer room/lobby state
#[derive(Debug, Clone)]
pub struct Room {
    pub room_id: Uuid,
    pub host_id: Uuid,
    pub players: HashMap<Uuid, PlayerInfo>,
    pub is_game_active: bool,
    pub song_name: String,
    pub max_players: usize,
}

impl Room {
    /// Create a new room
    pub fn new(host_id: Uuid, host_name: String, max_players: usize) -> Self {
        let mut players = HashMap::new();
        players.insert(host_id, PlayerInfo {
            user_id: host_id,
            username: host_name,
            is_ready: false,
            score: 0,
            combo: 0,
            accuracy: 0.0,
            rank: 1,
        });

        Self {
            room_id: Uuid::new_v4(),
            host_id,
            players,
            is_game_active: false,
            song_name: String::new(),
            max_players,
        }
    }

    /// Add a player to the room
    pub fn add_player(&mut self, user_id: Uuid, username: String) -> Result<()> {
        if self.players.len() >= self.max_players {
            return Err(anyhow::anyhow!("Room is full"));
        }

        self.players.insert(user_id, PlayerInfo {
            user_id,
            username,
            is_ready: false,
            score: 0,
            combo: 0,
            accuracy: 0.0,
            rank: (self.players.len() + 1) as u32,
        });

        Ok(())
    }

    /// Remove a player from the room
    pub fn remove_player(&mut self, user_id: Uuid) {
        self.players.remove(&user_id);
    }

    /// Update player readiness
    pub fn set_player_ready(&mut self, user_id: Uuid, ready: bool) -> Result<()> {
        if let Some(player) = self.players.get_mut(&user_id) {
            player.is_ready = ready;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Player not found in room"))
        }
    }

    /// Check if all players are ready
    pub fn all_players_ready(&self) -> bool {
        self.players.values().all(|p| p.is_ready)
    }

    /// Get player info
    pub fn get_player(&self, user_id: Uuid) -> Option<&PlayerInfo> {
        self.players.get(&user_id)
    }

    /// Update player score during gameplay
    pub fn update_player_score(&mut self, user_id: Uuid, score: u32, combo: u32, accuracy: f64) {
        if let Some(player) = self.players.get_mut(&user_id) {
            player.score = score;
            player.combo = combo;
            player.accuracy = accuracy;

            // Update rankings
            let mut ranked: Vec<_> = self.players.values().collect();
            ranked.sort_by(|a, b| b.score.cmp(&a.score));

            for (idx, p) in ranked.iter().enumerate() {
                if let Some(player_mut) = self.players.get_mut(&p.user_id) {
                    player_mut.rank = (idx + 1) as u32;
                }
            }
        }
    }

    /// Get all players sorted by rank
    pub fn get_ranked_players(&self) -> Vec<PlayerInfo> {
        let mut players: Vec<_> = self.players.values().cloned().collect();
        players.sort_by(|a, b| a.rank.cmp(&b.rank));
        players
    }
}

/// Connection info for a connected client
#[derive(Debug)]
pub struct ClientConnection {
    pub user_id: Uuid,
    pub username: String,
    pub room_id: Option<Uuid>,
}

/// WebSocket server for multiplayer
pub struct GameServer {
    clients: Arc<RwLock<HashMap<Uuid, ClientConnection>>>,
    rooms: Arc<RwLock<HashMap<Uuid, Room>>>,
}

impl GameServer {
    /// Create a new game server
    pub fn new() -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            rooms: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start the server
    pub async fn start(&self, addr: &str) -> Result<()> {
        let listener = tokio::net::TcpListener::bind(addr).await?;
        println!("Game server listening on {}", addr);

        while let Ok((stream, addr)) = listener.accept().await {
            println!("New connection from: {}", addr);
            let clients = self.clients.clone();
            let rooms = self.rooms.clone();

            tokio::spawn(async move {
                let ws_stream = tokio_tungstenite::accept_async(stream).await?;
                let (mut write, mut read) = ws_stream.split();

                let mut user_id: Option<Uuid> = None;

                while let Some(msg) = read.next().await {
                    match msg {
                        Ok(Message::Text(text)) => {
                            if let Ok(network_msg) = serde_json::from_str::<NetworkMessage>(&text) {
                                match network_msg {
                                    NetworkMessage::Auth { username, password } => {
                                        // TODO: Implement proper authentication
                                        let new_user_id = Uuid::new_v4();
                                        user_id = Some(new_user_id);

                                        clients.write().await.insert(new_user_id, ClientConnection {
                                            user_id: new_user_id,
                                            username: username.clone(),
                                            room_id: None,
                                        });

                                        let response = NetworkMessage::AuthResponse {
                                            success: true,
                                            token: Some(format!("token_{}", new_user_id)),
                                            user_id: Some(new_user_id),
                                        };

                                        let json = serde_json::to_string(&response)?;
                                        write.send(Message::Text(json)).await?;
                                    }
                                    NetworkMessage::HitEvent { player_id, circle_id, score, timestamp } => {
                                        // Broadcast hit event to all players in room
                                        // TODO: Implement room-specific broadcasting
                                    }
                                    NetworkMessage::Chat { user_id, username, message } => {
                                        // Broadcast chat message
                                        let response = NetworkMessage::Chat { user_id, username, message };
                                        let json = serde_json::to_string(&response)?;
                                        write.send(Message::Text(json)).await?;
                                    }
                                    _ => {}
                                }
                            }
                        }
                        Ok(Message::Close(_)) => break,
                        Err(e) => eprintln!("WebSocket error: {}", e),
                        _ => {}
                    }
                }

                // Cleanup on disconnect
                if let Some(id) = user_id {
                    clients.write().await.remove(&id);
                }

                Ok::<(), anyhow::Error>(())
            });
        }

        Ok(())
    }

    /// Create a new room
    pub async fn create_room(&self, host_id: Uuid, host_name: String, max_players: usize) -> Uuid {
        let room = Room::new(host_id, host_name, max_players);
        let room_id = room.room_id;
        self.rooms.write().await.insert(room_id, room);

        // Update client's room
        if let Some(client) = self.clients.write().await.get_mut(&host_id) {
            client.room_id = Some(room_id);
        }

        room_id
    }

    /// Join a room
    pub async fn join_room(&self, room_id: Uuid, user_id: Uuid, username: String) -> Result<()> {
        let mut rooms = self.rooms.write().await;
        if let Some(room) = rooms.get_mut(&room_id) {
            room.add_player(user_id, username)?;

            // Update client's room
            let mut clients = self.clients.write().await;
            if let Some(client) = clients.get_mut(&user_id) {
                client.room_id = Some(room_id);
            }

            Ok(())
        } else {
            Err(anyhow::anyhow!("Room not found"))
        }
    }

    /// Get room info
    pub async fn get_room(&self, room_id: Uuid) -> Option<Room> {
        self.rooms.read().await.get(&room_id).cloned()
    }

    /// Get all active rooms
    pub async fn get_all_rooms(&self) -> Vec<Room> {
        self.rooms.read().await.values().cloned().collect()
    }
}
