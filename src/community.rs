//! Community module for social features
//! Provides leaderboards, friends system, chat, and profiles

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::accounts::{User, UserProfile, UserStats, LeaderboardEntry, Friend, FriendStatus};

/// Chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub message_id: Uuid,
    pub sender_id: Uuid,
    pub sender_name: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub room_id: Option<Uuid>, // None for direct messages
    pub recipient_id: Option<Uuid>, // Some for direct messages
}

/// Lobby chat room
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRoom {
    pub room_id: Uuid,
    pub name: String,
    pub room_type: ChatRoomType,
    pub members: Vec<Uuid>,
    pub messages: Vec<ChatMessage>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChatRoomType {
    Public,
    Private,
    Lobby,
    Direct,
}

/// Achievement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Achievement {
    pub achievement_id: String,
    pub name: String,
    pub description: String,
    pub icon_url: Option<String>,
    pub rarity: AchievementRarity,
    pub condition: AchievementCondition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AchievementRarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "condition_type", content = "data")]
pub enum AchievementCondition {
    TotalGames { count: u32 },
    TotalScore { score: u64 },
    PerfectGame,
    FullCombo { combo: u32 },
    Accuracy { min_accuracy: f64 },
    FirstBlood,
}

/// User achievement progress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAchievement {
    pub achievement_id: String,
    pub unlocked_at: Option<DateTime<Utc>>,
    pub progress: f64,
}

/// Tournament
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tournament {
    pub tournament_id: Uuid,
    pub name: String,
    pub description: String,
    pub max_players: u32,
    pub players: Vec<Uuid>,
    pub status: TournamentStatus,
    pub created_at: DateTime<Utc>,
    pub starts_at: DateTime<Utc>,
    pub ends_at: Option<DateTime<Utc>>,
    pub rules: TournamentRules,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TournamentStatus {
    Registration,
    InProgress,
    Completed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TournamentRules {
    pub song_pool: Vec<String>,
    pub scoring_type: ScoringType,
    pub elimination_type: EliminationType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScoringType {
    ScoreV1,
    ScoreV2,
    Accuracy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EliminationType {
    SingleElimination,
    DoubleElimination,
    RoundRobin,
    Swiss,
}

/// Match in tournament
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Match {
    pub match_id: Uuid,
    pub tournament_id: Uuid,
    pub player1_id: Uuid,
    pub player2_id: Uuid,
    pub player1_score: u32,
    pub player2_score: u32,
    pub winner_id: Option<Uuid>,
    pub song: String,
    pub scheduled_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Community manager
#[derive(Debug, Clone)]
pub struct CommunityManager {
    chat_rooms: Arc<RwLock<HashMap<Uuid, ChatRoom>>>,
    achievements: Arc<RwLock<HashMap<String, Achievement>>>,
    user_achievements: Arc<RwLock<HashMap<Uuid, HashMap<String, UserAchievement>>>>,
    tournaments: Arc<RwLock<HashMap<Uuid, Tournament>>>,
    matches: Arc<RwLock<HashMap<Uuid, Match>>>,
}

impl CommunityManager {
    /// Create a new community manager
    pub fn new() -> Self {
        Self {
            chat_rooms: Arc::new(RwLock::new(HashMap::new())),
            achievements: Arc::new(RwLock::new(Self::init_achievements())),
            user_achievements: Arc::new(RwLock::new(HashMap::new())),
            tournaments: Arc::new(RwLock::new(HashMap::new())),
            matches: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Initialize default achievements
    fn init_achievements() -> HashMap<String, Achievement> {
        let mut achievements = HashMap::new();

        achievements.insert("first_game".to_string(), Achievement {
            achievement_id: "first_game".to_string(),
            name: "First Steps".to_string(),
            description: "Complete your first game".to_string(),
            icon_url: Some("achievements/first_game.png".to_string()),
            rarity: AchievementRarity::Common,
            condition: AchievementCondition::TotalGames { count: 1 },
        });

        achievements.insert("hundred_games".to_string(), Achievement {
            achievement_id: "hundred_games".to_string(),
            name: "Century Club".to_string(),
            description: "Complete 100 games".to_string(),
            icon_url: Some("achievements/hundred_games.png".to_string()),
            rarity: AchievementRarity::Rare,
            condition: AchievementCondition::TotalGames { count: 100 },
        });

        achievements.insert("million_score".to_string(), Achievement {
            achievement_id: "million_score".to_string(),
            name: "Millionaire".to_string(),
            description: "Reach 1,000,000 total score".to_string(),
            icon_url: Some("achievements/million_score.png".to_string()),
            rarity: AchievementRarity::Epic,
            condition: AchievementCondition::TotalScore { score: 1_000_000 },
        });

        achievements.insert("perfect_game".to_string(), Achievement {
            achievement_id: "perfect_game".to_string(),
            name: "Perfectionist".to_string(),
            description: "Complete a song with no misses and perfect accuracy".to_string(),
            icon_url: Some("achievements/perfect_game.png".to_string()),
            rarity: AchievementRarity::Epic,
            condition: AchievementCondition::PerfectGame,
        });

        achievements.insert("full_combo_100".to_string(), Achievement {
            achievement_id: "full_combo_100".to_string(),
            name: "Unstoppable".to_string(),
            description: "Achieve a 100x combo".to_string(),
            icon_url: Some("achievements/full_combo_100.png".to_string()),
            rarity: AchievementRarity::Rare,
            condition: AchievementCondition::FullCombo { combo: 100 },
        });

        achievements.insert("accuracy_95".to_string(), Achievement {
            achievement_id: "accuracy_95".to_string(),
            name: "Precision Master".to_string(),
            description: "Achieve 95% accuracy in a game".to_string(),
            icon_url: Some("achievements/accuracy_95.png".to_string()),
            rarity: AchievementRarity::Uncommon,
            condition: AchievementCondition::Accuracy { min_accuracy: 95.0 },
        });

        achievements
    }

    /// Create a chat room
    pub async fn create_chat_room(&self, name: String, room_type: ChatRoomType, members: Vec<Uuid>) -> Uuid {
        let room_id = Uuid::new_v4();
        let room = ChatRoom {
            room_id,
            name,
            room_type,
            members,
            messages: Vec::new(),
            created_at: Utc::now(),
        };
        self.chat_rooms.write().unwrap().insert(room_id, room);
        room_id
    }

    /// Send a message to a chat room
    pub async fn send_message(&self, room_id: Uuid, sender_id: Uuid, sender_name: String, content: String) -> Result<()> {
        let mut rooms = self.chat_rooms.write().unwrap();
        if let Some(room) = rooms.get_mut(&room_id) {
            let message = ChatMessage {
                message_id: Uuid::new_v4(),
                sender_id,
                sender_name,
                content,
                timestamp: Utc::now(),
                room_id: Some(room_id),
                recipient_id: None,
            };
            room.messages.push(message);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Chat room not found"))
        }
    }

    /// Send a direct message
    pub async fn send_direct_message(&self, sender_id: Uuid, sender_name: String, recipient_id: Uuid, content: String) {
        // Create a direct chat room if it doesn't exist
        let mut rooms = self.chat_rooms.write().unwrap();
        let room_id = Uuid::new_v4();

        let room = ChatRoom {
            room_id,
            name: format!("DM: {}", recipient_id),
            room_type: ChatRoomType::Direct,
            members: vec![sender_id, recipient_id],
            messages: Vec::new(),
            created_at: Utc::now(),
        };

        let message = ChatMessage {
            message_id: Uuid::new_v4(),
            sender_id,
            sender_name,
            content,
            timestamp: Utc::now(),
            room_id: Some(room_id),
            recipient_id: Some(recipient_id),
        };

        room.messages.push(message);
        rooms.insert(room_id, room);
    }

    /// Get messages from a chat room
    pub async fn get_messages(&self, room_id: Uuid, limit: usize) -> Vec<ChatMessage> {
        let rooms = self.chat_rooms.read().unwrap();
        if let Some(room) = rooms.get(&room_id) {
            room.messages.iter().rev().take(limit).cloned().collect()
        } else {
            Vec::new()
        }
    }

    /// Get all available achievements
    pub fn get_all_achievements(&self) -> Vec<Achievement> {
        self.achievements.read().unwrap().values().cloned().collect()
    }

    /// Get user's achievements
    pub fn get_user_achievements(&self, user_id: Uuid) -> HashMap<String, UserAchievement> {
        self.user_achievements.read().unwrap()
            .get(&user_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Check and unlock achievements based on user stats
    pub async fn check_achievements(&self, user_id: Uuid, stats: &UserStats) -> Vec<String> {
        let mut unlocked = Vec::new();
        let mut user_achievements = self.user_achievements.write().unwrap();

        // Get or create user's achievement map
        let user_map = user_achievements.entry(user_id).or_insert_with(HashMap::new);

        // Check each achievement
        for (achievement_id, achievement) in self.achievements.read().unwrap().iter() {
            // Skip if already unlocked
            if let Some(user_ach) = user_map.get(achievement_id) {
                if user_ach.unlocked_at.is_some() {
                    continue;
                }
            }

            // Check achievement condition
            let unlocked = match &achievement.condition {
                AchievementCondition::TotalGames { count } => stats.total_games >= *count,
                AchievementCondition::TotalScore { score } => stats.total_score >= *score,
                AchievementCondition::PerfectGame => stats.misses == 0 && stats.average_accuracy == 100.0,
                AchievementCondition::FullCombo { combo } => stats.highest_combo >= *combo,
                AchievementCondition::Accuracy { min_accuracy } => stats.best_accuracy >= *min_accuracy,
            };

            if unlocked {
                user_map.insert(achievement_id.clone(), UserAchievement {
                    achievement_id: achievement_id.clone(),
                    unlocked_at: Some(Utc::now()),
                    progress: 100.0,
                });
                unlocked.push(achievement.name.clone());
            }
        }

        unlocked
    }

    /// Create a tournament
    pub async fn create_tournament(
        &self,
        name: String,
        description: String,
        max_players: u32,
        starts_at: DateTime<Utc>,
        rules: TournamentRules
    ) -> Uuid {
        let tournament_id = Uuid::new_v4();
        let tournament = Tournament {
            tournament_id,
            name,
            description,
            max_players,
            players: Vec::new(),
            status: TournamentStatus::Registration,
            created_at: Utc::now(),
            starts_at,
            ends_at: None,
            rules,
        };
        self.tournaments.write().unwrap().insert(tournament_id, tournament);
        tournament_id
    }

    /// Join a tournament
    pub async fn join_tournament(&self, tournament_id: Uuid, player_id: Uuid) -> Result<()> {
        let mut tournaments = self.tournaments.write().unwrap();
        if let Some(tournament) = tournaments.get_mut(&tournament_id) {
            if tournament.status != TournamentStatus::Registration {
                return Err(anyhow::anyhow!("Tournament is not in registration phase"));
            }
            if tournament.players.len() >= tournament.max_players as usize {
                return Err(anyhow::anyhow!("Tournament is full"));
            }
            if tournament.players.contains(&player_id) {
                return Err(anyhow::anyhow!("Player already registered"));
            }

            tournament.players.push(player_id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Tournament not found"))
        }
    }

    /// Start a tournament
    pub async fn start_tournament(&self, tournament_id: Uuid) -> Result<()> {
        let mut tournaments = self.tournaments.write().unwrap();
        if let Some(tournament) = tournaments.get_mut(&tournament_id) {
            tournament.status = TournamentStatus::InProgress;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Tournament not found"))
        }
    }

    /// Create a match
    pub async fn create_match(
        &self,
        tournament_id: Uuid,
        player1_id: Uuid,
        player2_id: Uuid,
        song: String,
        scheduled_at: DateTime<Utc>
    ) -> Uuid {
        let match_id = Uuid::new_v4();
        let game_match = Match {
            match_id,
            tournament_id,
            player1_id,
            player2_id,
            player1_score: 0,
            player2_score: 0,
            winner_id: None,
            song,
            scheduled_at,
            completed_at: None,
        };
        self.matches.write().unwrap().insert(match_id, game_match);
        match_id
    }

    /// Update match score
    pub async fn update_match_score(&self, match_id: Uuid, player1_score: u32, player2_score: u32) -> Result<()> {
        let mut matches = self.matches.write().unwrap();
        if let Some(game_match) = matches.get_mut(&match_id) {
            game_match.player1_score = player1_score;
            game_match.player2_score = player2_score;

            // Determine winner
            if player1_score > player2_score {
                game_match.winner_id = Some(game_match.player1_id);
            } else if player2_score > player1_score {
                game_match.winner_id = Some(game_match.player2_id);
            }

            Ok(())
        } else {
            Err(anyhow::anyhow!("Match not found"))
        }
    }

    /// Complete a match
    pub async fn complete_match(&self, match_id: Uuid) -> Result<()> {
        let mut matches = self.matches.write().unwrap();
        if let Some(game_match) = matches.get_mut(&match_id) {
            game_match.completed_at = Some(Utc::now());
            Ok(())
        } else {
            Err(anyhow::anyhow!("Match not found"))
        }
    }

    /// Get tournament info
    pub async fn get_tournament(&self, tournament_id: Uuid) -> Option<Tournament> {
        self.tournaments.read().await.get(&tournament_id).cloned()
    }

    /// Get all active tournaments
    pub async fn get_active_tournaments(&self) -> Vec<Tournament> {
        self.tournaments.read().await.values()
            .filter(|t| t.status == TournamentStatus::Registration || t.status == TournamentStatus::InProgress)
            .cloned()
            .collect()
    }

    /// Get match info
    pub async fn get_match(&self, match_id: Uuid) -> Option<Match> {
        self.matches.read().await.get(&match_id).cloned()
    }

    /// Get player's matches
    pub async fn get_player_matches(&self, player_id: Uuid) -> Vec<Match> {
        self.matches.read().await.values()
            .filter(|m| m.player1_id == player_id || m.player2_id == player_id)
            .cloned()
            .collect()
    }
}

impl Default for CommunityManager {
    fn default() -> Self {
        Self::new()
    }
}
