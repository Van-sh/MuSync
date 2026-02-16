import { treaty } from "@elysiajs/eden";
import { App } from "api";

import { env } from "~web/env";

export const api = treaty<App>(env.NEXT_PUBLIC_API_URL);
