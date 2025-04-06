# Chat Application with CQRS and Event Sourcing

This is a chat application implemented using Command Query Responsibility Segregation (CQRS) and Event Sourcing architectural patterns in Rust. It provides both a Terminal User Interface (TUI) and a Web API.

## Features

- Group chat functionality
- Event Sourcing for complete audit trail
- CQRS architecture for separation of read and write operations
- Terminal User Interface (TUI) for interactive chat
- Web API for programmatic access
- PostgreSQL for event storage
- Docker setup for easy local deployment

## Architecture

The application follows the CQRS and Event Sourcing patterns:

- **Domain Layer**: Contains the core business logic and entity definitions
  - **Aggregate**: Defines the `ChatRoom` entity and its behavior
  - **Commands**: Defines operations that can be performed on chat rooms
  - **Events**: Defines state change events and error types

- **Service Layer**: Handles external integrations and view models
  - **ChatServices**: Provides notification operations
  - **ChatRoomViewRepository**: Implements the query side of CQRS

- **UI Layer**: Provides user interfaces
  - **TUI**: Terminal User Interface for interactive chat
  - **Web API**: RESTful API for programmatic access

## Running the Application

### Using Docker

The easiest way to run the application is using Docker Compose:

```bash
docker-compose up
```

This will start:
- PostgreSQL database for event storage
- The chat application with both TUI and Web API

### Manual Setup

If you prefer to run the application manually:

1. Install Rust and Cargo
2. Set up PostgreSQL and create a database
3. Build and run the application:

```bash
cargo build --release
RUST_LOG=info DATABASE_URL=postgres://postgres:postgres@localhost:5432/chat_app ./target/release/chat-app
```

## Using the Application

### TUI

The Terminal User Interface provides an interactive chat experience:

1. Enter your username
2. Create a new chat room or join an existing one
3. Send and receive messages in real-time
4. View participants in the room

### Web API

The Web API is available at `http://localhost:8080/api` with the following endpoints:

- `GET /api/rooms` - List all chat rooms
- `POST /api/rooms` - Create a new chat room
- `GET /api/rooms/{room_id}` - Get details of a specific room
- `POST /api/rooms/{room_id}/join` - Join a chat room
- `POST /api/rooms/{room_id}/leave` - Leave a chat room
- `POST /api/rooms/{room_id}/messages` - Send a message to a chat room

Example of creating a room:
```bash
curl -X POST http://localhost:8080/api/rooms \
  -H "Content-Type: application/json" \
  -d '{"name":"Test Room","created_by":"user1"}'
```

## Development

To set up the development environment:

1. Clone the repository
2. Install dependencies:
```bash
cargo build
```
3. Run tests:
```bash
cargo test
```
