# Blogpost Web Application

## Features

- **Create New Blog Posts**: Users can add text, a publication date (auto-generated), an optional blog image, their username, and an optional avatar image URL.
- **Blog Feed**: Displays all blog posts, showing text, date, username, and any uploaded images.
- **Avatar Download & Persistence**: User avatars are downloaded from the provided URL and saved on the server, ensuring persistence even if the original URL becomes unavailable.
- **Advanced Logging**: Tracing formatted as JSON is used to log backend activity.
- **Structured Error Response**: Errors are returned as JSON in the following form:

   ```
    {
        "status_code": u16,
        "message": "String"
    }
   ```

## Prerequisites

- Docker
- Rust
- `psql` for local development

## Project Structure
- **`configuration/`**: Contains `base.yaml` with default config for local app development. Possible extension with files like `local.yaml` or `production.yaml`.
- **`migrations/`**: Stores SQL migrations for setting up the PostgreSQL database.
- **`scripts/`**: Stores scripts for setting up **only the database** or **app and database** with Docker Compose.
- **`templates/`**: Stores the template for the `/home` view.
- **`tests/`**: API endpoint tests.
- **`Dockerfile`**: Optimized Docker image for the app.
- **`docker-compose.yml`**: Sets up the application **with the database**.

## Source Files Details
- **`src/startup.rs`** - Initializes the application.
- **`src/telemetry.rs`** - Sets up telemetry for the app (logging).
- **`src/domain.rs`** - Defines the `BlogPost` table in the database and query functions.
- **`src/configuration.rs`** - Handles configuration settings for the app.
---
- **`src/routes/posts.rs`** - Contains the endpoint `POST /posts` for adding posts.
- **`src/routes/home.rs`** - Home view with a `form` for uploading a post.
- **`src/routes/errors.rs`** - Custom error definitions.

## How to Run

### 1. Run with Docker Compose

If Docker Compose is preferred, this application can be run directly through Docker Compose:

```bash
./scripts/init_docker_compose.sh
```

The script will **automatically** build the app image and initialize the tables in the PostgreSQL container.

#### Accessing the Application

The app will be accessible at `http://localhost:8000/home`. Here, you can add new posts and view the blog feed.

### 2. Running Locally for Development (without Docker)

Alternatively, you can run the app locally **(requires a running PostgreSQL instance)**:

```bash
# Initialize the database container
./scripts/init_db.sh

# Run the app with watch
cargo watch -x run
```

## Application Endpoints

- **`GET /health_check`**: Health check endpoint.
- **`GET /home`**: Main page where users can add and view blog posts.
- **`POST /posts`**: Endpoint for creating a new blog post.

## File Storage

- Uploaded images and avatars are stored **by default** locally in the `uploads/` directory. This can be overridden by passing a new path via the environment variable **`APP_DATABASE__UPLOAD_PATH`**.
- The application will download and save user-provided avatar images to ensure their persistence.

## Telemetry
- Comprehensive telemetry is used to log every request and backend action into `stdout` in JSON format. More details can be found in `src/telemetry.rs`.

## Project Dependencies

- **Rust**: Backend is written in Rust.
- **Axum**: Web framework for the Rust backend.
- **Askama**: Jinja-like tool for generating HTML from templates. 
- **SQLx**: Used for handling database operations.
- **PostgreSQL**: For the database.
- **Docker**: For containerizing the application and database.

## Adjusting Configuration with Environment Variables
You can adjust the configuration dynamically. For example, assuming `configuration` holds your settings, you can override `configuration.database.host` with `APP_DATABASE__HOST`.

## Tests

To run tests you must have running database in the container. All you need to do is first run

```bash
./scripts/init_db.sh
```

and then 

```bash
cargo test
```