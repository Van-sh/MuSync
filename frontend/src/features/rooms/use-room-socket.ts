import type { QueryClient } from "@tanstack/react-query";
import { useEffect, useRef } from "react";

import { getRoomOptions, getRoomWebSocketUrl, type RoomState } from "./api";

interface StateSyncMessage {
   type: "state.sync";
   reason: string;
   commandId: string | null;
   state: RoomState;
}

interface ServerErrorMessage {
   type: "server.error";
   commandId: string | null;
   code: string;
   message: string;
   state: RoomState | null;
}

interface RoomDeletedMessage {
   type: "room.deleted";
}

type ServerMessage =
   | StateSyncMessage
   | ServerErrorMessage
   | RoomDeletedMessage
   | { type: "server.ping" };

export function useRoomSocket(
   roomId: string,
   queryClient: QueryClient,
   onError: (message: string) => void,
) {
   const socketRef = useRef<WebSocket | null>(null);

   useEffect(() => {
      const socket = new WebSocket(getRoomWebSocketUrl(roomId));
      socketRef.current = socket;

      socket.addEventListener("message", (event) => {
         const message = JSON.parse(event.data) as ServerMessage;
         if (message.type === "state.sync") {
            queryClient.setQueryData(getRoomOptions(roomId).queryKey, (current) =>
               current ? { ...current, state: message.state } : current,
            );
         } else if (message.type === "server.error") {
            const nextState = message.state;
            if (nextState) {
               queryClient.setQueryData(getRoomOptions(roomId).queryKey, (current) =>
                  current ? { ...current, state: nextState } : current,
               );
            }
            onError(message.message);
         } else {
            if (message.type === "room.deleted") {
               onError("This room was deleted by its owner.");
            } else {
               socket.send(JSON.stringify({ type: "client.pong" }));
            }
         }
      });

      socket.addEventListener("error", () => onError("The room connection failed."));
      socket.addEventListener("close", () => {
         if (socketRef.current === socket) {
            socketRef.current = null;
         }
      });

      return () => {
         socket.close();
         socketRef.current = null;
      };
   }, [onError, queryClient, roomId]);

   return (message: object) => {
      if (socketRef.current?.readyState !== WebSocket.OPEN) {
         onError("The room connection is not ready yet.");
         return;
      }
      socketRef.current.send(JSON.stringify(message));
   };
}
