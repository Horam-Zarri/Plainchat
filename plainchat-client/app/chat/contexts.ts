import { createContext, Dispatch, SetStateAction } from "react";
import { Socket } from "socket.io-client";

export const WebSocketContext = createContext<Socket>({} as Socket);
export const SideMenuContext = createContext<Dispatch<SetStateAction<boolean>>>
    ({} as Dispatch<SetStateAction<boolean>>);
export const SelectedChatContext = createContext<Dispatch<SetStateAction<string>>>
    ({} as Dispatch<SetStateAction<string>>);

