import { create } from "zustand";

type CurrentlyPlayingState = {
   videoId: string;
   setVideoId: (videoId: string) => void;
};

export const useCurrentlyPlayingStore = create<CurrentlyPlayingState>()((set) => ({
   videoId: "",
   setVideoId: (videoId) => set({ videoId }),
}));
