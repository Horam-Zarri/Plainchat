"use client";

import TokenStore from "@/app/lib/token";
import { useRouter } from "next/navigation";
import { useEffect, useRef, useState } from "react";

export default function Home() {

    const router = useRouter();
    
    useEffect(() => {
        TokenStore.setToken("", "");
        router.replace("/auth");
        router.refresh();
    }, []);

    return null;
}