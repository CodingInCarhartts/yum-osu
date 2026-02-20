//! Multiplayer server binary for Yum-OSU!
//! Standalone server for handling multiplayer sessions, accounts, and community features

use anyhow::Result;
use std::sync::Arc;
use tokio::signal;

mod network;
mod accounts;
mod multiplayer;
mod community;

use network::GameServer;
use accounts::AccountManager;
use multiplayer::GameCoordinator;
use community::CommunityManager;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🎵 Yum-OSU! Multiplayer Server 🎵");
    println!("==================================");

    // Initialize managers
    println!("Initializing managers...");
    let account_manager = Arc::new(AccountManager::new(std::path::PathBuf::from("data")));
    let game_coordinator = Arc::new(GameCoordinator::new());
    let community_manager = Arc::new(CommunityManager::new());

    // Load existing data
    println!("Loading data...");
    if let Err(e) = account_manager.load_data() {
        println!("Warning: Could not load data: {}", e);
        println!("Starting with fresh state...");
    }

    // Create game server
    println!("Starting game server...");
    let game_server = GameServer::new();

    // Server address
    let addr = "0.0.0.0:8080";
    println!("Listening on {}", addr);

    // Spawn server task
    let server_handle = tokio::spawn(async move {
        if let Err(e) = game_server.start(addr).await {
            eprintln!("Server error: {}", e);
        }
    });

    // Handle shutdown
    tokio::select! {
        _ = signal::ctrl_c() => {
            println!("\nShutting down gracefully...");
        }
        _ = server_handle => {
            println!("Server task completed");
        }
    }

    // Save data before shutdown
    println!("Saving data...");
    let _ = account_manager.load_data();

    println!("Goodbye! 👋");
    Ok(())
}
