import { getLogger } from "@logtape/drizzle-orm";
import { drizzle } from "drizzle-orm/bun-sql";

import { env } from "~/env";

export const db = drizzle(env.DB_URL, {
   logger: getLogger({ category: ["app", "db"] }),
});
