import { t } from "elysia";

import { videoId } from "~api/lib/youtube";

export namespace SearchModels {
   export const searchQuery = t.Object({
      q: t.Optional(t.String()),
   });
   export type searchQuery = typeof searchQuery.static;

   export const searchResult = t.Object({
      videoId,
      title: t.String(),
      authorName: t.String(),
      thumbnail: t.Optional(t.String()),
      duration: t.Any(),
   });
   export type searchResult = typeof searchResult.static;

   export const searchResponse = t.Array(searchResult);
   export type searchResponse = typeof searchResponse.static;
}
