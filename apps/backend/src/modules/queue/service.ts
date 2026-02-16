import { existsSync } from "fs";

import { FFmpeg } from "~api/lib/ffmpeg";
import { ytdlp } from "~api/lib/ytdlp";
import type { QueueModels } from "./model";

export abstract class Queue {
   static async play({ videoId }: QueueModels.playBody) {
      if (existsSync(`hls/${videoId}/index.m3u8`)) {
         return videoId;
      }

      await ytdlp
         .downloadVideo(videoId)
         .catch()
         .then(async () => {
            await FFmpeg.encodeHLS(videoId);
         });

      return videoId;
   }
}
