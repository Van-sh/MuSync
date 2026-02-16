import Hls from "hls.js";
import { useEffect, useRef } from "react";

import { useCurrentlyPlayingStore } from "~web/store/currentlyPlayingStore";

export default function AudioPlayer() {
   const videoId = useCurrentlyPlayingStore((state) => state.videoId);

   const audioRef = useRef<HTMLAudioElement>(null);
   const hlsRef = useRef<Hls>(null);
   if (hlsRef.current === null) {
      hlsRef.current = new Hls({
         manifestLoadingMaxRetry: 5,
      });
   }

   useEffect(() => {
      if (!audioRef.current) {
         return;
      }
      if (!videoId) {
         return;
      }

      let videoSrc = `http://localhost:8000/stream/${videoId}/index.m3u8`;
      if (Hls.isSupported()) {
         hlsRef.current!.loadSource(videoSrc);
         hlsRef.current!.attachMedia(audioRef.current);
      }
   }, [videoId]);
   return (
      <audio ref={audioRef} controls autoPlay>
         Your browser does not support audio
      </audio>
   );
}
