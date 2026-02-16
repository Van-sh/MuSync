import { api } from "~/api";
import { useCurrentlyPlayingStore } from "~/store/currentlyPlayingStore";

type SearchCardProps = {
   video: NonNullable<Awaited<ReturnType<typeof api.search.get>>["data"]>[0];
};

export default function SearchCard({ video }: SearchCardProps) {
   const setVideoId = useCurrentlyPlayingStore((state) => state.setVideoId);

   async function handlePlay() {
      console.log(`playing ${video.videoId}`);
      const { data, error } = await api.queue.post({ videoId: video.videoId });
      if (error) {
         console.error(error);
         alert(error);
         return;
      }

      setVideoId(data.id);
   }

   return (
      <div className="hover:bg-muted flex cursor-pointer gap-2 rounded-sm p-1" onClick={handlePlay}>
         {video.thumbnail && (
            <img
               className="rounded-sm"
               src={video.thumbnail}
               alt={video.title}
               height={45}
               width={80}
            />
         )}
         <div className="flex flex-col">
            <span className="line-clamp-2 text-sm whitespace-break-spaces">{video.title}</span>
            <span className="text-muted-foreground line-clamp-1 text-xs whitespace-break-spaces">
               by {video.authorName}
            </span>
         </div>
      </div>
   );
}
