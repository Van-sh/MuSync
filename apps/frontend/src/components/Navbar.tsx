"use client";

import { User } from "lucide-react";

import { signIn, useSession } from "~web/auth-client";
import { Button } from "./ui/button";

export default function Navbar() {
   const { data, isPending } = useSession();
   return (
      <nav className="flex w-full justify-end p-2">
         {isPending ? "Loading" : <UserSection userData={data} />}
      </nav>
   );
}

type UserSectionProps = {
   userData: ReturnType<typeof useSession>["data"];
};

function UserSection({ userData }: UserSectionProps) {
   return userData ? (
      userData.user.name
   ) : (
      <Button
         onClick={() => {
            signIn.social({
               provider: "google",
               callbackURL: "http://localhost:3000/",
            });
         }}
      >
         <User /> Log In With Google
      </Button>
   );
}
