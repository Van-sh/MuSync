import { Link } from "@tanstack/react-router";
import { useTheme } from "better-themes";
import { Computer, Moon, Sun } from "lucide-react";

import { Avatar, AvatarFallback, AvatarImage } from "~/components/ui/avatar";
import { Button } from "~/components/ui/button";
import {
   DropdownMenu,
   DropdownMenuContent,
   DropdownMenuGroup,
   DropdownMenuItem,
   DropdownMenuLabel,
   DropdownMenuSeparator,
   DropdownMenuTrigger,
} from "~/components/ui/dropdown-menu";
import { Skeleton } from "~/components/ui/skeleton";
import { ToggleGroup, ToggleGroupItem } from "~/components/ui/toggle-group";
import { authClient } from "~/lib/auth-client";

export default function UserArea() {
   const { theme, setTheme } = useTheme();
   const { data: session, isPending } = authClient.useSession();
   const user = session?.user;

   if (isPending) {
      return <Skeleton className="h-8 w-8 rounded-full" />;
   }

   if (!user) {
      return (
         <Button render={<Link to="/login" />} nativeButton={false}>
            Log In
         </Button>
      );
   }

   const initial = user.name.charAt(0).toUpperCase() || "U";

   return (
      <DropdownMenu>
         <DropdownMenuTrigger className="cursor-pointer rounded-full outline-offset-1 hover:outline-2">
            <Avatar className="h-8 w-8">
               <AvatarImage src={user.image ?? undefined} alt={user.name} />
               <AvatarFallback>{initial}</AvatarFallback>
            </Avatar>
         </DropdownMenuTrigger>
         <DropdownMenuContent align="end" className="w-48">
            <DropdownMenuGroup>
               <DropdownMenuLabel className="flex flex-col">
                  <span className="text-xs font-medium">{user.name || "User"}</span>
                  {user.email && (
                     <span className="truncate text-muted-foreground">{user.email}</span>
                  )}
               </DropdownMenuLabel>
            </DropdownMenuGroup>
            <DropdownMenuSeparator />
            <DropdownMenuGroup className="flex items-center justify-between">
               <DropdownMenuLabel className="text-sm font-normal text-foreground">
                  Theme
               </DropdownMenuLabel>
               <ToggleGroup
                  size={"sm"}
                  variant={"outline"}
                  multiple={false}
                  spacing={0}
                  value={[theme!]}
                  onValueChange={(newTheme) => setTheme(newTheme[0])}
               >
                  <ToggleGroupItem value={"system"}>
                     <Computer />
                  </ToggleGroupItem>
                  <ToggleGroupItem value={"light"}>
                     <Sun />
                  </ToggleGroupItem>
                  <ToggleGroupItem value={"dark"}>
                     <Moon />
                  </ToggleGroupItem>
               </ToggleGroup>
            </DropdownMenuGroup>
            <DropdownMenuSeparator />
            <DropdownMenuGroup>
               <DropdownMenuItem onClick={() => authClient.signOut()} className="cursor-pointer">
                  Log Out
               </DropdownMenuItem>
            </DropdownMenuGroup>
         </DropdownMenuContent>
      </DropdownMenu>
   );
}
