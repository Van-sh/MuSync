import { jwt as jwtPlugin } from "@elysiajs/jwt";
import { t } from "elysia";

export function jwt() {
   return jwtPlugin({
      secret: Bun.env.JWT_SECRET!,
      schema: t.Object({
         username: t.String(),
      }),
      exp: "7d",
   });
}
