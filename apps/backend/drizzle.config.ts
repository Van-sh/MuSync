import { defineConfig } from "drizzle-kit";

import { env } from "~/env";

export default defineConfig({
   out: "./migrations",
   schema: "./src/db/schema/index.ts",
   dialect: "postgresql",
   dbCredentials: {
      url: env.DATABASE_URL,
   },
   verbose: true,
});
