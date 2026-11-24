import { useMutation, useQueryClient, useSuspenseQuery } from "@tanstack/react-query";
import { createFileRoute, Link, useNavigate } from "@tanstack/react-router";
import { useCallback, useState, type FormEvent } from "react";

import { Button } from "~/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "~/components/ui/card";
import { authClient } from "~/lib/auth-client";
import {
   deleteRoomOptions,
   getRoomOptions,
   leaveRoomOptions,
   type QueueItem,
} from "~feat/rooms/api";
import { useRoomSocket } from "~feat/rooms/use-room-socket";
import { YouTubePlayerView } from "~feat/rooms/youtube-player";

export const Route = createFileRoute("/_app/room/$roomId")({
   loader: async ({ context, params }) => {
      await context.queryClient.prefetchQuery(getRoomOptions(params.roomId));
   },
   component: RoomPage,
});

function RoomPage() {
   const { roomId } = Route.useParams();
   const navigate = useNavigate();
   const queryClient = useQueryClient();
   const { data } = useSuspenseQuery(getRoomOptions(roomId));
   const { data: session } = authClient.useSession();
   const [videoId, setVideoId] = useState("");
   const [error, setError] = useState<string | null>(null);
   const onSocketError = useCallback((message: string) => setError(message), []);
   const send = useRoomSocket(roomId, queryClient, onSocketError);
   const leaveRoom = useMutation(leaveRoomOptions);
   const deleteRoom = useMutation(deleteRoomOptions);

   const command = useCallback(
      (payload: Record<string, unknown>) => {
         send({
            ...payload,
            commandId: crypto.randomUUID(),
            expectedVersion: data.state.stateVersion,
         });
      },
      [data.state.stateVersion, send],
   );

   const currentItem =
      data.state.queue.find((item) => item.id === data.state.playback.itemId) ??
      data.state.queue.at(0);

   const addVideo = (event: FormEvent<HTMLFormElement>) => {
      event.preventDefault();
      if (!videoId.trim()) return;
      command({ type: "queue.add", videoId: videoId.trim(), beforeItemId: null });
      setVideoId("");
   };

   const exit = () => {
      leaveRoom.mutate(roomId, { onSuccess: () => navigate({ to: "/" }) });
   };

   return (
      <main className="mx-auto max-w-7xl space-y-6 px-6 py-8">
         <header className="flex flex-wrap items-end justify-between gap-4">
            <div>
               <Link to="/" className="text-sm text-muted-foreground hover:text-foreground">
                  ← Back to rooms
               </Link>
               <h1 className="mt-3 text-3xl font-semibold tracking-tight">Listening room</h1>
               <p className="mt-1 text-sm text-muted-foreground">
                  Share code{" "}
                  <span className="font-semibold tracking-[0.2em] text-foreground">
                     {data.room.joinCode}
                  </span>
               </p>
            </div>
            <div className="flex gap-2">
               <Button variant="outline" onClick={exit} disabled={leaveRoom.isPending}>
                  Leave
               </Button>
               {session?.user.id === data.room.ownerId && (
                  <Button
                     variant="destructive"
                     onClick={() =>
                        deleteRoom.mutate(roomId, { onSuccess: () => navigate({ to: "/" }) })
                     }
                     disabled={deleteRoom.isPending}
                  >
                     Delete room
                  </Button>
               )}
            </div>
         </header>

         {error && (
            <div className="rounded-lg border border-destructive/30 bg-destructive/10 px-4 py-3 text-sm text-destructive">
               {error}
            </div>
         )}

         <div className="grid gap-6 lg:grid-cols-[1.35fr_0.65fr]">
            <Card className="overflow-hidden">
               <div className="aspect-video bg-black">
                  {currentItem ? (
                     <YouTubePlayerView
                        item={currentItem}
                        playback={data.state.playback}
                        onEnded={() => command({ type: "playback.ended", itemId: currentItem.id })}
                     />
                  ) : (
                     <div className="flex h-full items-center justify-center text-sm text-white/60">
                        Add a video to start listening.
                     </div>
                  )}
               </div>
               <CardContent className="flex flex-wrap items-center gap-2 py-4">
                  <Button
                     onClick={() => command({ type: "playback.play" })}
                     disabled={!currentItem}
                  >
                     Play
                  </Button>
                  <Button
                     variant="secondary"
                     onClick={() => command({ type: "playback.pause" })}
                     disabled={!currentItem}
                  >
                     Pause
                  </Button>
                  <Button
                     variant="outline"
                     onClick={() => command({ type: "playback.next" })}
                     disabled={!currentItem}
                  >
                     Next
                  </Button>
                  <span className="ml-auto text-xs text-muted-foreground">
                     Version {data.state.stateVersion}
                  </span>
               </CardContent>
            </Card>

            <Card>
               <CardHeader>
                  <CardTitle>Queue</CardTitle>
                  <CardDescription>
                     {data.state.queue.length} video{data.state.queue.length === 1 ? "" : "s"} in
                     this room
                  </CardDescription>
               </CardHeader>
               <CardContent className="space-y-4">
                  <form className="flex gap-2" onSubmit={addVideo}>
                     <input
                        value={videoId}
                        onChange={(event) => setVideoId(event.target.value)}
                        placeholder="YouTube video ID"
                        className="h-9 min-w-0 flex-1 rounded-lg border border-input bg-transparent px-3 text-sm outline-none focus-visible:ring-3 focus-visible:ring-ring/50"
                     />
                     <Button type="submit" size="sm">
                        Add
                     </Button>
                  </form>
                  <div className="space-y-2">
                     {data.state.queue.length === 0 && (
                        <p className="py-8 text-center text-sm text-muted-foreground">
                           The queue is empty.
                        </p>
                     )}
                     {data.state.queue.map((item, index) => (
                        <QueueRow
                           key={item.id}
                           item={item}
                           index={index}
                           active={item.id === data.state.playback.itemId}
                           onLoad={() =>
                              command({
                                 type: "playback.load",
                                 itemId: item.id,
                                 autoplay: false,
                                 positionMs: 0,
                              })
                           }
                           onRemove={() => command({ type: "queue.remove", itemId: item.id })}
                        />
                     ))}
                  </div>
               </CardContent>
            </Card>
         </div>
      </main>
   );
}

function QueueRow({
   item,
   index,
   active,
   onLoad,
   onRemove,
}: {
   item: QueueItem;
   index: number;
   active: boolean;
   onLoad: () => void;
   onRemove: () => void;
}) {
   return (
      <div
         className={`flex items-center gap-3 rounded-lg border p-3 ${active ? "border-primary bg-primary/5" : "border-border"}`}
      >
         <span className="w-5 text-center text-xs text-muted-foreground">{index + 1}</span>
         <div className="min-w-0 flex-1">
            <p className="truncate text-sm font-medium">{item.videoId}</p>
            <p className="text-xs text-muted-foreground">Added by {item.addedBy}</p>
         </div>
         <Button size="sm" variant="ghost" onClick={onLoad}>
            Load
         </Button>
         <Button size="sm" variant="ghost" onClick={onRemove}>
            Remove
         </Button>
      </div>
   );
}
