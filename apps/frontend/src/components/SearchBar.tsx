"use client";

import { useDebouncedCallback } from "@tanstack/react-pacer";
import { useState, type ChangeEvent, type ComponentProps } from "react";

import { Input } from "./ui/input";

type SearchBarProps = { onSearch: (query: string) => void } & ComponentProps<typeof Input>;

export default function SearchBar({ onSearch, className, ...props }: SearchBarProps) {
   const [query, setQuery] = useState("");
   const debouncedHandleQuery = useDebouncedCallback(
      (query: string) => {
         onSearch(query);
      },
      { wait: 500 },
   );

   function handleSearchChange(event: ChangeEvent<HTMLInputElement>) {
      const newQuery = event.target.value;
      setQuery(newQuery);
      debouncedHandleQuery(newQuery);
   }

   return (
      <Input
         id="search"
         name="search"
         placeholder="Search"
         autoComplete="off"
         value={query}
         onChange={handleSearchChange}
         className={className}
         {...props}
      />
   );
}
