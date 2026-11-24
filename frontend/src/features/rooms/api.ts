import { mutationOptions, queryOptions } from "@tanstack/react-query";

import { env } from "~/env";

export interface RoomSummary {
   id: string;
   joinCode: string;
   ownerId: string;
   createdAt: string;
}

export interface QueueItem {
   id: string;
   videoId: string;
   addedBy: string;
   sortOrder: number;
   createdAt: string;
}

export interface PlaybackState {
   itemId: string | null;
   status: "paused" | "playing";
   positionMs: number;
   changedAt: string;
}

export interface RoomState {
   stateVersion: number;
   serverTime: string;
   queue: QueueItem[];
   playback: PlaybackState;
}

export interface RoomResponse {
   room: RoomSummary;
   state: RoomState;
}

export interface ApiErrorBody {
   message?: string;
}

async function request<T>(path: string, init?: RequestInit): Promise<T> {
   const response = await fetch(`${env.VITE_API_URL}${path}`, {
      credentials: "include",
      ...init,
      headers: {
         ...(init?.body ? { "content-type": "application/json" } : {}),
         ...init?.headers,
      },
   });

   if (!response.ok) {
      const body = (await response.json().catch(() => null)) as ApiErrorBody | null;
      throw new Error(body?.message ?? `Request failed with status ${response.status}`);
   }

   if (response.status === 204) {
      return undefined as T;
   }

   return (await response.json()) as T;
}

export const getRoomOptions = (roomId: string) =>
   queryOptions({
      queryKey: ["rooms", roomId],
      queryFn: () => request<RoomResponse>(`/api/v1/rooms/${roomId}`),
   });

export const createRoomOptions = mutationOptions({
   mutationFn: () => request<RoomResponse>("/api/v1/rooms", { method: "POST", body: "{}" }),
});

export const joinRoomOptions = mutationOptions({
   mutationFn: (joinCode: string) =>
      request<RoomResponse>("/api/v1/rooms/join", {
         method: "POST",
         body: JSON.stringify({ joinCode }),
      }),
});

export const leaveRoomOptions = mutationOptions({
   mutationFn: (roomId: string) =>
      request<void>(`/api/v1/rooms/${roomId}/membership`, { method: "DELETE" }),
});

export const deleteRoomOptions = mutationOptions({
   mutationFn: (roomId: string) => request<void>(`/api/v1/rooms/${roomId}`, { method: "DELETE" }),
});

export function getRoomWebSocketUrl(roomId: string): string {
   const url = new URL(`/api/v1/rooms/${roomId}/ws`, env.VITE_API_URL);
   url.protocol = url.protocol === "https:" ? "wss:" : "ws:";
   return url.toString();
}
