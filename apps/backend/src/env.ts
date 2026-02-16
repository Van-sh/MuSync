import z from "zod";

for (const [key, value] of Object.entries(Bun.env)) {
   if (value === "") {
      delete Bun.env[key];
   }
}

const envSchema = z.object({
   PORT: z.coerce.number(),

   DATABASE_URL: z.url().min(1),

   BETTER_AUTH_SECRET: z.string().min(1),
   BETTER_AUTH_URL: z.url().min(1),
   BETTER_AUTH_PATH: z.string().startsWith("/").default("/auth"),

   GOOGLE_CLIENT_ID: z.string().min(1),
   GOOGLE_CLIENT_SECRET: z.string().min(1),

   NODE_ENV: z.enum(["development", "production", "test"]).default("development"),
});

export const env = envSchema.parse(Bun.env);
