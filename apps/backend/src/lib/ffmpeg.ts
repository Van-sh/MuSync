import { existsSync, mkdirSync, rmSync } from "fs";

export abstract class FFmpeg {
   static async encodeHLS(videoId: string) {
      if (videoId in _encodeProcesses) {
         await _encodeProcesses[videoId]?.exited;
         return;
      }

      const inputPath = `downloads/${videoId}.mp3`;
      const outputPath = `hls/${videoId}`;
      const hlsPath = `${outputPath}/index.m3u8`;
      if (!existsSync(outputPath)) {
         mkdirSync(outputPath, {
            recursive: true,
         });
      }
      console.log(`encoding to ${hlsPath}`);

      _encodeProcesses[videoId] = Bun.spawn(
         [
            "ffmpeg",
            "-i",
            inputPath,
            "-codec:a",
            "aac",
            "-b:a",
            "256k",
            "-hls_time",
            "10",
            "-hls_playlist_type",
            "vod",
            "-hls_segment_filename",
            `${outputPath}/segment%03d.ts`,
            "-start_number",
            "0",
            hlsPath,
         ],
         {
            stderr: "ignore",
            onExit(_proc, exitCode, _signalCode, error) {
               if (exitCode === 0) {
                  console.log(`finished encoding ${inputPath}`);
                  rmSync(inputPath);
                  console.log(`deleted ${inputPath}`);
               } else {
                  console.error(error);
               }
               delete _encodeProcesses[videoId];
            },
         },
      );

      await _encodeProcesses[videoId].exited;
   }
}

const _encodeProcesses: Record<string, Bun.Subprocess> = {};
