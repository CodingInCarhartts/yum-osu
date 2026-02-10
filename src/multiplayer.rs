//! Multiplayer module for game state synchronization and coordination
//! Handles real-time gameplay synchronization between multiple players

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;
use anyhow::Result;

use crate::network::{NetworkMessage, PlayerInfo, Room};
use crate::game::Circle;

/// Multiplayer game state for synchronization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiplayerGameState {
    pub game_id: Uuid,
    pub room_id: Uuid,
    pub song_name: String,
    pub is_active: bool,
    pub started_at: Option<f64>,
    pub players: HashMap<Uuid, PlayerGameState>,
    pub circles: Vec<CircleSync>,
    pub seed: u64,
}

/// Individual player's game state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerGameState {
    pub user_id: Uuid,
    pub username: String,
    pub score: u32,
    pub combo: u32,
    pub max_combo: u32,
    pub accuracy: f64,
    pub health: f32,
    pub hits: HitStats,
    pub rank: u32,
    pub is_finished: bool,
}

/// Hit statistics for a player
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HitStats {
    pub perfect: u32,
    pub good: u32,
    pub ok: u32,
    pub miss: u32,
}

impl Default for HitStats {
    fn default() -> Self {
        Self {
            perfect: 0,
            good: 0,
            ok: 0,
            miss: 0,
        }
    }
}

impl Default for PlayerGameState {
    fn default() -> Self {
        Self {
            user_id: Uuid::new_v4(),
            username: String::new(),
            score: 0,
            combo: 0,
            max_combo: 0,
            accuracy: 100.0,
            health: 100.0,
            hits: HitStats::default(),
            rank: 1,
            is_finished: false,
        }
    }
}

/// Synchronized circle data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircleSync {
    pub circle_id: u32,
    pub spawn_time: f64,
    pub hit_time: Option<f64>,
    pub hit_by: Option<Uuid>,
    pub missed_by: Vec<Uuid>,
}

/// Event from a player during gameplay
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type", content = "data")]
pub enum GameEvent {
    Hit {
        player_id: Uuid,
        circle_id: u32,
        score: u16,
        timestamp: f64,
    },
    Miss {
        player_id: Uuid,
        circle_id: u32,
        timestamp: f64,
    },
    ComboBreak {
        player_id: Uuid,
        timestamp: f64,
    },
    GameFinished {
        player_id: Uuid,
        final_score: u32,
        final_accuracy: f64,
        timestamp: f64,
    },
}

/// Multiplayer game coordinator
#[derive(Debug, Clone)]
pub struct GameCoordinator {
    active_games: Arc<RwLock<HashMap<Uuid, MultiplayerGameState>>>,
    game_rooms: Arc<RwLock<HashMap<Uuid, Uuid>>>, // room_id -> game_id
    event_channels: Arc<RwLock<HashMap<Uuid, mpsc::UnboundedSender<GameEvent>>>>,
}

impl GameCoordinator {
    /// Create a new game coordinator
    pub fn new() -> Self {
        Self {
            active_games: Arc::new(RwLock::new(HashMap::new())),
            game_rooms: Arc::new(RwLock::new(HashMap::new())),
            event_channels: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new multiplayer game from a room
    pub async fn create_game(&self, room: &Room, seed: u64, song_name: String) -> Result<Uuid> {
        let game_id = Uuid::new_v4();

        let mut players = HashMap::new();
        for (user_id, player_info) in &room.players {
            players.insert(*user_id, PlayerGameState {
                user_id: *user_id,
                username: player_info.username.clone(),
                score: 0,
                combo: 0,
                max_combo: 0,
                accuracy: 100.0,
                health: 100.0,
                hits: HitStats::default(),
                rank: player_info.rank,
                is_finished: false,
            });
        }

        let game_state = MultiplayerGameState {
            game_id,
            room_id: room.room_id,
            song_name,
            is_active: false,
            started_at: None,
            players,
            circles: Vec::new(),
            seed,
        };

        self.active_games.write().await.insert(game_id, game_state);
        self.game_rooms.write().await.insert(room.room_id, game_id);

        Ok(game_id)
    }

    /// Start a game
    pub async fn start_game(&self, game_id: Uuid, start_time: f64) -> Result<()> {
        let mut games = self.active_games.write().await;
        if let Some(game) = games.get_mut(&game_id) {
            game.is_active = true;
            game.started_at = Some(start_time);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Game not found"))
        }
    }

    /// Add circles to game
    pub async fn add_circles(&self, game_id: Uuid, circles: Vec<Circle>) -> Result<()> {
        let mut games = self.active_games.write().await;
        if let Some(game) = games.get_mut(&game_id) {
            game.circles = circles.iter().enumerate()
                .map(|(idx, c)| CircleSync {
                    circle_id: idx as u32,
                    spawn_time: c.spawn_time,
                    hit_time: None,
                    hit_by: None,
                    missed_by: Vec::new(),
                })
                .collect();
            Ok(())
        } else {
            Err(anyhow::anyhow!("Game not found"))
        }
    }

    /// Process a game event
    pub async fn process_event(&self, event: GameEvent, game_id: Uuid) -> Result<()> {
        match event {
            GameEvent::Hit { player_id, circle_id, score, timestamp } => {
                self.handle_hit(game_id, player_id, circle_id, score, timestamp).await?;
            }
            GameEvent::Miss { player_id, circle_id, timestamp } => {
                self.handle_miss(game_id, player_id, circle_id, timestamp).await?;
            }
            GameEvent::ComboBreak { player_id, timestamp } => {
                self.handle_combo_break(game_id, player_id).await?;
            }
            GameEvent::GameFinished { player_id, final_score, final_accuracy, timestamp } => {
                self.handle_game_finished(game_id, player_id, final_score, final_accuracy).await?;
            }
        }

        // Recalculate rankings
        self.update_rankings(game_id).await?;

        Ok(())
    }

    /// Handle a hit event
    async fn handle_hit(&self, game_id: Uuid, player_id: Uuid, circle_id: u32, score: u16, _timestamp: f64) -> Result<()> {
        let mut games = self.active_games.write().await;
        if let Some(game) = games.get_mut(&game_id) {
            // Update player state
            if let Some(player) = game.players.get_mut(&player_id) {
                player.score += score as u32;
                player.combo += 1;
                player.max_combo = player.max_combo.max(player.combo);

                // Update hit stats
                match score {
                    300 => player.hits.perfect += 1,
                    100 => player.hits.good += 1,
                    50 => player.hits.ok += 1,
                    _ => {}
                }

                // Recalculate accuracy
                let total_hits = player.hits.perfect + player.hits.good + player.hits.ok + player.hits.miss;
                if total_hits > 0 {
                    player.accuracy = (player.hits.perfect as f64 * 300.0 +
                                     player.hits.good as f64 * 100.0 +
                                     player.hits.ok as f64 * 50.0) /
                                     (total_hits as f64 * 300.0) * 100.0;
                }
            }

            // Update circle state
            if let Some(circle) = game.circles.get_mut(circle_id as usize) {
                circle.hit_time = Some(_timestamp);
                circle.hit_by = Some(player_id);
            }
        }

        Ok(())
    }

    /// Handle a miss event
    async fn handle_miss(&self, game_id: Uuid, player_id: Uuid, circle_id: u32, _timestamp: f64) -> Result<()> {
        let mut games = self.active_games.write().await;
        if let Some(game) = games.get_mut(&game_id) {
            // Update player state
            if let Some(player) = game.players.get_mut(&player_id) {
                player.combo = 0;
                player.hits.miss += 1;
                player.health = (player.health - 10.0).max(0.0);

                // Recalculate accuracy
                let total_hits = player.hits.perfect + player.hits.good + player.hits.ok + player.hits.miss;
                if total_hits > 0 {
                    player.accuracy = (player.hits.perfect as f64 * 300.0 +
                                     player.hits.good as f64 * 100.0 +
                                     player.hits.ok as f64 * 50.0) /
                                     (total_hits as f64 * 300.0) * 100.0;
                }
            }

            // Update circle state
            if let Some(circle) = game.circles.get_mut(circle_id as usize) {
                circle.missed_by.push(player_id);
            }
        }

        Ok(())
    }

    /// Handle a combo break
    async fn handle_combo_break(&self, game_id: Uuid, player_id: Uuid) -> Result<()> {
        let mut games = self.active_games.write().await;
        if let Some(game) = games.get_mut(&game_id) {
            if let Some(player) = game.players.get_mut(&player_id) {
                player.combo = 0;
            }
        }
        Ok(())
    }

    /// Handle game finished
    async fn handle_game_finished(&self, game_id: Uuid, player_id: Uuid, final_score: u32, final_accuracy: f64) -> Result<()> {
        let mut games = self.active_games.write().await;
        if let Some(game) = games.get_mut(&game_id) {
            if let Some(player) = game.players.get_mut(&player_id) {
                player.is_finished = true;
                player.score = final_score;
                player.accuracy = final_accuracy;
            }
        }
        Ok(())
    }

    /// Update player rankings
    async fn update_rankings(&self, game_id: Uuid) -> Result<()> {
        let mut games = self.active_games.write().await;
        if let Some(game) = games.get_mut(&game_id) {
            let mut ranked_players: Vec<_> = game.players.values().collect();
            ranked_players.sort_by(|a, b| b.score.cmp(&a.score));

            for (idx, player_info) in ranked_players.iter().enumerate() {
                if let Some(player) = game.players.get_mut(&player_info.user_id) {
                    player.rank = (idx + 1) as u32;
                }
            }
        }
        Ok(())
    }

    /// Get current game state
    pub async fn get_game_state(&self, game_id: Uuid) -> Option<MultiplayerGameState> {
        self.active_games.read().await.get(&game_id).cloned()
    }

    /// Get player's state in a game
    pub async fn get_player_state(&self, game_id: Uuid, player_id: Uuid) -> Option<PlayerGameState> {
        let games = self.active_games.read().await;
        games.get(&game_id)?.players.get(&player_id).cloned()
    }

    /// Check if game is finished (all players done)
    pub async fn is_game_finished(&self, game_id: Uuid) -> bool {
        let games = self.active_games.read().await;
        if let Some(game) = games.get(&game_id) {
            game.players.values().all(|p| p.is_finished)
        } else {
            false
        }
    }

    /// End a game and return results
    pub async fn end_game(&self, game_id: Uuid) -> Option<MultiplayerGameState> {
        let mut games = self.active_games.write().await;
        if let Some(mut game) = games.remove(&game_id) {
            game.is_active = false;
            Some(game)
        } else {
            None
        }
    }

    /// Get game ID from room ID
    pub async fn get_game_id_from_room(&self, room_id: Uuid) -> Option<Uuid> {
        *self.game_rooms.read().await.get(&room_id)?
    }
}

impl Default for GameCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

/// Client-side multiplayer manager
#[derive(Debug, Clone)]
pub struct MultiplayerClient {
    game_id: Option<Uuid>,
    player_id: Uuid,
    current_score: u32,
    current_combo: u32,
    current_accuracy: f64,
}

impl MultiplayerClient {
    /// Create a new multiplayer client
    pub fn new(player_id: Uuid) -> Self {
        Self {
            game_id: None,
            player_id,
            current_score: 0,
            current_combo: 0,
            current_accuracy: 100.0,
        }
    }

    /// Join a game
    pub fn join_game(&mut self, game_id: Uuid) {
        self.game_id = Some(game_id);
    }

    /// Create hit event
    pub fn create_hit_event(&self, circle_id: u32, score: u16, timestamp: f64) -> GameEvent {
        GameEvent::Hit {
            player_id: self.player_id,
            circle_id,
            score,
            timestamp,
        }
    }

    /// Create miss event
    pub fn create_miss_event(&self, circle_id: u32, timestamp: f64) -> GameEvent {
        GameEvent::Miss {
            player_id: self.player_id,
            circle_id,
            timestamp,
        }
    }

    /// Create combo break event
    pub fn create_combo_break_event(&self, timestamp: f64) -> GameEvent {
        GameEvent::ComboBreak {
            player_id: self.player_id,
            timestamp,
        }
    }

    /// Create game finished event
    pub fn create_finished_event(&self, final_score: u32, final_accuracy: f64, timestamp: f64) -> GameEvent {
        GameEvent::GameFinished {
            player_id: self.player_id,
            final_score,
            final_accuracy,
            timestamp,
        }
    }

    /// Update local state
    pub fn update_local_state(&mut self, score: u32, combo: u32, accuracy: f64) {
        self.current_score = score;
        self.current_combo = combo;
        self.current_accuracy = accuracy;
    }
}
