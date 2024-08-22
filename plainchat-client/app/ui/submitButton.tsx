import { MouseEventHandler } from "react";

export function SubmitButton({content, enable}: {content: string, enable?:boolean}) {

    return (
        <GenericButton content={content} color={2} type="submit" border={false} enable={enable}/>
    );
}

export function SettingsButton(
    {content, shadow, border, color, onClick} :
    {content: string, border: boolean, shadow?: boolean, 
        color?: number, onClick?: MouseEventHandler<HTMLButtonElement>} 
) {
    return (
        <GenericButton content={content} color={shadow ? (color || 1) : undefined} type="button" border={border} onClick={onClick}/>
    )
}

function GenericButton(
    {content, color, type, border, onClick, enable}: 
    {content: string,  color?: number, type?: string, enable?: boolean,
        border :boolean, onClick?: MouseEventHandler<HTMLButtonElement>}
) {

    type ButtonTypes = "submit" | "reset" | "button" | undefined;

    // Either tailwind or nextjs have problem with runtime css 
    // variable deduction so we need to do this...
    const UiColors = new Map([
        [1, "shadow-[5px_5px_0px_0px_#fef200]"], 
        [2, "shadow-[5px_5px_0px_0px_#ff0078]"],
        [3, "shadow-[5px_5px_0px_0px_#0201ff]"],
        [4, "shadow-[5px_5px_0px_0px_#dfdfdf]"],
        [5, "shadow-[5px_5px_0px_0px_#37ff2d]"]
    ]);

    let v = UiColors.get(color as number)!;

    const style = `text-black py-2 lg:py-3 px-1 text-xl lg:text-2xl ${v} w-full 
        ${border && "border-2 border-black"} 
        ${enable === false ? "bg-gray-500 hover:bg-gray-500" : "bg-white hover:bg-[var(--color-4)]"}`;

    return (
        <button 
        onClick={onClick}
        type={type as ButtonTypes}
        className={style}
        disabled={enable === false}
        >
        {content}
        </button>
    )
}

