import { betterAuth } from "better-auth";
import { drizzleAdapter } from "better-auth/adapters/drizzle";
import { openAPI } from "better-auth/plugins";

import { db } from "./db";
import { env } from "./env";

export const auth = betterAuth({
   database: drizzleAdapter(db, { provider: "pg" }),
   secret: env.BETTER_AUTH_SECRET,
   baseURL: env.BETTER_AUTH_URL,
   basePath: env.BETTER_AUTH_PATH,
   socialProviders: {
      google: {
         clientId: env.GOOGLE_CLIENT_ID,
         clientSecret: env.GOOGLE_CLIENT_SECRET,
      },
   },
   plugins: [openAPI()],
   trustedOrigins: ["http://localhost:3000"],
});

let _schema: ReturnType<typeof auth.api.generateOpenAPISchema>;
const getSchema = async () => (_schema ??= auth.api.generateOpenAPISchema());

export const OpenAPI = {
   getPaths: (prefix = env.BETTER_AUTH_PATH) =>
      getSchema().then(({ paths }) => {
         const reference: typeof paths = Object.create(null);

         for (const path of Object.keys(paths)) {
            const key = prefix + path;
            // @ts-ignore
            reference[key] = paths[path];

            // @ts-ignore
            for (const method of Object.keys(paths[path])) {
               const operation = (reference[key] as any)[method];

               operation.tags = ["Better Auth"];
            }
         }

         return reference;
      }) as Promise<any>,
   components: getSchema().then(({ components }) => components) as Promise<any>,
} as const;
