# Blogpost Web Application

## Features

- **Create New Blog Posts**: Users can add text, a publication date (auto-generated), an optional blog image, their username, and an optional avatar image URL.
- **Blog Feed**: Displays all blog posts, showing text, date, username, and any uploaded images.
- **Avatar Download & Persistence**: User avatars are downloaded from the provided URL and saved on the server, ensuring persistence even if the original URL becomes unavailable.
- **Advanced logging**: Tracing formatted as JSON is used to log the backend work.
- **Structurized Error Response**: Errors are returned as JSON of the form

   ```
    {
        status_code: u16,
        message: String
    }
    ```

## Prerequisites

- Docker
- Rust
- psql for local development

## Project Structure
- **`configuration/`**: Stores `base.yaml` with default config for the local app development. Possbile extension with files like `local.yaml` or `production.yaml`
- **`migrations/`**: Stores SQL migrations for setting up the PostgreSQL database.
- **`scripts/`**: Stores scripts for setting up **only database** or both **app and database** with docker-compose.
- **`templates/`**: Stores template for the `/home` view.
- **`Dockerfile`**: Optimized docker image for the app.
- **`docker-compose.yml`**: Set ups applicaction **with the database**.

## Source Files details
- **`src/startup.rs`** - Initializes Application
- **`src/telemetry.rs`** - Setup telemetry for the app (logging)
- **`src/domain.rs`** - Definitions of `BlogPost` table in the database and queries functions
- **`src/configuration.rs`** - Definition of setting up the configurations for the app.
---
- **`src/routes/posts.rs`** - Consists endpoint `POST /posts` for adding posts.
- **`src/routes/home.rs`** - Home view with `form` for uploading the post.
- **`src/routes/errors.rs`** - Custom error definitions.

## How tu run

### 1. Run with Docker Compose

If Docker compose is preferred, this application can be run directly through Docker Compose:

```bash
./scripts/init_docker_compose.sh
```

The script will **automatically** build the app image and initialize the tables in PostgreSQL container.

#### Accessing the Application

The app will be accessible at `http://localhost:8000/home`. Here, you can add new posts and view the blog feed.

### 2. Running Locally for development (without Docker)

Alternatively, you can run the app locally **(requires PostgreSQL image running).**

```bash
# Initialize database container
./scripts/init_db.sh

# Run the app with watch
cargo watch -x run
```

## Application Endpoints

- **`GET /health_check`**: Healtch check endpoint.
- **`GET /home`**: Main page where users can add and view blog posts.
- **`POST /posts`**: Endpoint for creating a new blog post.


## File Storage

- Uploaded images and avatars are stored **by default** locally in the `uploads/` directory. This can be overwritten by passing a new value with env **`APP_DATABASE__UPLOAD_PATH`**
- The application will download and save the user-provided avatar images to ensure their persistence.

## Telemetry
- Proper telemetry is used in order to log every request and backend actions into `stdout` in JSON format. More details can be found at `src/telemetry.rs`

## Project Dependencies

- **Rust**: The backend is written in Rust.
- **Axum**: Web framework for the Rust backend.
- **Askama**: Jinja-like tool for generating HTML from templates. 
- **SQLx**: Used for handling database operations.
- **PostgreSQL**: For the database
- **Docker**: For containerizing the application and database.

## Adjusting configuration with environment variables
You can adjust the config dynamically. E.g. assuming `configuration` variable holds our settings you can override `configuration.database.host` with `APP_DATABASE__HOST`.

## Tests

**WIP**
