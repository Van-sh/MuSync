import { useMutation } from "@tanstack/react-query";
import { createLazyFileRoute } from "@tanstack/react-router";
import { useState } from "react";

import { Button } from "~/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "~/components/ui/card";
import { authClient } from "~/lib/auth-client";
import { createRoomOptions, joinRoomOptions } from "~feat/rooms/api";

export const Route = createLazyFileRoute("/_app/")({ component: Home });

function Home() {
   const navigate = Route.useNavigate();
   const { data: session } = authClient.useSession();
   const [joinCode, setJoinCode] = useState("");
   const createRoom = useMutation(createRoomOptions);
   const joinRoom = useMutation(joinRoomOptions);

   const openRoom = (roomId: string) => navigate({ to: "/room/$roomId", params: { roomId } });

   if (!session?.user) {
      return (
         <main className="mx-auto flex min-h-[calc(100vh-3.5rem)] max-w-6xl items-center px-6 py-16">
            <div className="max-w-2xl space-y-6">
               <p className="text-sm font-semibold tracking-[0.25em] text-primary uppercase">
                  MuSync rooms
               </p>
               <h1 className="text-5xl leading-tight font-semibold tracking-tight sm:text-7xl">
                  Listen together, wherever you are.
               </h1>
               <p className="max-w-xl text-lg text-muted-foreground">
                  Share a room, build a queue, and keep every YouTube player on the same beat.
               </p>
               <Button render={<a href="/login" />} nativeButton={false} size="lg">
                  Sign in to start a room
               </Button>
            </div>
         </main>
      );
   }

   return (
      <main className="mx-auto grid min-h-[calc(100vh-3.5rem)] max-w-6xl gap-8 px-6 py-12 lg:grid-cols-[1.2fr_0.8fr] lg:items-center">
         <section className="space-y-6">
            <p className="text-sm font-semibold tracking-[0.25em] text-primary uppercase">
               MuSync rooms
            </p>
            <h1 className="max-w-3xl text-5xl leading-tight font-semibold tracking-tight sm:text-7xl">
               Your queue. Their speakers. One timeline.
            </h1>
            <p className="max-w-xl text-lg text-muted-foreground">
               Create a room for a shared listening session, or join one with a short code.
            </p>
         </section>

         <section className="grid gap-4">
            <Card>
               <CardHeader>
                  <CardTitle>Start a room</CardTitle>
                  <CardDescription>
                     You become the room owner and can control playback.
                  </CardDescription>
               </CardHeader>
               <CardContent>
                  <Button
                     className="w-full"
                     disabled={createRoom.isPending}
                     onClick={() =>
                        createRoom.mutate(undefined, {
                           onSuccess: (room) => openRoom(room.room.id),
                        })
                     }
                  >
                     {createRoom.isPending ? "Creating…" : "Create room"}
                  </Button>
               </CardContent>
            </Card>

            <Card>
               <CardHeader>
                  <CardTitle>Join a room</CardTitle>
                  <CardDescription>Enter the code shared by the room owner.</CardDescription>
               </CardHeader>
               <CardContent>
                  <form
                     className="flex gap-2"
                     onSubmit={(event) => {
                        event.preventDefault();
                        joinRoom.mutate(joinCode, { onSuccess: (room) => openRoom(room.room.id) });
                     }}
                  >
                     <input
                        value={joinCode}
                        onChange={(event) => setJoinCode(event.target.value.toUpperCase())}
                        placeholder="K7M4QX2P"
                        maxLength={8}
                        className="h-9 min-w-0 flex-1 rounded-lg border border-input bg-transparent px-3 text-sm tracking-[0.2em] uppercase outline-none focus-visible:ring-3 focus-visible:ring-ring/50"
                     />
                     <Button
                        type="submit"
                        variant="secondary"
                        disabled={joinRoom.isPending || joinCode.length < 8}
                     >
                        {joinRoom.isPending ? "Joining…" : "Join"}
                     </Button>
                  </form>
                  {(createRoom.error || joinRoom.error) && (
                     <p className="mt-3 text-sm text-destructive">
                        {(createRoom.error ?? joinRoom.error)?.message}
                     </p>
                  )}
               </CardContent>
            </Card>
         </section>
      </main>
   );
}
