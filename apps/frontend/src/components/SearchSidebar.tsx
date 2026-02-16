import { useQuery } from "@tanstack/react-query";
import { Loader } from "lucide-react";
import { useState } from "react";

import { api } from "~web/api";
import SearchBar from "./SearchBar";
import SearchCard from "./SearchCard";

export default function SearchSidebar() {
   const [query, setQuery] = useState("");

   const { data: searchResults, isLoading } = useQuery({
      queryKey: ["search", query],
      queryFn: async () => {
         const { data, error } = await api.search.get({
            query: { q: query },
         });
         if (error) throw error;
         return data;
      },
   });

   console.log(searchResults);

   function handleSearch(query: string) {
      console.log("Searching ", query);
      setQuery(query);
   }

   return (
      <>
         <SearchBar onSearch={handleSearch} placeholder="Search for Songs" />
         <div className="flex flex-1 shrink-0 basis-0 flex-col gap-2 overflow-y-auto">
            {isLoading ? (
               <Loader size={60} className="animate-spin self-center justify-self-center" />
            ) : (
               searchResults!.map((video) => <SearchCard video={video} key={video.videoId} />)
            )}
         </div>
      </>
   );
}
