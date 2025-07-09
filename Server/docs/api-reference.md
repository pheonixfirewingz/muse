# API Reference

> **All `/api/*` except in `authentication` endpoints require authentication via a valid session**
> If the session is missing or invalid, the API will return a 401 Unauthorized error. This includes all playlist, image, and streaming endpoints.

## Authentication
- `GET /api/login` — Get A Session From The Server If the User Is Registered
- `GET /api/register` — Registration Account

## Data Retrieval

- `GET /api/songs?index_start=X&index_end=Y` - Gets the song data based on the index
