# rs-polar-mqtt

Code to support this blog post.

## Prerequisites

Before building this project, ensure you have the following dependencies installed:

### Common Requirements
- Rust toolchain (stable)
- CMake (3.1 or higher)
- C++ compiler with C++17 (or above) support
- Eclipse Paho MQTT C++ Client Library

### macOS (via Homebrew)
```bash
brew install cmake
brew install paho-mqtt
```

### Linux (Debian/Ubuntu)
```bash
sudo apt update
sudo apt install build-essential cmake libssl-dev
sudo apt install libpaho-mqtt-dev
```

For other Linux distributions, please use the appropriate package manager.

## Building

This project uses Cargo with a custom build script (`build.rs`) to handle the C++ dependencies.

```bash
# Clone the repository
git clone https://github.com/jsulmont/rs-polar-mqtt
cd rs-polar-mqtt

# Build the project
cargo build
```

## Testing

```bash
cargo test
```



## Platform Support

| Platform | Status |
|----------|--------|
| macOS    | ✅     |
| Linux    | ✅     |
| Windows  | ❓ Untested |


