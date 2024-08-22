"use client";

import { Dispatch, MouseEventHandler, SetStateAction, useContext, useEffect, useRef, useState } from "react";
import TokenStore from "@/app/lib/token";
import MessageItem from "@/app/ui/messageItem";
import { SelectedChatContext, SideMenuContext, WebSocketContext } from "../contexts";
import { SimpleTextInput, TextInput } from "@/app/ui/inputs";
import { SettingsButton, SubmitButton } from "@/app/ui/submitButton";
import Modal from "@/app/ui/modal";
import { useRouter } from "next/navigation";
import { revalidatePath, revalidateTag } from "next/cache";
import { API_BASE_URL } from "@/app/lib/globals";


type MsgType = "Normal" | "Event";
type Role = "User" | "Admin";

type Message = {
    id?: string,
    sender?: string,
    msg_type: MsgType,
    content: string,
    date: string,
};

type Member = {
    username: string,
    role: Role,
    presence: boolean,
}

function messageElems(msgs: Message[]): React.JSX.Element[] {
    return msgs.toReversed().map((msg, idx) => {
        const date = new Date(msg.date);
        const time = date.getHours() + ":" + date.getMinutes();
        const sender = (msg.sender === null && msg.msg_type !== "Event") ? "DELETED ACCOUNT" : msg.sender!;
        return <MessageItem key={msg.id ?? "emsg-" + idx} sender={sender}
            content={msg.content} date={msg.date} msg_type={msg.msg_type}/>
    });
}

function onlineMemCount(mems: Member[]): number {
    return mems.filter((m) => m.presence).length;
}

export default function Home({ params }: { params: { group_id: string } }) {

    // General States
    const socket = useContext(WebSocketContext);
    const setShowMenu = useContext(SideMenuContext);
    const setSelectedChat = useContext(SelectedChatContext); 

    const onceJoin = useRef(false);

    const [isGroupLoading, setIsGroupLoading] = useState(true);
    const [isMembersLoading, setIsMembersLoading] = useState(true);
    const [isSocketLoading, setIsSocketLoading] = useState(true);

    const [messageList, setMessageList] = useState<Message[]>();
    const [memberList, setMemberList] = useState<Member[]>();
    const [isGroupModalOpen, setIsGroupModalOpen] = useState(false);
    // Component States
    const [messageText, setMessageText] = useState("");
    const [usersTyping, setUsersTyping] = useState<string[]>([]);
    const [isTypeStarted, setIsTypeStarted] = useState(false);


    useEffect(() => {
        const timeOutId = setTimeout(() => {
            if (isTypeStarted) {
                setIsTypeStarted(false);
                socket.emit("type_stop", TokenStore.getTokenOwner());
            }
        }, 5000);

        return () => clearTimeout(timeOutId);
    });

    const sendMsg = async (e: React.FormEvent) => {
        e.preventDefault();
        console.log(messageText);
        socket.emit("message", messageText);
        socket.emit("type_start", TokenStore.getTokenOwner());
        setMessageText("");
    };

    useEffect(() => {
        if (onceJoin.current) {
            return;
        }
        socket.emit("join", params.group_id);

        setIsSocketLoading(false);
        onceJoin.current = false;
    }, []);

    useEffect(() => {
        let done = false;
        socket.on("message", (msg) => {
            if (!done) {
                console.log("MSG");
                setMessageList((prev) => {
                    return [...prev ?? [], msg];
                });
            }
        });
        socket.on("add_user", (uadd) => {
            if (!done) {
                let [added, adder, status] = uadd.split(",");
                const presense = status === "true";
                const addMsg = added + " was added by " + adder + '.';
                const eventMsg: Message = {
                    msg_type: "Event",
                    content: addMsg,
                    date: ""
                };
                setMessageList((prev) => {
                    return [...prev ?? [], eventMsg];
                });
                const newMem: Member = {
                    username: added as string,
                    role: "User",
                    presence: presense
                };
                setMemberList((prev) => {
                    return [...prev ?? [], newMem]
                })
            }
        });
        socket.on("leave", (user) => {
            if (!done) {
                let leaveMsg = user + " left.";
                let eventMsg: Message = {
                    msg_type: "Event",
                    content: leaveMsg,
                    date: ""
                };
                setMessageList((prev) => {
                    return [...prev ?? [], eventMsg];
                });
            }
        });
        socket.on("type_start", (user) => {
            if (!done && user !== TokenStore.getTokenOwner()) {
                setUsersTyping([...usersTyping, user]);
            }
        });
        socket.on("type_stop", (user) => {
            if (!done) {
                const typers = usersTyping.filter((u: string) => u !== user);
                setUsersTyping(typers);
            }
        });
        socket.on("u-online", (user: string) => {
            if (!done) {
                setMemberList((prev) => prev?.map((u) => {
                    if (u.username === user) u.presence = true;
                    return u;
                }))
            }
        });
        socket.on("u-offline", (user: string) => {
            if (!done) {
                setMemberList((prev) => prev?.map((u) => {
                    if (u.username === user) u.presence = false;
                    return u;
                }))
            }
        });
        socket.on("kick", (udel: string) => {
            if (!done) {
                let [rem_user, kicker] = udel.split(',');
                const kickMsg = rem_user + " was kicked out by " + kicker + ".";
                const eventMsg: Message = {
                    msg_type: "Event",
                    content: kickMsg,
                    date: ""
                };
                setMemberList((prev) => {
                    return prev?.filter((m) => m.username !== rem_user);
                });
                setMessageList((prev) => {
                    return [...(prev ?? []), eventMsg];
                })
            }
        })
        return () => { done = true }
    }, []);

    useEffect(() => {
        const groupMessagesUrl = API_BASE_URL + "group/"
            + params.group_id + "/messages";
        fetch(groupMessagesUrl, {
            method: "GET",
            headers: {
                "Authorization": TokenStore.getToken(),
            }
        }).then((res) => res.json())
            .then((data) => {
                const err = data.error;
                if (err) {
                    console.log(err.msg);
                } else {
                    const datas = data as Message[];
                    setSelectedChat(params.group_id);
                    setMessageList(datas);
                    setIsGroupLoading(false);
                }
            })
            .catch((e) => {
                console.log(e);
                setIsGroupLoading(false);
            })
    }, []);

    useEffect(() => {
        const groupMembersUrl = API_BASE_URL + "group/"
            + params.group_id + "/members";
        fetch(groupMembersUrl, {
            method: "GET",
            headers: {
                "Authorization": TokenStore.getToken(),
            }
        }).then((res) => res.json())
            .then((data) => {
                const err = data.error;
                if (err) {
                    console.log(err.msg);
                } else {
                    const datas = data as Member[];
                    setMemberList(datas);

                    setIsMembersLoading(false);
                    console.log(datas);
                }
            })
            .catch((e) => {
                console.log(e);
                setIsMembersLoading(false);
            })
    }, []);

    if (isGroupLoading || isSocketLoading) {
        return (
            <div className="text-4xl text-white grid place-content-center h-screen">
                <h1>Loading...</h1>
            </div>
        );
    }

    let typersText = "";
    if (usersTyping.length > 0) {
        typersText = usersTyping.join(", ") +
            (usersTyping.length === 1 ? " is typing..." : " are typing...");
    }
    const documentStyle = `${isGroupModalOpen && "opacity-25 pointer-events-none"}`;

    return (
        <div className={documentStyle}>
            <div id="main-actions" className="h-[12.5vh] z-10 flex flex-row justify-between content-center px-4 lg:px-12 pt-2 ">
                <button className="text-4xl lg:hidden" onClick={(e) => { setShowMenu(true); }}>&#9776;</button>
                <p className="self-center text-md lg:text-2xl px-4 ">{memberList?.length} members, {onlineMemCount(memberList ?? [])} online</p>
                <div className="w-[200px] self-center justify-self-end">
                    <SettingsButton content="Group Settings" shadow={true} border={false} onClick={() => setIsGroupModalOpen(true)} />
                    <GroupModal isOpen={isGroupModalOpen} onClose={() => setIsGroupModalOpen(false)}
                        setIsOpen={setIsGroupModalOpen} memberList={memberList!} />
                </div>
            </div>
            <div id="messages-list" className="w-full h-[72.5vh] py-4 px-6 overflow-y-scroll flex flex-col-reverse gap-y-4">
                {messageElems(messageList ?? [])}
            </div>
            <div className="h-[12.5vh] z-10">
                <form className="flex flex-row w-full items-center justify-between gap-x-8 lg:gap-x-24 px-8 py-4 mt-[2.5vh]" onSubmit={sendMsg}>
                    <span className="absolute mb-24 z-50 text-[var(--color-4)] text-lg lg:text-xl xl:text-2xl">
                        {typersText}
                    </span>
                    <TextInput id="input-whatever" value={messageText}
                        onChange={(e) => {
                            setMessageText((e.target as HTMLInputElement).value);
                            if (!isTypeStarted) {
                                socket.emit("type_start", TokenStore.getTokenOwner());
                                setIsTypeStarted(true);
                            }
                        }} noLabelSpace={true} />
                    <div className="w-[120px] lg:w-[180px]">
                        <SubmitButton content="Send" />
                    </div>
                </form>
            </div>
        </div>
    )
}

function GroupModal(
    { isOpen, setIsOpen, onClose, memberList }:
        {
            isOpen: boolean,
            setIsOpen: Dispatch<SetStateAction<boolean>>,
            onClose: MouseEventHandler<HTMLButtonElement>,
            memberList: Member[],
        }
) {

    const [addUser, setAddUser] = useState("");
    const [dataChanged, setDataChanged] = useState(false);
    const socket = useContext(WebSocketContext);
    const router = useRouter();

    const leaveGroup = async () => {
        socket.emit("leave");
        router.replace("/chat");
        router.refresh();
    }

    const addMember = async () => {
        socket.emit("add_user", addUser);
        setIsOpen(false);
    }

    return (<Modal {...{ isOpen, onClose }}>
        <GroupModalMemList memberList={memberList} />
        <div className="flex flex-row items-center mb-6 pt-4 border-t-2 border-[var(--color-4)]">
            <label htmlFor="name-input" className="w-48">Add User: </label>
            <SimpleTextInput id="name-input" value={addUser} onChange={(e) => {
                const targetValue = (e.target as HTMLInputElement).value;
                setAddUser(targetValue);
                setDataChanged(targetValue !== "");
            }} />
        </div>
        <div className="flex flex-row justify-between">
            {dataChanged && <p className="underline cursor-pointer" onClick={addMember}>Add Member</p>}
            <p className="text-red-500 cursor-pointer" onClick={leaveGroup}>Leave group</p>
        </div>
    </Modal>);
}


function GroupModalMemList({ memberList }: { memberList: Member[] }) {
    const admins = memberList.filter((m) => m.role === "Admin");
    const members = memberList.filter((m) => m.role === "User");

    const currUser = TokenStore.getTokenOwner();
    const isUserAdmin = admins.find((a) => a.username === currUser);

    const socket = useContext(WebSocketContext);

    const remMember = async (user: string) => {
        socket.emit("kick", user);
    }

    const presense = (isOnline: boolean) =>
        <p className={`${isOnline ? "text-[var(--color-5)]" : "text-[var(--color-2)]"}`}>{isOnline ? "ONLINE" : "OFFLINE"}</p>

    const userListWrapperStyle = "flex flex-col mx-4 lg:mx-8 max-h-[15vh] overflow-y-scroll";
    const userListRowStyle = "flex flex-row justify-between w-[160px] sm:w-[240px]";

    return (
        <div className="text-lg sm:text-xl lg:text-2xl mb-6">
            <h3 className="mb-2">Admins: </h3>
            <div className={userListWrapperStyle}>
                {admins.map((m) => {
                    return <div key={m.username} className={userListRowStyle}>
                        <p>{m.username}</p>
                        {presense(m.presence)}
                    </div>;
                })}
            </div>
            <h3 className="mt-4 mb-2">Members: </h3>
            <div className={userListWrapperStyle}>
                {members.map((m) => {
                    return (
                        <div key={m.username} className="flex flex-row justify-between items-center">
                            <div className={userListRowStyle}>
                                <p>{m.username}</p>
                                {presense(m.presence)}
                            </div>
                            {isUserAdmin &&
                                <div className="flex flex-row gap-x-4 text-base sm:text-lg lg:text-2xl">
                                    <p className="underline cursor-pointer">Promote</p>
                                    <p className="underline cursor-pointer" onClick={() => { remMember(m.username); }}>Remove</p>
                                </div>
                            }
                        </div>
                    );
                })}
            </div>
        </div>
    );
}