"use client";

import TokenStore from "../lib/token";

export default function MessageItem(
    {sender, content, msg_type, date}:
    {sender: string, content: string, date: string, msg_type: string}
) {
    const serverEvent = msg_type === "Event";
    const sentBySelf = sender === TokenStore.getTokenOwner();
    const dateV = new Date(date);
    const time = dateV.getFullYear() + '/' + dateV.getMonth() +  '/' + dateV.getDate() + ", " + dateV.getHours() + ':' + dateV.getMinutes();

    return (
        <div className={"px-1 text-lg lg:text-xl xl:text-2xl text-white \
         first:mt-0 " + 
         `${sentBySelf && 'self-end text-right py-2'}` + 
        `${serverEvent && 'justify-self-center self-center text-xl lg:text-2xl xl:text-3xl text-gray-300 py-1'}`}>

            {serverEvent ||
                <span className="text-sm lg:text-md xl:text-lg">{sentBySelf || sender + " - "}{time}</span>
            }
            <p>{content}</p>
        </div>
    );
}