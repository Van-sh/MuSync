import Elysia from "elysia";

import { createResponseTypes } from "~/lib/createResponseTypes";
import { SearchModels } from "./model";
import { Search } from "./service";

export const searchRouter = new Elysia({
   name: "search",
   prefix: "/search",
}).get(
   "/",
   async ({ query, status }) => {
      const searchResults = await Search.searchVideos(query);

      return status(200, searchResults);
   },
   {
      query: SearchModels.searchQuery,
      response: createResponseTypes({ OK: SearchModels.searchResponse }),
   },
);
