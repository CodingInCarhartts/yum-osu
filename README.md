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

- 🎵 **Automatic Beat Detection** - Analyzes audio files using aubio to detect kick drums and beats
- 🎨 **Cyberpunk Visual Style** - Neon colors, glowing effects, and futuristic UI
- 🎶 **Dynamic Song Loading** - Load any MP3 file from the assets folder
- 📊 **Real-time Scoring** - Score points based on hit accuracy (300/100/50)
- ⚡ **Smooth Animations** - Circles shrink smoothly with pulsing glow effects
- 🎯 **Precision Gameplay** - Test your timing with millisecond-accurate hit detection

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

| Key | Action |
|-----|--------|
| `A` or `S` | Hit the circle when it shrinks to center |
| `↑` / `↓` | Scroll through song list |
| `Escape` | Exit to main menu (during gameplay) |
| `Enter` / `Click` | Select menu options |

### How to Play

1. Click **"Start Game"** from the main menu
2. Select a song from the list
3. Wait for the countdown
4. Press **A** or **S** when the shrinking circle reaches the center
5. Time your hits perfectly for maximum score!

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

## 📁 Project Structure <a name="project-structure"></a>

```
yum-osu/
├── src/
│   ├── main.rs           # Entry point and game state machine
│   ├── game.rs           # Gameplay logic, circles, scoring
│   ├── ui.rs             # UI rendering (menu, song select, HUD)
│   ├── audio.rs          # Beat detection and audio analysis
│   ├── structs.rs        # Data structures and game state
│   ├── constants.rs      # Game constants and styling
│   └── assets/
│       ├── music/        # MP3 files for gameplay
│       ├── images/       # UI images and textures
│       └── fonts/        # Custom fonts
├── Cargo.toml            # Rust dependencies
└── README.md             # This file
```

### Key Modules

- **`main.rs`** - Game loop and state management (Menu → Song Select → Loading → Gameplay)
- **`game.rs`** - Circle spawning, hit detection, score calculation
- **`ui.rs`** - All UI rendering including menus, buttons, and HUD elements
- **`audio.rs`** - Audio file processing and beat detection using aubio
- **`structs.rs`** - Core data structures (GameState, Circle, Assets, etc.)
- **`constants.rs`** - Styling constants including cyberpunk color palette

---

## 🛠️ Built With <a name="built-with"></a>

| Technology | Purpose |
|------------|---------|
| [Rust](https://www.rust-lang.org/) | Systems programming language |
| [Macroquad](https://macroquad.rs/) | Lightweight game engine for rendering |
| [Rodio](https://github.com/RustAudio/rodio) | Audio playback |
| [aubio](https://aubio.org/) | Audio analysis and beat detection |
| [biquad](https://github.com/korken89/biquad-rs) | Audio filtering (low-pass for kick detection) |
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
