export default function InputLabel(
    {forId, content}: 
    {forId: string, content: string}
) {
    return(
        <label htmlFor={forId} className=
        "absolute transform translate-x-2 translate-y-1 \
        text-black peer-focus:text-white text-xl">
            {content}
        </label>
    );
}