"use client";

import AudioPlayer from "~/components/AudioPlayer";
import Sidebar from "~/components/Sidebar";

export default function Player() {
   return (
      <section className="divide-foreground/30 flex w-full grow gap-2 divide-x-2">
         <div className="h-full grow pr-2 text-center">
            <div className="text-9xl">WIP</div>
            <AudioPlayer />
         </div>
         <Sidebar />
      </section>
   );
}
