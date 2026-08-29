import { elysiaLogger } from "@logtape/elysia";
import Elysia from "elysia";

import { env } from "./env";

export const app = new Elysia({ name: "MuSync" })
   .use(
      elysiaLogger({
         category: ["app", "elysia"],
         format: "dev",
         scope: "global",
      }),
   )
   .listen(env.PORT);

export type App = typeof app;
