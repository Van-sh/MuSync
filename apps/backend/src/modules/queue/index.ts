import Elysia from "elysia";

import { createResponseTypes } from "~api/lib/createResponseTypes";
import { QueueModels } from "./model";
import { Queue } from "./service";

export const queueRouter = new Elysia({
   prefix: "/queue",
}).post(
   "/",
   async ({ body, status }) => {
      const videoId = await Queue.play(body);

      return status(200, { id: videoId });
   },
   {
      body: QueueModels.playBody,
      response: createResponseTypes({ OK: QueueModels.playResponseVideoId }),
   },
);
