import { z } from "zod";

for (const [key, value] of Object.entries(Bun.env)) {
   if (value === "") {
      delete Bun.env[key];
   }
}

console.log(Bun.env.NODE_ENV);

const envSchema = z.object({
   NODE_ENV: z.enum(["development", "production", "test"]).default("development"),
   LOG_LEVEL: z.enum(["trace", "debug", "info", "warning", "error", "fatal"]),

   PORT: z.coerce.number().default(8000).pipe(z.int()),

   DB_URL: z.url(),
});

export const env = envSchema.parse({
   LOG_LEVEL: Bun.env.NODE_ENV === "development" ? "debug" : "info",
   ...Bun.env,
});
