import { drizzle } from "drizzle-orm/bun-sql";

import { env } from "~api/env";
import * as schema from "./schema";

export const db = drizzle(env.DATABASE_URL, {
   schema: schema,
});
