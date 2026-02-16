import { createWriteStream, existsSync, mkdirSync } from "fs";

import { YtDlp } from "ytdlp-nodejs";

const _ytdlp = new YtDlp();

const downloadDirPath = "downloads";

export abstract class ytdlp {
   static async downloadVideo(videoId: string) {
      const downloadUrl = `${downloadDirPath}}/${videoId}.mp3`;
      if (!existsSync(downloadUrl)) {
         const videoUrl = `https://www.youtube.com/watch?v=${videoId}`;

         console.log(`getting info for ${videoId}`);
         const videoInfo = await _ytdlp.getInfoAsync(videoUrl);
         console.log(`got info for ${videoInfo.title}`);

         if (videoInfo._type === "playlist") {
            console.error("received id for a playlist");
            throw new Error("Id returned a playlist, not a video");
         }

         const startTime = Bun.nanoseconds();
         console.log(`downloading ${videoInfo.title}`);
         await _ytdlp.downloadAsync(videoUrl, {
            format: {
               filter: "audioonly",
               type: "mp3",
            },
            output: `${downloadDirPath}/${videoId}.mp3`,
            onProgress(prog) {
               console.log(`downloading ${videoInfo.title} | ${prog.percentage_str}`);
            },
         });
         console.log(`downloaded ${videoInfo.title} in ${Bun.nanoseconds() - startTime}ns`);
      } else {
         console.log("already downloaded");
      }
   }
}
