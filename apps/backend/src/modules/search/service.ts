import yts from "yt-search";

import type { SearchModels } from "./model";

export abstract class Search {
   static async searchVideos({ q: searchQuery }: SearchModels.searchQuery) {
      const query = searchQuery ?? "";
      if (!query) {
         return [] as SearchModels.searchResponse;
      }
      const results = await yts(query);
      const videos = results.videos.slice(0, 10);
      const formattedVideos = videos.map((video) => ({
         videoId: video.videoId,
         title: video.title,
         authorName: video.author.name,
         thumbnail: video.thumbnail,
         duration: video.duration,
      }));

      return formattedVideos satisfies SearchModels.searchResponse;
   }
}
