/**
 * Isn't used currently during development
 */

import { Elysia, type Context } from "elysia";

import { auth } from "~/auth";

export const betterAuthView = (context: Context) => {
   const BETTER_AUTH_ACCEPT_METHODS = ["POST", "GET"];
   // validate request method
   if (BETTER_AUTH_ACCEPT_METHODS.includes(context.request.method)) {
      return auth.handler(context.request);
   } else {
      throw context.status(405);
   }
};

export const betterAuth = new Elysia({ name: "better-auth" }).macro({
   auth: {
      async resolve({ status, request: { headers } }) {
         const session = await auth.api.getSession({
            headers,
         });
         if (!session) return status(401);
         return {
            user: session.user,
            session: session.session,
         };
      },
   },
});
