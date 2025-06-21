# Database CRUD Example

This example demonstrates real database operations with Layer9, including:

- SQLite database backend (lightweight and easy to set up)
- Full CRUD operations (Create, Read, Update, Delete)
- Repository pattern for database access
- Query builder for complex queries
- Real-time UI updates
- Error handling

## Features

- **User Management**: Create, list, and delete users
- **Post Management**: Create, list, and delete posts for each user
- **Relationships**: Posts are linked to users via foreign keys
- **Real Database**: Uses SQLite with actual persistence

## Running the Example

1. Build the WASM module:
```bash
./build.sh
```

2. Run the server:
```bash
cargo run --bin server --features layer9-core/ssr
```

3. Open your browser to `http://localhost:3000`

## Database Configuration

By default, this example uses SQLite with a local database file. You can configure it using environment variables:

- `DATABASE_TYPE`: Set to `sqlite` (default), `postgres`, or `memory`
- `DATABASE_URL`: Database connection string
  - SQLite file: `sqlite:crud_example.db` (default)
  - SQLite memory: `sqlite::memory:`
  - PostgreSQL: `postgres://user:password@localhost/dbname`

## API Endpoints

The server exposes database API endpoints at `/api/db/*`:

- `POST /api/db/execute` - Execute a query
- `POST /api/db/query_one` - Query single row
- `POST /api/db/query_many` - Query multiple rows

## Code Structure

- `src/lib.rs` - Main application with UI components
- `src/server.rs` - Axum server with database API
- Models:
  - `User` - User model with username and email
  - `Post` - Post model with title, content, and published status

## Security Note

This example doesn't include authentication for simplicity. In production:
- Add proper authentication middleware
- Use secure database credentials
- Implement proper authorization checks
- Validate and sanitize all inputs