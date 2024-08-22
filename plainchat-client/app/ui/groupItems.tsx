import { useState } from "react";
import GroupItem from "./groupItem";

type GroupStat = {
    id: string,
    name: string,
}
export default function GroupItems({ gstats, selectedChatId}: { gstats: GroupStat[], selectedChatId: string}) {
    return(
        <>
            {gstats.map((g) => {
                let id = g.id.toString();
                return <GroupItem key={id} current={selectedChatId} id={id} name={g.name}/>;
            })}
        </>
    );
}