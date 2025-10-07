# api_reference.md

> **Authentication Requirement:**
> All `/api/*` endpoints **except** `/api/health`, `/api/login`, and `/api/register` require authentication via a valid **JWT** session token passed in the `Authorization` header as a Bearer token.
>
> If the session is missing or invalid, the API will return a 401 Unauthorized error.
>
---

## Table of contents
- Authentication
- Songs
- Artists
- Playlists
- User Management
- Admin (RBAC)
- Streaming
- Errors & Conventions
- Notes

---

## Authentication

### Health Check
**Endpoint:** `GET /api/health`

**Authentication:** Not required

**Description:** Check server health status and available protocols.

**Response:**
```json
{
  "status": "OK",
  "protocols": { "http": true, "https": true },
  "server": "Muse Music Server",
  "version": "1.0.0"
}
```

---

### Register
**Endpoint:** `POST /api/register`

**Authentication:** Not required

**Headers:**
- `Content-Type: application/json`

**Request Body:**
```json
{
  "username": "string",
  "email": "string",
  "password": "string",
  "confirm_password": "string"
}
```

**Validation Rules:**
- Username: 3–20 characters, only letters, numbers, and underscores
- Email: Valid email format
- Password: Must contain at least one number and one special character (!@#$%^&*)
- Passwords must match

**Success Response:**
```json
{
  "success": true,
  "message": "Registration successful",
  "token": "<jwt-token>"
}
```

**Error Response:**
```json
{
  "success": false,
  "message": "Please correct the errors below",
  "errors": { "field_name": "error message" }
}
```

---

### Login
**Endpoint:** `POST /api/login`

**Authentication:** Not required

**Headers:**
- `Content-Type: application/json`

**Request Body:**
```json
{ "username": "string", "password": "string" }
```

**Success Response:**
```json
{ "success": true, "message": "Login successful", "token": "<jwt-token>" }
```

**Error Response:**
```json
{ "success": false, "message": "Invalid credentials" }
```

---

### Admin Login
**Endpoint:** `POST /api/admin/login`

**Authentication:** Not required

**Headers:**
- `Content-Type: application/json`

**Request Body:**
```json
{
  "username": "string",
  "password": "string"
}
```

**Description:**
Authenticate an administrator and issue a **JWT** with elevated privileges (`role: "admin"`).
Tokens from this endpoint are required for all `/api/admin/*` routes.

**Success Response:**
```json
{
  "success": true,
  "message": "Admin login successful",
  "token": "<admin-jwt-token>",
  "role": "admin"
}
```

**Error Response:**
```json
{
  "success": false,
  "message": "Invalid admin credentials"
}
```

---

### Logout
**Endpoint:** `POST /api/logout`

**Authentication:** Required (JWT)

**Description:** Invalidate the provided JWT (server-side blacklist or token revocation).

**Response:**
```json
{ "success": true, "message": "Logged out successfully" }
```

---

### Refresh Token
**Endpoint:** `POST /api/refresh`

**Authentication:** Required (JWT)

**Note** admin tokens can not be refreshed

**Description:** Exchange a near-expiry token for a fresh JWT. Implementation may return a new token with renewed expiry.

**Response:**
```json
{ "success": true, "message": "Token refreshed", "token": "<new-jwt-token>" }
```

---

## Songs

### Get Songs
**Endpoint:** `GET /api/songs?index_start=X&index_end=Y`

**Authentication:** Required (JWT)

**Query Parameters:**
- `index_start` (required)
- `index_end` (required)

**Response:**
```json
{
  "success": true,
  "message": "songs",
  "data": [ { "name": "Song Name", "artist_name": "Artist Name" } ],
  "timestamp": "2025-10-07T00:00:00Z"
}
```

---

### Get Song Info
**Endpoint:** `GET /api/songs/info?artist_name=X&name=Y`

**Authentication:** Required

**Response:**
```json
{
  "success": true,
  "message": "Song info",
  "data": {
    "name": "Song Name",
    "artist_name": "Artist Name",
    "album": "Album Name",
    "duration": 210,
    "bitrate": 320,
    "genre": "Pop"
  },
  "timestamp": "2025-10-07T00:00:00Z"
}
```

---

### Get Total Songs
**Endpoint:** `GET /api/songs/total`

**Authentication:** Required

**Response:**
```json
{ "success": true, "message": "Got Total", "data": { "total": 100 }, "timestamp": "2025-10-07T00:00:00Z" }
```

---

### Get Song Cover Image
**Endpoint:** `GET /api/songs/cover?artist_name=X&name=Y`

**Authentication:** Required

**Response:**
- Content-Type: `image/avif`
- Returns binary image data, or 404 if not found

---

### Search Songs
**Endpoint:** `GET /api/songs/search?query=X`

**Authentication:** Required

**Response:**
```json
{ "success": true, "message": "fuzzy search results", "data": [ { "name": "Song Name", "artist_name": "Artist Name" } ], "timestamp": "2025-10-07T00:00:00Z" }
```

---

## Artists

(Endpoints mirror your original spec; include timestamps in responses)

- `GET /api/artists?index_start=X&index_end=Y`
- `GET /api/artists/total`
- `GET /api/artists/cover?name=X`
- `GET /api/artists/songs?name=X`

Responses follow the same JSON envelope and timestamp pattern.

---

## Playlists

### Visibility rules
- **Private**: visible only to owner
- **Public**: visible to all authenticated users
- **Shared**: visible to specified users only

### Get Private Playlists
`GET /api/playlists/private?index_start=X&index_end=Y`

### Get Public Playlists
`GET /api/playlists/public?index_start=X&index_end=Y`

### Create Playlist
`POST /api/playlists`

Request:
```json
{ "name": "Playlist Name", "isPublic": false }
```

Response:
```json
{ "success": true, "message": "Playlist created successfully" }
```

### Add Song to Playlist
`POST /api/playlists/song/add`

Request:
```json
{ "playlist": "Playlist Name", "song": "Song Name", "artist": "Artist Name" }
```

### Remove Song from Playlist
`POST /api/playlists/song/remove`

Request:
```json
{ "playlist": "Playlist Name", "song": "Song Name" }
```

### Delete Playlist
`DELETE /api/playlists?name=X`

### Share Playlist
`POST /api/playlists/share`

Request:
```json
{ "playlist_name": "Chill Vibes", "target_user": "friend_username" }
```

### Get Shared Playlists
`GET /api/playlists/shared`

### Revoke Playlist Share
`DELETE /api/playlists/share`

Request:
```json
{ "playlist_name": "Chill Vibes", "target_user": "friend_username" }
```

---

## User Management

### Get User Info
`GET /api/user`

### Update User Info
`PUT /api/user`

Request:
```json
{ "username": "new_username", "email": "new_email@example.com" }
```

### Change Password
`PUT /api/user/password`

Request:
```json
{ "old_password": "current_password", "new_password": "new_secure_password" }
```

### Reset Password
`POST /api/user/reset`

Request:
```json
{ "email": "user@example.com" }
```

### Delete Account
`POST /api/user/delete`

Request:
```json
{ "password": "current_password" }
```

---

## Streaming

### Stream Song
`GET /api/stream?artist=X&name=Y&format=Z`

Query parameters:
- `artist` (required)
- `name` (required)
- `format` (optional) default `mp3`

Response:
- Content-Type: `audio/mpeg` or `audio/mp4`
- `Accept-Ranges: bytes`
- Partial content support (206) for streaming

---

## Admin (RBAC)

> Admin-only endpoints are grouped under `/api/admin/*`. Access requires a JWT token for an admin `role`.

### Get All Users
`GET /api/admin/users?index_start=X&index_end=Y`

### Edit User
`PUT /api/admin/users/edit`

Request:
```json
{ "username": "john_doe", "new_email": "john@newmail.com", "role": "admin" }
```

### Delete User
`DELETE /api/admin/users/delete`

Request:
```json
{ "username": "john_doe" }
```

### Add Song (admin upload)
`POST /api/admin/songs/add` (multipart/form-data)

Form fields:
- `name` (string)
- `artist` (string)
- `album` (string)
- `genre` (string, optional)
- `file` (binary audio)
- `cover` (binary image, optional)

### Edit Song Metadata
`PUT /api/admin/songs/edit`

### Delete Song
`DELETE /api/admin/songs/delete`

### Admin Playlist Management
- `GET /api/admin/playlists`
- `PUT /api/admin/playlists/edit`
- `DELETE /api/admin/playlists/delete`

---

## Unified Error Responses

All endpoints follow this envelope for errors:
```json
{
  "success": false,
  "message": "Error message describing the issue",
  "code": 400,
  "timestamp": "2025-10-07T00:00:00Z",
  "errors": { "field": "optional detailed explanation" }
}
```

---

## Notes
- All responses use UTF-8 JSON unless stated otherwise.
- All date/time strings are ISO 8601 (`YYYY-MM-DDTHH:mm:ssZ`).
- The API is stateless; a valid JWT must be sent on each request except the public endpoints.
- File responses support `Range` requests.
- Pagination parameters are 0-indexed and inclusive.
- Rate limiting: 60 requests/minute per IP (example; implement as needed).
- CORS: Requests allowed only from approved origins.

