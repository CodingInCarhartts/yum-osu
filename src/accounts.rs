//! Accounts module for user authentication and management
//! Provides user registration, login, session management, and profiles

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use anyhow::Result;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{rand_core::OsRng, SaltString};

/// User account information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub user_id: Uuid,
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub is_online: bool,
    pub profile: UserProfile,
    pub stats: UserStats,
    pub settings: UserSettings,
}

/// User profile information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub display_name: String,
    pub bio: String,
    pub avatar_url: Option<String>,
    pub country: String,
    pub rank: u32,
    pub global_rank: u32,
}

impl Default for UserProfile {
    fn default() -> Self {
        Self {
            display_name: String::new(),
            bio: String::new(),
            avatar_url: None,
            country: "Unknown".to_string(),
            rank: 0,
            global_rank: 0,
        }
    }
}

/// User gameplay statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStats {
    pub total_games: u32,
    pub total_score: u64,
    pub highest_combo: u32,
    pub perfect_hits: u32,
    pub good_hits: u32,
    pub ok_hits: u32,
    pub misses: u32,
    pub play_time_seconds: u64,
    pub average_accuracy: f64,
    pub best_accuracy: f64,
    pub songs_played: HashMap<String, SongStats>,
}

/// Statistics for a specific song
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongStats {
    pub plays: u32,
    pub high_score: u32,
    pub best_combo: u32,
    pub best_accuracy: f64,
    pub grade_counts: HashMap<String, u32>,
}

impl Default for SongStats {
    fn default() -> Self {
        Self {
            plays: 0,
            high_score: 0,
            best_combo: 0,
            best_accuracy: 0.0,
            grade_counts: HashMap::new(),
        }
    }
}

impl Default for UserStats {
    fn default() -> Self {
        Self {
            total_games: 0,
            total_score: 0,
            highest_combo: 0,
            perfect_hits: 0,
            good_hits: 0,
            ok_hits: 0,
            misses: 0,
            play_time_seconds: 0,
            average_accuracy: 0.0,
            best_accuracy: 0.0,
            songs_played: HashMap::new(),
        }
    }
}

/// User-specific settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettings {
    pub public_profile: bool,
    pub show_online_status: bool,
    pub allow_friend_requests: bool,
    pub receive_notifications: bool,
    pub preferred_skin: Option<String>,
    pub preferred_difficulty: String,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            public_profile: true,
            show_online_status: true,
            allow_friend_requests: true,
            receive_notifications: true,
            preferred_skin: None,
            preferred_difficulty: "Normal".to_string(),
        }
    }
}

impl User {
    /// Create a new user
    pub fn new(username: String, password: &str, email: String) -> Result<Self> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2.hash_password(password.as_bytes(), &salt)?.to_string();

        Ok(Self {
            user_id: Uuid::new_v4(),
            username: username.clone(),
            password_hash,
            email,
            created_at: Utc::now(),
            last_login: None,
            is_online: false,
            profile: UserProfile {
                display_name: username,
                ..Default::default()
            },
            stats: UserStats::default(),
            settings: UserSettings::default(),
        })
    }

    /// Verify password
    pub fn verify_password(&self, password: &str) -> Result<bool> {
        let parsed_hash = PasswordHash::new(&self.password_hash)?;
        let argon2 = Argon2::default();
        Ok(argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok())
    }

    /// Update user stats after a game
    pub fn update_stats(&mut self, score: u32, combo: u32, accuracy: f64, song_name: String, play_time: u64) {
        self.stats.total_games += 1;
        self.stats.total_score += score as u64;
        self.stats.highest_combo = self.stats.highest_combo.max(combo);
        self.stats.play_time_seconds += play_time;

        // Update average accuracy
        let total_acc = self.stats.average_accuracy * (self.stats.total_games - 1) as f64;
        self.stats.average_accuracy = (total_acc + accuracy) / self.stats.total_games as f64;
        self.stats.best_accuracy = self.stats.best_accuracy.max(accuracy);

        // Update song-specific stats
        let song_stats = self.stats.songs_played.entry(song_name).or_default();
        song_stats.plays += 1;
        song_stats.high_score = song_stats.high_score.max(score);
        song_stats.best_combo = song_stats.best_combo.max(combo);
        song_stats.best_accuracy = song_stats.best_accuracy.max(accuracy);
    }

    /// Update hit statistics
    pub fn update_hits(&mut self, perfect: u32, good: u32, ok: u32, misses: u32) {
        self.stats.perfect_hits += perfect;
        self.stats.good_hits += good;
        self.stats.ok_hits += ok;
        self.stats.misses += misses;
    }

    /// Update last login time
    pub fn update_last_login(&mut self) {
        self.last_login = Some(Utc::now());
        self.is_online = true;
    }

    /// Set online status
    pub fn set_online(&mut self, online: bool) {
        self.is_online = online;
    }
}

/// Session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub token: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub ip_address: Option<String>,
}

impl Session {
    /// Create a new session
    pub fn new(user_id: Uuid, ip_address: Option<String>) -> Self {
        let now = Utc::now();
        let expires_at = now + chrono::Duration::days(30);

        Self {
            session_id: Uuid::new_v4(),
            user_id,
            token: format!("session_{}_{:?}", user_id, Uuid::new_v4()),
            created_at: now,
            expires_at,
            ip_address,
        }
    }

    /// Check if session is expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
}

/// Friend relationship
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Friend {
    pub friend_id: Uuid,
    pub username: String,
    pub status: FriendStatus,
    pub added_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FriendStatus {
    Pending,
    Accepted,
    Blocked,
}

/// Account manager for handling users, sessions, and friends
#[derive(Debug, Clone)]
pub struct AccountManager {
    users: Arc<RwLock<HashMap<Uuid, User>>>,
    username_to_id: Arc<RwLock<HashMap<String, Uuid>>>,
    sessions: Arc<RwLock<HashMap<String, Session>>>,
    friends: Arc<RwLock<HashMap<Uuid, Vec<Friend>>>>,
    leaderboard: Arc<RwLock<Vec<LeaderboardEntry>>>,
    data_path: PathBuf,
}

/// Leaderboard entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardEntry {
    pub user_id: Uuid,
    pub username: String,
    pub rank: u32,
    pub total_score: u64,
    pub average_accuracy: f64,
    pub total_games: u32,
}

impl AccountManager {
    /// Create a new account manager
    pub fn new(data_path: PathBuf) -> Self {
        Self {
            users: Arc::new(RwLock::new(HashMap::new())),
            username_to_id: Arc::new(RwLock::new(HashMap::new())),
            sessions: Arc::new(RwLock::new(HashMap::new())),
            friends: Arc::new(RwLock::new(HashMap::new())),
            leaderboard: Arc::new(RwLock::new(Vec::new())),
            data_path,
        }
    }

    /// Register a new user
    pub async fn register(&self, username: String, password: String, email: String) -> Result<Uuid> {
        // Check if username already exists
        {
            let username_map = self.username_to_id.read().unwrap();
            if username_map.contains_key(&username) {
                return Err(anyhow::anyhow!("Username already exists"));
            }
        }

        // Create new user
        let user = User::new(username.clone(), &password, email)?;
        let user_id = user.user_id;

        // Store user
        self.users.write().unwrap().insert(user_id, user.clone());
        self.username_to_id.write().unwrap().insert(username, user_id);

        // Save to disk
        self.save_data()?;

        Ok(user_id)
    }

    /// Login user
    pub async fn login(&self, username: String, password: String, ip_address: Option<String>) -> Result<Session> {
        // Find user
        let user_id = {
            let username_map = self.username_to_id.read().unwrap();
            username_map.get(&username)
                .copied()
                .ok_or_else(|| anyhow::anyhow!("User not found"))?
        };

        let mut users = self.users.write().unwrap();
        let user = users.get_mut(&user_id)
            .ok_or_else(|| anyhow::anyhow!("User not found"))?;

        // Verify password
        if !user.verify_password(&password)? {
            return Err(anyhow::anyhow!("Invalid password"));
        }

        // Update last login
        user.update_last_login();

        // Create session
        let session = Session::new(user_id, ip_address);
        self.sessions.write().unwrap().insert(session.token.clone(), session.clone());

        Ok(session)
    }

    /// Logout user
    pub async fn logout(&self, token: String) -> Result<()> {
        self.sessions.write().unwrap().remove(&token);
        Ok(())
    }

    /// Validate session
    pub async fn validate_session(&self, token: &str) -> Result<Uuid> {
        let sessions = self.sessions.read().unwrap();
        let session = sessions.get(token)
            .ok_or_else(|| anyhow::anyhow!("Invalid session"))?;

        if session.is_expired() {
            return Err(anyhow::anyhow!("Session expired"));
        }

        Ok(session.user_id)
    }

    /// Get user by ID
    pub async fn get_user(&self, user_id: Uuid) -> Option<User> {
        self.users.read().unwrap().get(&user_id).cloned()
    }

    /// Get user by username
    pub async fn get_user_by_username(&self, username: &str) -> Option<User> {
        let username_map = self.username_to_id.read().unwrap();
        let user_id = username_map.get(username)?;
        self.users.read().unwrap().get(user_id).cloned()
    }

    /// Update user profile
    pub async fn update_profile(&self, user_id: Uuid, profile: UserProfile) -> Result<()> {
        let mut users = self.users.write().unwrap();
        if let Some(user) = users.get_mut(&user_id) {
            user.profile = profile;
            self.save_data()?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("User not found"))
        }
    }

    /// Send friend request
    pub async fn send_friend_request(&self, requester_id: Uuid, target_username: String) -> Result<()> {
        let target_id = {
            let username_map = self.username_to_id.read().unwrap();
            username_map.get(&target_username)
                .copied()
                .ok_or_else(|| anyhow::anyhow!("User not found"))?
        };

        let mut friends = self.friends.write().unwrap();
        friends.entry(requester_id).or_insert_with(Vec::new).push(Friend {
            friend_id: target_id,
            username: target_username,
            status: FriendStatus::Pending,
            added_at: Utc::now(),
        });

        Ok(())
    }

    /// Accept friend request
    pub async fn accept_friend_request(&self, user_id: Uuid, friend_id: Uuid) -> Result<()> {
        let mut friends = self.friends.write().unwrap();

        // Update requester's friend list
        if let Some(friend_list) = friends.get_mut(&user_id) {
            if let Some(friend) = friend_list.iter_mut().find(|f| f.friend_id == friend_id) {
                friend.status = FriendStatus::Accepted;
            }
        }

        // Add to friend's list
        let target_user = self.users.read().unwrap().get(&friend_id)
            .ok_or_else(|| anyhow::anyhow!("User not found"))?;
        friends.entry(friend_id).or_insert_with(Vec::new).push(Friend {
            friend_id: user_id,
            username: target_user.username.clone(),
            status: FriendStatus::Accepted,
            added_at: Utc::now(),
        });

        Ok(())
    }

    /// Get friends list
    pub async fn get_friends(&self, user_id: Uuid) -> Vec<Friend> {
        self.friends.read().unwrap()
            .get(&user_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Update leaderboard
    pub async fn update_leaderboard(&self) {
        let users = self.users.read().unwrap();
        let mut entries: Vec<LeaderboardEntry> = users.values()
            .map(|user| LeaderboardEntry {
                user_id: user.user_id,
                username: user.username.clone(),
                rank: 0,
                total_score: user.stats.total_score,
                average_accuracy: user.stats.average_accuracy,
                total_games: user.stats.total_games,
            })
            .collect();

        // Sort by total score
        entries.sort_by(|a, b| b.total_score.cmp(&a.total_score));

        // Assign ranks
        for (idx, entry) in entries.iter_mut().enumerate() {
            entry.rank = (idx + 1) as u32;
        }

        *self.leaderboard.write().unwrap() = entries;
    }

    /// Get leaderboard
    pub async fn get_leaderboard(&self, limit: usize) -> Vec<LeaderboardEntry> {
        let leaderboard = self.leaderboard.read().unwrap();
        leaderboard.iter().take(limit).cloned().collect()
    }

    /// Save data to disk
    fn save_data(&self) -> Result<()> {
        std::fs::create_dir_all(&self.data_path)?;

        let users = self.users.read().unwrap();
        let users_json = serde_json::to_string_pretty(&*users)?;
        std::fs::write(self.data_path.join("users.json"), users_json)?;

        let sessions = self.sessions.read().unwrap();
        let sessions_json = serde_json::to_string_pretty(&*sessions)?;
        std::fs::write(self.data_path.join("sessions.json"), sessions_json)?;

        Ok(())
    }

    /// Load data from disk
    pub fn load_data(&self) -> Result<()> {
        if !self.data_path.exists() {
            return Ok(());
        }

        // Load users
        let users_path = self.data_path.join("users.json");
        if users_path.exists() {
            let users_json = std::fs::read_to_string(users_path)?;
            let users: HashMap<Uuid, User> = serde_json::from_str(&users_json)?;

            let username_map: HashMap<String, Uuid> = users.values()
                .map(|u| (u.username.clone(), u.user_id))
                .collect();

            *self.users.write().unwrap() = users;
            *self.username_to_id.write().unwrap() = username_map;
        }

        // Load sessions
        let sessions_path = self.data_path.join("sessions.json");
        if sessions_path.exists() {
            let sessions_json = std::fs::read_to_string(sessions_path)?;
            let sessions: HashMap<String, Session> = serde_json::from_str(&sessions_json)?;
            *self.sessions.write().unwrap() = sessions;
        }

        // Update leaderboard
        tokio::spawn({
            let this = self.clone();
            async move {
                this.update_leaderboard().await;
            }
        });

        Ok(())
    }
}
