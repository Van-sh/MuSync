import { treaty } from "@elysia/eden";
import { type App } from "api";

import { env } from "~/env";

export const api = treaty<App>(env.VITE_API_URL, {
   fetch: {
      credentials: "include",
   },
});
