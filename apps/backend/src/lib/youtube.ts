import { t } from "elysia";

export const videoId = t.String({
   minLength: 11,
   maxLength: 11,
   error: "Not a valid YouTube Video ID",
});
export type videoId = typeof videoId.static;

export const duration = t.Object({
   seconds: t.Integer(),
   timestamp: t.TemplateLiteral("${number}:${number}"),
});
export type duration = typeof duration.static;
