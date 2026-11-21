# Docker Environment Setup

## Prerequisites

- Docker
- Docker Compose V2

## 1\. Configure Environment

Update `.env` with the required application and database credentials.

## 2\. Build and Start


```bash
# Build Docker images
docker compose build

# Start containers
docker compose up -d

# Check running containers
docker compose ps
```

## 3\. Health Checks


```bash
# Application / Database health
curl -i http://localhost:3010/health

# pgAdmin availability
curl -I http://localhost:8085
```

## 4\. Access pgAdmin

Open in your browser:

**http://localhost:8085**

Development credentials:

- Email: `admin@example.com`
- Password: `admin-dev-secret`

To connect pgAdmin to PostgreSQL, use:

- Host: `postgres`
- Port: `5432`
- Username/Password: From your `.env` configuration

## 5\. Useful Commands

```bash
# View logs
docker compose logs -f

# Restart services
docker compose restart

# Stop containers
docker compose down
```

**Note:** `docker compose down -v` also deletes persistent volumes and database data.