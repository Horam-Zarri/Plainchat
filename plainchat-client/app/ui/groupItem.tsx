import Link from "next/link";
import { MouseEventHandler } from "react";

export default function GroupItem(
    {id, name, current}: 
    {id: string, name: string, current: string}
) {
    const groupPath = "/chat/" + id;
    const isSelected = id === current;
    const style = `text-2xl border-b-2 border-black last:border-b-0 py-2 ${isSelected && "text-white bg-black"}`;

    return(
        <Link 
            className={style}
            href={groupPath}
        >
            {name}
        </Link>
    )
}