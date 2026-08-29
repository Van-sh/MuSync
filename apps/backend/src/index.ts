import { ansiColorFormatter, configure, getConsoleSink } from "@logtape/logtape";

import { app } from "./app";
import { env } from "./env";

export type { App } from "./app";

await configure({
   sinks: {
      console: getConsoleSink({ formatter: ansiColorFormatter }),
   },
   loggers: [
      { category: ["logtape"], sinks: [] },
      {
         category: ["app"],
         sinks: ["console"],
         lowestLevel: env.LOG_LEVEL,
      },
   ],
   reset: env.NODE_ENV === "development",
});

console.log(`🎵 App listening on ${app.server?.url}`);
