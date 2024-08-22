'use client';

import { revalidatePath, revalidateTag } from "next/cache";
import { redirect, useRouter } from "next/navigation";
import { useContext, useEffect, useState } from "react";
import TokenStore from "../lib/token";
import { SideMenuContext } from "./contexts";


export default function Home() {
    const setShowMenu = useContext(SideMenuContext);
    return (
        <>
            <button className="text-4xl p-4 lg:hidden" onClick={(e) => { setShowMenu(true); }}>&#9776;</button>
            <div className="grid place-content-center h-screen">
                <h1 className="text-4xl">
                    Select a chat or create a new one...
                </h1>
            </div>
        </>
    );
}
