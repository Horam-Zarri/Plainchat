"use client";

import { useRouter } from "next/navigation";
import { Dispatch, MouseEventHandler, SetStateAction, useEffect, useRef, useState } from "react";
import TokenStore from "../lib/token";
import GroupItem from "../ui/groupItem";
import { io, Socket } from "socket.io-client";
import { createContext } from "react";
import { SettingsButton, SubmitButton } from "../ui/submitButton";
import Modal from "../ui/modal";
import Link from "next/link";
import InputLabel from "../ui/inputLabel";
import { SimpleTextInput, TextInput } from "../ui/inputs";
import { revalidatePath } from "next/cache";
import { WebSocketContext, SideMenuContext, SelectedChatContext } from "./contexts";
import { API_BASE_URL } from "../lib/globals";
import GroupItems from "../ui/groupItems";
type GroupStat = {
    id: string,
    name: string,
}

export default function ChatLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {

    const [socket, setSocket] = useState<Socket>();
    const [isSocketLoading, setIsSocketLoading] = useState(true);
    const [isGroupsLoading, setIsGroupsLoading] = useState(true);
    const [groupStats, setGroupStats] = useState<GroupStat[]>([]);
    const [selectedChat, setSelectedChat] = useState("");

    const [showMenu, setShowMenu] = useState(false);
    const [isUserModalOpen, setIsUserModalOpen] = useState(false);
    const [isGroupModalOpen, setIsGroupModalOpen] = useState(false);
    const [reloadGroup, setReloadGroup] = useState(false);

    const onceConnect = useRef(false);

    const router = useRouter();

    useEffect(() => {
        if (onceConnect.current) {
            return;
        }

        onceConnect.current = true;

        const socket = io("http://localhost:5000", {
            extraHeaders: {
                'Authorization': TokenStore.getToken(),
            }
        });
        socket.on("disconnect", () => {
            console.log("SOCKET DC!!!!!");
        });
        setSocket(socket);
        setIsSocketLoading(false);
    }, [])

    useEffect(() => {
        setIsGroupModalOpen(false);
        setReloadGroup(false);

        fetch(API_BASE_URL + "group", {
            method: "GET",
            headers: {
                "Authorization": TokenStore.getToken()
            },
        }).then((res) => res.json())
        .then((data) => {
            const err = data.error;
            if (err) {
                if (err.token) {
                    router.replace("/auth");
                    router.refresh();
                } else {
                    console.log(err.msg);
                    router.refresh();
                }
            } else {
                const datas = data as GroupStat[];
                setGroupStats(datas);
                setIsGroupsLoading(false);
            }
        })
        .catch((e) => {
            console.log(e);
            setIsGroupsLoading(false);
            setReloadGroup(false);
            router.replace("/auth");
            router.refresh();
        })
    }, [reloadGroup]);

    const logout = async () => {
        socket?.disconnect();
        router.replace("/auth/logout");
        router.refresh();
    }

    if (isSocketLoading || isGroupsLoading) {
        return <h1>LOADING...</h1>;
    }

    const documentStyle = `${(isUserModalOpen || isGroupModalOpen) && "opacity-25 pointer-events-none"}`;
    const sideBarStyle = `fixed z-10 bg-white text-black h-screen flex shadow-[5px_5px_0px_0px_var(--color-5)]
            flex-col ${showMenu ? "w-full" : "hidden lg:flex lg:flex-col"} lg:w-1/4`;
    const mainContentStyle = `${showMenu ? "w-3/4 ml-[25vw]" : "w-full lg:w-3/4 lg:ml-[25vw]"}`

    return (
        <div className={documentStyle}>
            <div className={sideBarStyle}>
                <div className="flex flex-col h-[80vh] overflow-y-scroll">
                    <button className="self-start ml-4 mt-2 text-3xl lg:hidden" onClick={(e) => { setShowMenu(false) }}>X</button>
                    <GroupItems gstats={groupStats} selectedChatId={selectedChat}/>
                </div>
                <div id="control-center" className="w-3/4 place-self-center py-8 flex flex-col gap-y-4">
                    <SettingsButton content="Add chat" border={true} shadow={false} onClick={() => { setIsGroupModalOpen(true); }} />
                    <GroupModal isOpen={isGroupModalOpen} onClose={() => { setIsGroupModalOpen(false); }} setReloadGroup={setReloadGroup}/>
                    <SettingsButton content="User Settings" border={true} shadow={false} onClick={() => { setIsUserModalOpen(true); }} />
                    <UserModal isOpen={isUserModalOpen} onClose={() => { setIsUserModalOpen(false); }}  logout={logout}/>
                </div>
            </div>
            <WebSocketContext.Provider value={socket as Socket}>
                <SideMenuContext.Provider value={setShowMenu}>
                    <SelectedChatContext.Provider value={setSelectedChat}>
                        <div className={mainContentStyle}>
                            {children}
                        </div>
                    </SelectedChatContext.Provider>
                </SideMenuContext.Provider>
            </WebSocketContext.Provider>
        </div>
    );
}

function GroupModal({ isOpen, onClose, setReloadGroup }: { isOpen: boolean, onClose: MouseEventHandler<HTMLButtonElement>, setReloadGroup: Dispatch<SetStateAction<boolean>> }) {

    const [dataChanged, setDataChanged] = useState(false);
    const [groupName, setGroupName] = useState("");
    const [err, setErr] = useState("");

    const router = useRouter();

    const addGroup = async () => {
        if (!dataChanged) {
            return;
        }

        const res = await fetch(API_BASE_URL + "group", {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
                "Authorization": TokenStore.getToken(),
            },
            body: JSON.stringify({
                name: groupName
            })
        }).then((res) => res.json()).catch((e) => {
            setErr("Server Error");
            console.log(e);
        });

        const err = res.error;
        if (err) {
            if (err.token) {
                router.replace("/auth");
            } else {
                setErr(err.msg);
            }
        } else {
            setReloadGroup(true);
            router.refresh();
        }
    }
    return (<Modal isOpen={isOpen} onClose={onClose}>
        <p className="text-xl text-red-600 mb-12">{err}</p>
        <div className="flex flex-row items-center">
            <label htmlFor="name-input" className="w-48">Name: </label>
            <SimpleTextInput id="name-input" value={groupName} onChange={(e) => {
                const targetValue = (e.target as HTMLInputElement).value;
                setGroupName(targetValue);
                setDataChanged(targetValue !== "");
            }} />
        </div>
        <div className="flex flex-row justify-between mt-6">
            <p className={`underline cursor-pointer ${dataChanged || "text-gray-400"}`} onClick={addGroup}>Add Group</p>
        </div>
    </Modal>)
}
function UserModal({ isOpen, onClose, logout }: { isOpen: boolean, onClose: MouseEventHandler<HTMLButtonElement>, logout: any }) {
    const [dataChanged, setDataChanged] = useState(false);
    const user = TokenStore.getTokenOwner();
    const [username, setUsername] = useState(user);
    const [password, setPassword] = useState("");
    const [err, setErr] = useState("");

    const router = useRouter();

    const updateUser = async () => {
        try {
            let updatedPass = null;
            if (password !== "") updatedPass = password;

            const res = await fetch(API_BASE_URL + "user", {
                method: "PUT",
                headers: {
                    "Content-Type": "application/json",
                    "Authorization": TokenStore.getToken()
                },
                body: JSON.stringify({
                    username,
                    password: updatedPass
                })
            }).then((res) => res.json());

            const err = res.error;
            if (err) {
                setErr(err.msg);
            } else {
                router.replace("/auth");
            }
        } catch (e) {
            console.log(e);
        }
    }

    return (<Modal isOpen={isOpen} onClose={onClose}>
        <p className="text-xl text-red-600 mb-12">{err}</p>
        <div className="flex flex-row items-center">
            <label htmlFor="user-input" className="w-48">Username: </label>
            <SimpleTextInput id="user-input" value={username} onChange={(e) => {
                const targetValue = (e.target as HTMLInputElement).value;
                setUsername(targetValue);
                setDataChanged(targetValue !== user || password !== "");
            }} />
        </div>
        <div className="flex flex-row items-center mt-4">
            <label htmlFor="pass-input" className="w-48">Password: </label>
            <SimpleTextInput id="user-input" value={password} placeHolder="********" onChange={(e) => {
                const targetValue = (e.target as HTMLInputElement).value;
                setPassword(targetValue);
                setDataChanged(targetValue !== "" || username !== user);
            }} />
        </div>
        <div className="flex flex-row justify-between mt-6">
            {dataChanged && <p className="underline cursor-pointer" onClick={updateUser}>Apply Changes</p>}
            <p
                className="text-red-600 cursor-pointer text-2xl"
                onClick={logout}
            >
                Log out
            </p>
        </div>
    </Modal>)
}