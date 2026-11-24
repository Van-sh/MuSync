import { useEffect, useRef } from "react";

import type { PlaybackState, QueueItem } from "./api";

interface YouTubePlayer {
   loadVideoById: (videoId: string) => void;
   playVideo: () => void;
   pauseVideo: () => void;
   seekTo: (seconds: number, allowSeekAhead: boolean) => void;
   destroy: () => void;
}

interface YouTubeApi {
   Player: new (
      element: HTMLElement,
      options: {
         videoId: string;
         playerVars: { playsinline: 1; rel: 0 };
         events: { onReady: () => void; onStateChange: (event: { data: number }) => void };
      },
   ) => YouTubePlayer;
}

declare global {
   interface Window {
      YT?: YouTubeApi;
      onYouTubeIframeAPIReady?: () => void;
   }
}

let apiPromise: Promise<YouTubeApi> | undefined;

function loadApi(): Promise<YouTubeApi> {
   if (window.YT) return Promise.resolve(window.YT);
   if (apiPromise) return apiPromise;

   apiPromise = new Promise((resolve) => {
      const previousCallback = window.onYouTubeIframeAPIReady;
      window.onYouTubeIframeAPIReady = () => {
         previousCallback?.();
         if (window.YT) resolve(window.YT);
      };

      if (!document.querySelector("script[data-youtube-iframe-api]")) {
         const script = document.createElement("script");
         script.src = "https://www.youtube.com/iframe_api";
         script.dataset.youtubeIframeApi = "true";
         document.head.appendChild(script);
      }
   });

   return apiPromise;
}

export function YouTubePlayerView({
   item,
   playback,
   onEnded,
}: {
   item: QueueItem | undefined;
   playback: PlaybackState;
   onEnded: () => void;
}) {
   const containerRef = useRef<HTMLDivElement>(null);
   const playerRef = useRef<YouTubePlayer | null>(null);
   const onEndedRef = useRef(onEnded);
   const playbackRef = useRef(playback);

   useEffect(() => {
      onEndedRef.current = onEnded;
   }, [onEnded]);

   useEffect(() => {
      playbackRef.current = playback;
   }, [playback]);

   useEffect(() => {
      if (!item || !containerRef.current) return;
      let disposed = false;

      void loadApi().then((api) => {
         if (disposed || !containerRef.current) return;
         playerRef.current?.destroy();
         playerRef.current = new api.Player(containerRef.current, {
            videoId: item.videoId,
            playerVars: { playsinline: 1, rel: 0 },
            events: {
               onReady: () => syncPlayer(playerRef.current, playbackRef.current),
               onStateChange: (event) => {
                  if (event.data === 0) onEndedRef.current();
               },
            },
         });
      });

      return () => {
         disposed = true;
         playerRef.current?.destroy();
         playerRef.current = null;
      };
   }, [item?.id, item?.videoId]);

   useEffect(() => {
      syncPlayer(playerRef.current, playback);
   }, [playback]);

   return item ? <div ref={containerRef} className="h-full w-full" /> : null;
}

function syncPlayer(player: YouTubePlayer | null, playback: PlaybackState) {
   if (!player) return;
   if (playback.status === "playing") {
      player.playVideo();
   } else {
      player.pauseVideo();
   }
   player.seekTo(Math.max(0, playback.positionMs) / 1000, true);
}
