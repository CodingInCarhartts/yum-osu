<h1 align="center">🎵 Yum-OSU!</h1>

<p align="center">
  <strong>A cyberpunk-styled rhythm game inspired by OSU!</strong>
</p>

<div align="center">

[![Status](https://img.shields.io/badge/status-active-success.svg)](https://github.com/Yumshot/yum-osu)
[![GitHub Issues](https://img.shields.io/github/issues/Yumshot/yum-osu.svg)](https://github.com/Yumshot/yum-osu/issues)
[![GitHub Pull Requests](https://img.shields.io/github/issues-pr/Yumshot/yum-osu.svg)](https://github.com/Yumshot/yum-osu/pulls)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](/LICENSE)
[![Rust](https://img.shields.io/badge/rust-%23000000.svg?logo=rust&logoColor=white)](https://www.rust-lang.org/)

</div>

<p align="center">
  <img src="https://img.shields.io/badge/macroquad-0.4-ff69b4" alt="Macroquad">
  <img src="https://img.shields.io/badge/rust-2021-orange" alt="Rust Edition">
</p>

---

## 📖 Table of Contents

- [About](#about)
- [Features](#features)
- [Screenshots](#screenshots)
- [Getting Started](#getting-started)
- [Controls](#controls)
- [Gameplay](#gameplay)
- [Project Structure](#project-structure)
- [Built With](#built-with)
- [Authors](#authors)
- [Acknowledgments](#acknowledgments)

---

## 🎮 About <a name="about"></a>

**Yum-OSU!** is a rhythm game built in Rust with a sleek cyberpunk aesthetic. Players hit circles that shrink to the beat of the music, testing their timing and precision. The game features automatic beat detection from audio files, creating dynamic gameplay from any song.

The game uses **beat detection algorithms** to analyze audio files in real-time and generate hit circles that synchronize with the music's rhythm.

---

## ✨ Features <a name="features"></a>

### Core Gameplay
- 🎵 **Automatic Beat Detection** - Analyzes audio files using aubio to detect kick drums and beats
- 🎨 **Cyberpunk Visual Style** - Neon colors, glowing effects, and futuristic UI
- 🎶 **Dynamic Song Loading** - Load any MP3 file from the assets folder
- 📊 **Real-time Scoring** - Score points based on hit accuracy (300/100/50)
- ⚡ **Smooth Animations** - Circles shrink smoothly with pulsing glow effects
- 🎯 **Precision Gameplay** - Test your timing with millisecond-accurate hit detection
- 🔥 **Combo System** - Build combos for higher scores with visual feedback

### Customization (Issue #12)
- ⌨️ **Custom Key Bindings** - Configure your own keys for hitting circles, navigation, and actions
- 🎨 **Visual Themes** - Customize colors, circle sizes, particles, and screen effects
- 🔊 **Audio Settings** - Adjust master, music, and effects volumes independently
- 💾 **Persistent Config** - Settings saved to `config.json`

### Practice Tools (Issue #12)
- ⏱️ **Playback Speed Control** - Practice at 0.25x to 2.0x speed
- 🛡️ **No-Fail Mode** - Practice without penalties for missing
- 🤖 **Autoplay Mode** - Watch the game play itself
- 🔊 **Hit Sounds** - Audio feedback on every hit

### Multiplayer, Accounts, and Community (Issue #16)
- 🌐 **Real-Time Multiplayer** - Compete with players worldwide in real-time rhythm battles
- 👤 **User Accounts** - Create accounts, manage profiles, and track progress across devices
- 🔐 **Secure Authentication** - Password hashing with Argon2, session management, and token-based auth
- 🏆 **Global Leaderboards** - Climb the ranks on global, country, and friends leaderboards
- 👥 **Friends System** - Add friends, see their online status, and challenge them to matches
- 🎖️ **Achievements** - Unlock achievements for milestones like perfect games, high combos, and more
- 💬 **Live Chat** - Chat in lobbies and send direct messages to friends
- 🏅 **Tournaments** - Participate in community tournaments with prizes and rankings
- 🏠 **Lobby System** - Create and join game rooms, set player limits, and host matches
- 📊 **Live Score Sync** - Real-time score updates and ranking during multiplayer matches
- 🎯 **Game State Synchronization** - Accurate hit detection and combo tracking for all players
- 📈 **Performance Tracking** - Detailed stats on every game session
- 🏆 **Grade System** - SS, S, A, B, C, D, F grades based on accuracy
- 📊 **Hit Statistics** - Track Perfect, Good, Okay, and Miss counts
- 📉 **Accuracy Trends** - Visual graphs of your improvement over time
- 🎖️ **Achievements** - Unlock achievements for milestones
- 🎵 **Per-Song Stats** - Track best scores and accuracy for each song
- 💾 **Persistent Data** - Analytics saved to `analytics.json`

---

## 📸 Screenshots <a name="screenshots"></a>

> _Screenshots coming soon!_

The game features:
- A neon-styled main menu with glowing buttons
- Song selection screen with scrolling playlist
- Loading screen with animated progress bar
- Gameplay with shrinking circles and hit feedback

---

## 🚀 Getting Started <a name="getting-started"></a>

### Prerequisites

- **Rust** (latest stable version) - [Install Rust](https://www.rust-lang.org/tools/install)
- **System Dependencies** (for audio processing):
  - On Ubuntu/Debian: `sudo apt-get install libasound2-dev`
  - On macOS: `brew install pkg-config`
  - On Windows: No additional dependencies required

### Installation

1. **Clone the repository:**
   ```bash
   git clone https://github.com/Yumshot/yum-osu
   cd yum-osu
   ```

2. **Build the project:**
   ```bash
   cargo build --release
   ```

3. **Run the game:**
   ```bash
   cargo run --release
   ```

> **Note:** Use `--release` flag for optimal performance. The game uses audio processing that benefits from release optimizations.

---

## 🎮 Controls <a name="controls"></a>

### Default Controls
| Key | Action |
|-----|--------|
| `A` | Primary hit key |
| `S` | Secondary hit key |
| `↑` / `↓` | Scroll through song list |
| `Enter` | Select menu options |
| `Escape` | Exit to main menu / Pause |

### Customizable Controls
All controls can be customized in the **Settings** menu:
- Primary Hit Key
- Secondary Hit Key
- Navigate Up/Down
- Select/Confirm
- Pause
- Exit

### How to Play

1. Click **"Start Game"** from the main menu
2. Select a song from the list
3. Wait for the countdown
4. Press your configured hit keys when the shrinking circle reaches the center
5. Time your hits perfectly for maximum score!

### Practice Mode

1. Click **"Practice"** from the main menu
2. Select your practice settings:
   - Playback speed (0.25x - 2.0x)
   - No-fail mode
   - Autoplay mode
   - Hit sounds
3. Select a song and start practicing!

---

## 🎯 Gameplay <a name="gameplay"></a>

### Scoring System

| Timing | Points | Feedback |
|--------|--------|----------|
| < 0.1s | 300 | Perfect! |
| < 0.3s | 100 | Good |
| < 0.5s | 50 | Okay |
| Miss | 0 | Miss |

### Adding Your Own Songs

1. Place your `.mp3` files in `src/assets/music/`
2. Restart the game
3. Your songs will appear in the song selection menu

---

## 🌐 Multiplayer Setup <a name="multiplayer"></a>

### Running the Multiplayer Server

The game includes a dedicated multiplayer server for hosting games, managing accounts, and community features.

**Start the server:**
```bash
cargo run --bin server
```

The server will listen on `0.0.0.0:8080` by default and handle:
- WebSocket connections for real-time gameplay
- User authentication and session management
- Lobby and room management
- Leaderboard updates
- Tournament coordination

### Account Management

**Creating an Account:**
1. Select "Profile" from the main menu
2. Click "Register" if you don't have an account
3. Enter your username, email, and password
4. Your account is created and you can start competing!

**Login:**
1. Select "Profile" or attempt to access multiplayer features
2. Enter your username and password
3. Your session is valid for 30 days

### Multiplayer Gameplay

**Joining a Lobby:**
1. Select "Multiplayer" from the main menu
2. Browse available rooms or create your own
3. Join a room and wait for the host to start the game

**Creating a Room:**
1. In Multiplayer lobby, press "C" to create a room
2. Set room name and maximum players (2-8)
3. Select a song from your library
4. Wait for other players to join and ready up
5. Start the match!

**During Multiplayer Matches:**
- Scores are synced in real-time
- See rankings update live as players hit notes
- Chat with other players
- Spectate matches if you join late

### Community Features

**Leaderboards:**
- Access via "Leaderboard" from main menu
- View Global, Country, and Friends leaderboards
- See your rank, score, and accuracy
- Track progress over time

**Friends System:**
- Add players by username
- See online status and current activity
- Send direct messages
- Challenge friends to private matches

**Tournaments:**
- Browse active tournaments in "Community Hub"
- Join open tournaments
- Compete in brackets
- Win prizes and climb tournament rankings

**Achievements:**
- Unlock achievements for milestones
- View progress in your profile
- Show off achievements to friends

### Multiplayer Networking

The game uses WebSockets for real-time communication:
- Low-latency hit event synchronization
- Reliable message delivery
- Efficient JSON-based protocol
- Automatic reconnection support

---

## 📁 Project Structure <a name="project-structure"></a>

```
yum-osu/
├── src/
│   ├── main.rs           # Entry point and game state machine
│   ├── game.rs           # Gameplay logic, circles, scoring
│   ├── ui.rs             # UI rendering (menu, song select, HUD, settings, analytics)
│   ├── audio.rs          # Beat detection and audio analysis
│   ├── structs.rs        # Data structures and game state
│   ├── constants.rs      # Game constants and styling
│   ├── config.rs         # Settings and customization system
│   ├── analytics.rs      # Performance tracking and statistics
│   ├── network.rs        # WebSocket client/server for multiplayer
│   ├── accounts.rs       # User authentication, profiles, sessions
│   ├── multiplayer.rs    # Game state synchronization, rooms, events
│   ├── community.rs      # Leaderboards, friends, chat, tournaments
│   ├── beatmap.rs        # Beat detection and circle spawning
│   └── assets/
│       ├── music/        # MP3 files for gameplay
│       ├── images/       # UI images and textures
│       └── fonts/        # Custom fonts
├── src/bin/
│   └── server.rs        # Dedicated multiplayer server
├── Cargo.toml            # Rust dependencies
├── config.json           # User settings (auto-generated)
├── analytics.json        # Player statistics (auto-generated)
├── data/
│   ├── users.json       # User accounts database
│   └── sessions.json    # Active sessions
└── README.md             # This file
```

### Key Modules

- **`main.rs`** - Game loop and state management (Menu → Song Select → Loading → Gameplay → Results)
- **`game.rs`** - Circle spawning, hit detection, score calculation, combo system
- **`ui.rs`** - All UI rendering including menus, buttons, HUD elements, settings screens, and analytics views
- **`audio.rs`** - Audio file processing and beat detection using aubio
- **`structs.rs`** - Core data structures (GameState, Circle, Assets, VisualizingState, EndState, etc.)
- **`constants.rs`** - Styling constants including cyberpunk color palette
- **`config.rs`** - Settings system with key bindings, themes, audio, and practice mode configuration
- **`analytics.rs`** - Performance tracking with grades, hit statistics, achievements, and session history
- **`network.rs`** - WebSocket client/server implementation for real-time multiplayer communication
- **`accounts.rs`** - User authentication with Argon2 password hashing, session management, and profiles
- **`multiplayer.rs`** - Game state synchronization, room management, and event coordination
- **`community.rs`** - Leaderboards, friends system, chat rooms, and tournaments

---

## 🛠️ Built With <a name="built-with"></a>

| Technology | Purpose |
|------------|---------|
| [Rust](https://www.rust-lang.org/) | Systems programming language |
| [Macroquad](https://macroquad.rs/) | Lightweight game engine for rendering |
| [Rodio](https://github.com/RustAudio/rodio) | Audio playback |
| [aubio](https://aubio.org/) | Audio analysis and beat detection |
| [biquad](https://github.com/korken89/biquad-rs) | Audio filtering (low-pass for kick detection) |
| [Tokio](https://tokio.rs/) | Async runtime for networking |
| [Tokio-Tungstenite](https://github.com/snapview/tokio-tungstenite) | WebSocket implementation |
| [Argon2](https://github.com/RustCrypto/password-hashes) | Secure password hashing |
| [UUID](https://github.com/uuid-rs/uuid) | Unique identifiers for users and rooms |
| [Chrono](https://github.com/chronotope/chrono) | Date and time handling |
| [rayon](https://github.com/rayon-rs/rayon) | Data parallelism |
| [rand](https://github.com/rust-random/rand) | Random number generation |

---

## ✍️ Authors <a name="authors"></a>

- **[@Yumshot](https://github.com/Yumshot)** - Creator & Developer

---

## 🙏 Acknowledgments <a name="acknowledgments"></a>

- Inspired by [OSU!](https://osu.ppy.sh/) - the popular rhythm game
- Cyberpunk color palette inspired by synthwave aesthetics
- Beat detection powered by [aubio](https://aubio.org/) library

---

## 📄 License

This project is licensed under the MIT License.

---

<p align="center">
  <strong>Get your groove on! 🎵</strong>
</p>
