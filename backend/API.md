# MuSync backend API plan

This document defines the first backend API for synchronized YouTube listening rooms.

The backend owns the room queue and playback state. Clients send commands over a WebSocket. The backend validates each command, applies it in order, increments the room version, and broadcasts the new full state to every connected client.

The current backend exposes `/health`. Better Auth and the music API live under `/api/v1`, with authentication under `/api/v1/auth` and rooms under `/api/v1/rooms`.

## Goals

- Keep one authoritative queue and playback state for each room.
- Let a new or reconnected client recover with one complete snapshot.
- Keep playback clocks aligned without sending a message for every video time update.
- Persist rooms and queues so a backend restart does not lose them.
- Reject stale commands instead of silently overwriting a newer room state.
- Use the existing Better Auth session for HTTP requests and WebSocket upgrades.

## Decisions for the first version

- A room requires an authenticated user.
- The room owner controls playback and queue ordering.
- Members can add videos and remove videos that they added.
- A queue item is separate from the YouTube video ID. The same video can appear more than once.
- The backend stores only the YouTube video ID. Clients load the video through the YouTube player.
- The backend does not proxy YouTube media or call the YouTube Data API.
- A room keeps its queue until the owner deletes the room.
- The first deployment runs as one backend instance. The protocol must remain valid when a shared room coordinator or pub/sub layer is added later.

Playback control for every member can be added later as a room policy. It should not change the message format.

## Domain model

### Room

A room is the durable aggregate for shared listening state.

```text
Room
  id: UUID
  ownerId: UserId
  joinCode: string
  stateVersion: integer
  currentItemId: UUID | null
  playbackStatus: paused | playing
  playbackPositionMs: non-negative integer
  playbackChangedAt: timestamp
  createdAt: timestamp
  updatedAt: timestamp
```

`stateVersion` starts at `0` and increases by one for every accepted state-changing command. Presence changes do not change this version.

### Room member

Membership is durable and presence is temporary.

```text
RoomMember
  roomId: UUID
  userId: UserId
  role: owner | member
  joinedAt: timestamp
```

The WebSocket coordinator tracks whether a member is currently connected. It does not store that connection state in the database.

### Queue item

```text
QueueItem
  id: UUID
  roomId: UUID
  videoId: string
  addedBy: UserId
  sortOrder: integer
  createdAt: timestamp
```

`videoId` must match the YouTube video ID format: 11 characters from `A-Z`, `a-z`, `0-9`, `_`, or `-`. The API accepts an ID, not a full YouTube URL.

The queue order is the ascending `sortOrder`. Queue item IDs remain stable when another item is moved or removed.

### Playback clock

The server stores a position base and the time at which the base changed.

When `playbackStatus` is `playing`, a client calculates the current position as:

```text
playbackPositionMs + max(0, snapshot.serverTime - playbackChangedAt)
```

When `playbackStatus` is `paused`, the stored position is already current. The server materializes the position before it handles `pause`, `seek`, `load`, or `next`.

The server does not store a video duration. The controller sends `playback.ended` when the YouTube player reports the end of the current video.

## Authentication and transport

All room endpoints require a valid Better Auth session. The browser sends the existing session cookie with HTTP requests and with the WebSocket upgrade.

The WebSocket handler must:

- Validate the session during the upgrade.
- Check the request `Origin` against the configured frontend origins.
- Check that the user is a member of the requested room.
- Re-check membership and session validity when the connection sends a command.
- Close the connection when the session expires or the user is removed.

Do not put a session token in the WebSocket URL. Query strings are commonly logged.

The current HTTP CORS layer is permissive. Production configuration should replace it with the actual frontend origins before the room API is exposed.

## HTTP API

The HTTP API handles room lifecycle and initial page loads. It does not handle queue or playback mutations.

### Create a room

```http
POST /api/v1/rooms
Cookie: better-auth.session_token=...
Content-Type: application/json

{}
```

Response: `201 Created`

```json
{
  "room": {
    "id": "6e4d9cf5-8fe9-4c2d-bf2e-ecf8a4b2dd2e",
    "joinCode": "K7M4QX2P",
    "ownerId": "user_123",
    "createdAt": "2026-10-31T10:00:00Z"
  }
}
```

The creator becomes the owner and the first member. The queue starts empty, playback is paused, and `stateVersion` is `0`.

### Join a room

```http
POST /api/v1/rooms/join
Cookie: better-auth.session_token=...
Content-Type: application/json

{
  "joinCode": "K7M4QX2P"
}
```

Response: `200 OK`

```json
{
  "room": {
    "id": "6e4d9cf5-8fe9-4c2d-bf2e-ecf8a4b2dd2e",
    "joinCode": "K7M4QX2P",
    "ownerId": "user_123",
    "createdAt": "2026-10-31T10:00:00Z"
  }
}
```

Joining an existing membership is idempotent and returns `200 OK` without creating a second membership row.

### Get the current room

```http
GET /api/v1/rooms/{roomId}
Cookie: better-auth.session_token=...
```

Response: `200 OK`

```json
{
  "room": {
    "id": "6e4d9cf5-8fe9-4c2d-bf2e-ecf8a4b2dd2e",
    "joinCode": "K7M4QX2P",
    "ownerId": "user_123",
    "createdAt": "2026-10-31T10:00:00Z"
  },
  "state": {
    "stateVersion": 12,
    "serverTime": "2026-10-31T10:05:00Z",
    "queue": [],
    "playback": {
      "itemId": null,
      "status": "paused",
      "positionMs": 0,
      "changedAt": "2026-10-31T10:00:00Z"
    }
  }
}
```

The HTTP response and the first WebSocket snapshot use the same state shape.

### Leave a room

```http
DELETE /api/v1/rooms/{roomId}/membership
Cookie: better-auth.session_token=...
```

Response: `204 No Content`

The owner cannot leave without transferring ownership. Ownership transfer is out of scope for the first version. The owner must delete the room instead.

### Delete a room

```http
DELETE /api/v1/rooms/{roomId}
Cookie: better-auth.session_token=...
```

Response: `204 No Content`

Only the owner can delete a room. The operation deletes the room, its memberships, and its queue items in one transaction. Connected clients receive a `room.deleted` message before the server closes their connections.

## WebSocket API

### Connect

```text
GET /api/v1/rooms/{roomId}/ws
Upgrade: websocket
Cookie: better-auth.session_token=...
```

After the connection opens, the server sends one `state.sync` message. The client must treat that message as the complete current state and discard any locally queued commands that were based on an older state.

### Message envelope

State-changing client messages use this shape:

```json
{
  "type": "queue.add",
  "commandId": "f8e0b7dd-1231-4a4a-97fd-2b4e7a9c4b30",
  "expectedVersion": 12,
  "videoId": "dQw4w9WgXcQ",
  "beforeItemId": null
}
```

Rules for commands:

- `commandId` is a UUID generated by the client for every state-changing command.
- `expectedVersion` is optional. When present, it must equal the current room version.
- The server orders commands by arrival at the room coordinator.
- The server ignores client timestamps.
- A command either commits all of its changes or commits nothing.
- Retrying the same `commandId` does not apply the command twice.

The server broadcasts a full state after every accepted command:

```json
{
  "type": "state.sync",
  "reason": "command.accepted",
  "commandId": "f8e0b7dd-1231-4a4a-97fd-2b4e7a9c4b30",
  "state": {
    "stateVersion": 13,
    "serverTime": "2026-10-31T10:05:12.400Z",
    "queue": [
      {
        "id": "2c93d7f3-9c9d-47b4-9b0d-e311e02b3d9e",
        "videoId": "dQw4w9WgXcQ",
        "addedBy": "user_123",
        "sortOrder": 0,
        "createdAt": "2026-10-31T10:05:12.400Z"
      }
    ],
    "playback": {
      "itemId": null,
      "status": "paused",
      "positionMs": 0,
      "changedAt": "2026-10-31T10:05:12.400Z"
    }
  }
}
```

The server sends the same state to the command sender and to every other connection in the room. This removes the need for a separate command acknowledgement protocol.

### Queue commands

#### `queue.add`

Any member can add a video.

```json
{
  "type": "queue.add",
  "commandId": "uuid",
  "expectedVersion": 12,
  "videoId": "dQw4w9WgXcQ",
  "beforeItemId": null
}
```

The server creates a queue item. When `beforeItemId` is `null`, the server appends the item.

#### `queue.remove`

The owner can remove any item. A member can remove an item that they added.

```json
{
  "type": "queue.remove",
  "commandId": "uuid",
  "expectedVersion": 13,
  "itemId": "2c93d7f3-9c9d-47b4-9b0d-e311e02b3d9e"
}
```

If the removed item is current, the server selects the next item and pauses at position `0`. If there is no next item, the server clears the current item and keeps playback paused.

#### `queue.move`

Only the owner can reorder the queue.

```json
{
  "type": "queue.move",
  "commandId": "uuid",
  "expectedVersion": 14,
  "itemId": "2c93d7f3-9c9d-47b4-9b0d-e311e02b3d9e",
  "beforeItemId": null
}
```

`beforeItemId: null` moves the item to the end.

### Playback commands

Only the owner can send playback commands in the first version.

#### `playback.load`

```json
{
  "type": "playback.load",
  "commandId": "uuid",
  "expectedVersion": 15,
  "itemId": "2c93d7f3-9c9d-47b4-9b0d-e311e02b3d9e",
  "positionMs": 0,
  "autoplay": false
}
```

The item must exist in the room queue. `autoplay: false` sets the item and pauses it. `autoplay: true` sets the item and starts playback.

#### `playback.play`

```json
{
  "type": "playback.play",
  "commandId": "uuid",
  "expectedVersion": 16
}
```

The room must have a current item. The server keeps the current position and sets the status to `playing`.

#### `playback.pause`

```json
{
  "type": "playback.pause",
  "commandId": "uuid",
  "expectedVersion": 17
}
```

The server first calculates the current position, then sets the status to `paused`.

#### `playback.seek`

```json
{
  "type": "playback.seek",
  "commandId": "uuid",
  "expectedVersion": 18,
  "positionMs": 92400
}
```

`positionMs` must be non-negative. Seeking does not change whether the room is playing or paused.

The client should send one seek command when the user finishes dragging the seek bar. It should not send a command for every pointer movement.

#### `playback.next`

```json
{
  "type": "playback.next",
  "commandId": "uuid",
  "expectedVersion": 19
}
```

The server selects the next queue item. It keeps the current playing status if the room was playing and pauses if the room was paused. If there is no next item, it clears the current item and pauses.

#### `playback.ended`

```json
{
  "type": "playback.ended",
  "commandId": "uuid",
  "expectedVersion": 20,
  "itemId": "2c93d7f3-9c9d-47b4-9b0d-e311e02b3d9e"
}
```

The server accepts this command only when `itemId` is the current item. It applies the same transition as `playback.next` and keeps the room playing when a next item exists.

### Synchronization commands

#### `state.request`

The client can request a fresh snapshot after reconnecting or detecting a local mismatch.

```json
{
  "type": "state.request"
}
```

The server responds with `state.sync` and `reason: "requested"`. This command does not change `stateVersion`.

### Server messages

The server sends these message types:

| Message | Meaning |
| --- | --- |
| `state.sync` | Complete queue and playback state. Sent after connect, after an accepted command, and after `state.request`. |
| `presence.sync` | Current members and connection presence. This does not change `stateVersion`. |
| `server.error` | A command failed validation or could not be applied. |
| `room.deleted` | The owner deleted the room. The server closes the connection after sending this message. |
| `server.ping` | Application heartbeat. The client replies with `client.pong`. |

Presence is separate from the versioned state because a browser can disconnect without changing the room queue or playback.

### Errors

```json
{
  "type": "server.error",
  "commandId": "uuid",
  "code": "VERSION_CONFLICT",
  "message": "The room changed before this command arrived.",
  "state": {
    "stateVersion": 21,
    "serverTime": "2026-10-31T10:06:00Z",
    "queue": [],
    "playback": {
      "itemId": null,
      "status": "paused",
      "positionMs": 0,
      "changedAt": "2026-10-31T10:06:00Z"
    }
  }
}
```

The server should include the latest state for `VERSION_CONFLICT` so the client can render the correct room without making another request.

Initial error codes:

| Code | Meaning |
| --- | --- |
| `UNAUTHENTICATED` | The session is missing or expired. |
| `NOT_MEMBER` | The user does not belong to the room. |
| `FORBIDDEN` | The user's role cannot run the command. |
| `INVALID_MESSAGE` | The message is not valid JSON or does not match a known command. |
| `INVALID_VIDEO_ID` | `videoId` is not a valid YouTube ID. |
| `INVALID_COMMAND_ID` | `commandId` is not a UUID. |
| `INVALID_POSITION` | `positionMs` is negative. |
| `ITEM_NOT_FOUND` | The queue item does not exist in this room. |
| `QUEUE_LIMIT_REACHED` | The room already has 100 queue items. |
| `NO_CURRENT_ITEM` | Playback was requested without a selected queue item. |
| `STALE_PLAYBACK_EVENT` | An ended event refers to a queue item that is no longer current. |
| `VERSION_CONFLICT` | `expectedVersion` does not match the current room version. |
| `COMMAND_REPLAY` | The command ID was already used with a different command body. |
| `RATE_LIMITED` | The connection sent too many commands. |
| `ROOM_DELETED` | The room no longer exists. |

HTTP errors should use the same `code` and `message` fields. The existing `ApiError` response can keep its `message` field and add `code` for consistency.

## State transition rules

The room service owns these rules. HTTP handlers, WebSocket handlers, and database adapters must not implement their own copies.

1. The room service loads or receives the current room state.
2. The protocol layer parses the incoming JSON into a typed command.
3. The authorization layer checks the member role.
4. The room service validates references and applies the command.
5. The persistence layer commits the room and queue changes in one database transaction.
6. The service increments `stateVersion` exactly once.
7. The WebSocket coordinator broadcasts the committed full snapshot.

The service must not broadcast before the transaction commits. If broadcasting fails, the next client request or reconnect reads the committed state from the database.

## Persistence plan

Add a migration with these tables:

### `rooms`

- `id` primary key
- `owner_id` foreign key to `users.id`
- `join_code` unique, fixed-length generated value
- `state_version` non-negative integer
- `current_item_id` nullable foreign key to `room_queue_items.id`
- `playback_status`
- `playback_position_ms`
- `playback_changed_at`
- `created_at`
- `updated_at`

### `room_members`

- composite primary key of `room_id` and `user_id`
- `role`
- `joined_at`

Add indexes on `user_id` and `join_code`.

### `room_queue_items`

- `id` primary key
- `room_id` foreign key
- `video_id`
- `added_by` foreign key to `users.id`
- `sort_order`
- `created_at`

Add an index on `(room_id, sort_order)` and an index on `(room_id, id)`.

### `room_commands`

Use this table to make command retries safe across reconnects and process restarts.

- `room_id`
- `command_id`
- `user_id`
- `command_type`
- `command_hash`
- `result_version`
- `created_at`
- unique key on `(room_id, command_id)`

The service inserts the command record in the same transaction as the state change. If the same command ID arrives with a different hash, return `COMMAND_REPLAY`. If the hash matches, return the previously committed result without applying the mutation again.

Use a cleanup task or retention policy for old command records after the retry window has passed.

## Backend module layout

Add the feature under `backend/src/features/music/`.

```text
backend/src/features/music/
  mod.rs            Route registration and feature state.
  domain.rs         Room state, commands, and state transitions.
  protocol.rs       WebSocket message types and wire validation.
  service.rs        Authorization and transactional use cases.
  repository.rs     SeaORM queries and transaction boundaries.
  http.rs           Room lifecycle handlers.
  websocket.rs      Upgrade, connection lifecycle, and message loop.
  coordinator.rs    Per-room serialization and connection broadcast.
```

Update `AppState` to hold a shared room coordinator. Keep the coordinator behind an application-owned type rather than placing a global map in a handler.

The intended dependency direction is:

```text
HTTP and WebSocket adapters
  -> music service
    -> music domain
    -> repository
    -> room coordinator
```

The domain types must not depend on Axum, WebSocket frames, or SeaORM models. Parse and validate network data at the protocol boundary, then pass typed commands to the service.

## Concurrency and scaling

For the first deployment, the coordinator serializes commands for a room in one process. The database transaction remains the final guard against stale writes.

If the backend runs more than one instance, add one of these before allowing clients on different instances to share rooms:

- A distributed room coordinator that routes every room to one owner.
- A Redis or similar pub/sub channel for room snapshots, combined with database row locking.
- A database-backed event or notification stream.

Do not rely on an in-memory broadcast channel alone. It loses updates when two clients connect to different instances.

## Reconnect behavior

When a client reconnects, it should:

1. Open the WebSocket again.
2. Replace local queue and playback state with the first `state.sync` message.
3. Use the returned `serverTime` to estimate the clock offset.
4. Render the current playback position from `changedAt` and the local monotonic clock.
5. Send new commands with the latest `stateVersion`.

The server does not replay an event history. The full snapshot is the recovery mechanism.

## Rate limits and operational rules

- Limit room membership and WebSocket connections per user.
- Rate-limit `queue.add`, `queue.move`, and `queue.remove`.
- Rate-limit `playback.seek` and ask the client to send seeks on drag end.
- Send an application heartbeat and close connections that miss the heartbeat deadline.
- Cap the queue length per room.
- Cap the number of connected clients per room.
- Log room ID, user ID, command type, command ID, result, and state version. Do not log session cookies.
- Record command latency and WebSocket disconnect reasons.

The first queue limit can be a configuration value. The API should return `QUEUE_LIMIT_REACHED` when the limit is exceeded.

## Implementation order

1. Add the room, membership, queue item, and command migrations.
2. Add SeaORM entities and typed domain models.
3. Implement room creation, joining, reading, leaving, and deletion.
4. Implement the transactional command service and state transition tests.
5. Add the per-room coordinator and WebSocket upgrade.
6. Add the message protocol, full snapshots, errors, heartbeat, and reconnect handling.
7. Add authorization, rate limits, queue limits, and origin checks.
8. Add integration tests with multiple WebSocket clients.
9. Add the frontend client that treats `state.sync` as authoritative.

## Test cases

The backend should prove these cases before the frontend depends on the API:

- Two members receive the same snapshot after one member adds a video.
- A duplicate `commandId` does not create a duplicate queue item.
- A reused `commandId` with a different body returns `COMMAND_REPLAY`.
- A stale `expectedVersion` returns `VERSION_CONFLICT` and the latest state.
- Two concurrent queue changes commit in a deterministic order.
- A member cannot reorder another member's queue item.
- A member cannot control playback when the room policy is owner-only.
- Removing the current item selects the next item or clears playback correctly.
- Pausing a playing room stores the materialized current position.
- Seeking while paused and while playing preserves the expected status.
- `playback.ended` advances only when the reported item is current.
- A client that joins after playback starts receives a position that accounts for elapsed server time.
- A room deletion notifies all connected clients and closes their connections.
- A backend restart preserves the room, queue, and playback state.

## Later extensions

These features are intentionally outside the first version:

- Transfer room ownership.
- A room setting that lets all members control playback.
- Queue history and recently played items.
- YouTube metadata caching for titles and thumbnails.
- An event history for audit and time-travel debugging.
- Multi-instance room coordination.
