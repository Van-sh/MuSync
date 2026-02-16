import { logger } from "@bogeychan/elysia-logger";
import cors from "@elysiajs/cors";
import openapi from "@elysiajs/openapi";
import staticPlugin from "@elysiajs/static";
import { Elysia } from "elysia";

import { auth, OpenAPI } from "./auth";
import { env } from "./env";
import { queueRouter } from "./modules/queue";
import { searchRouter } from "./modules/search";

const PORT = env.PORT || 8000;
const envToLogger = {
   development: {
      transport: {
         target: "pino-pretty",
         options: {
            colorize: true,
         },
      },
   },
   production: {},
   test: { enabled: false },
} as const;

export const app = new Elysia({
   name: "MuSync",
})
   .use(logger({ ...envToLogger[env.NODE_ENV] }))
   .use(
      openapi({
         documentation: {
            info: {
               title: "MuSync",
               description: "MuSync API Documentation",
               version: "1.0.0",
            },
            components: await OpenAPI.components,
            paths: await OpenAPI.getPaths(),
         },
      }),
   )
   .use(
      cors({
         origin: ["http://localhost:3000"],
         methods: ["GET", "POST", "PUT", "DELETE", "OPTIONS"],
         credentials: true,
      }),
   )
   .use(
      staticPlugin({
         assets: "hls",
         prefix: "/stream",
      }),
   )
   .mount("", auth.handler)
   .use(searchRouter)
   .use(queueRouter)
   .listen(PORT);
export type App = typeof app;

console.log(`🎵 App listening on ${app.server?.url}`);
