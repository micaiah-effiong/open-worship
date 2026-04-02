# Openworship

A free and open-source church worship presentation applicatoin built with Rust, and GTK4. Designed to help churches and worship teams display song lyrics, Bible verses, and other content on a projector or secondary screen during service.

## Table of contents

- [Features](#features)
- [Screenshots](#screenshots)
- [Prerequisites](#prerequisites)
- [Installation](#installation)
  - [Building from source](#building-from-source)
- [Usage](#usage)
- [Contributing](#contributing)
- [License](#license)

---

### Features

- **Bible Integration** - Search and diesplay Bible verses from downloadable Bible translations
- **Song Lyrics Presentaon** - Manage and project worship song lyrics slide-by-slide.
- **Content Search** - Quickly find songs or Bible verses by keyword or reference
- **Slide Management** - Organize and control the order of slides during a live service
- **Schedule Management** - Create and organize schedules for service

---

### Screenshots

> Screenshots coming soon. Contributions welcome!

---

### Prerequisites

Before building Openworship, make sure you have the following installed:

- Rust (latest stable) - install via rustup
- GTK4 - install the development libraries for your platform

#### Check your GTK4 version

```bash
pkg-config --modversion gtk4
```

#### Install GTK4

Follow the offical setup guide for your platform: [See the gtk-rs book](https://gtk-rs.org/gtk4-rs/stable/latest/book/installation.html)

---

### Installation

#### Building from source

```bash
# Clone the repository
git clone https://github.com/micaiah-effiong/open-worship.git
cd open-worship

# Build in release mode
cargo build --release

# Run the application
cargo run --release

```

The bundle binary is also avaliable in the github actions (release version are
yet available).
You can download the bundle binary and install base on you platform.

If you are on MacOs you will have to remove the quarantine flag.

```bash
sudo xattr -dr com.apple.quarantine ~/Downloads/openworship-macos-aarch64.zip
```

---

### Usage

Launch the application binary by double clicking on the application icon.

#### Basic Workflow

1. Add content - Import or create song lyrics and load Bible translations
2. Build your service - Arranage the order of songs and scripture readings and save the schedule
3. Go live - Connect your secondary screen and display the content
4. Navigate slide - Step through slides from the activity interface during the service

---

### Contributing

Contributions are welcome! Here is how to get started:

1. Fork the repository
2. Create a new branch: `git switch -C feat/your-feature-name`
3. Make your changes and commit: `git commit -m "Add your feature description"`
4. Push to your fork: `git push origin feat/your-feature-name`
5. Open a Pull Request

Please check the [open issues](https://github.com/micaiah-effiong/open-worship/issues) for known bugs and planned features before starting work.

---

### License

This project is open source. See the repository for the license details.

---
