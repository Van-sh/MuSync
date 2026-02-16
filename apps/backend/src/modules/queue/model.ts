import { t } from "elysia";

import { videoId } from "~/lib/youtube";

export namespace QueueModels {
   export const playBody = t.Object({
      videoId: videoId,
   });
   export type playBody = typeof playBody.static;

   export const playResponseVideoId = t.Object({
      id: videoId,
   });
   export type playResponseVideoId = typeof playResponseVideoId.static;
}
