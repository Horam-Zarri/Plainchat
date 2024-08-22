interface InputProps {
    type?: string,
    id: string,
    value: string,
    onChange: React.ChangeEventHandler,
    noLabelSpace?: boolean,
}

export function GeneralInput(
    { type, id, value, onChange, noLabelSpace}: InputProps
) {
    const topPad = noLabelSpace === undefined ? "pt-8" : "pt-2";
    const style = "bg-white text-black \
            focus:bg-black focus:text-white text-2xl lg:text-3xl \
            pb-2 px-2 w-full peer  \
            shadow-[5px_5px_0px_0px_var(--color-3)] "
            + topPad;
            

    return (
        <input
            type={type}
            id={id}
            value={value}
            onChange={onChange}
            className={style}
        />
    );
}

export function TextInput(
    {id, value, onChange, noLabelSpace}:
    {id: string, value: string, onChange: React.ChangeEventHandler, noLabelSpace?: boolean}
) {
    return <GeneralInput type="text" {...{id, value, onChange, noLabelSpace: noLabelSpace}}/>;
}

export function SimpleTextInput(
    {id, value, onChange, placeHolder}:
    {id: string, value: string, onChange: React.ChangeEventHandler, placeHolder?: string}
) {
    const style = "bg-white text-black border-2 border-black w-full py-1 px-2";
    return (
        <input
            type="text"
            id={id}
            value={value}
            onChange={onChange}
            className={style}
            placeholder={placeHolder}
        />
    );
}
export function PasswordInput(
    { id, value, onChange }:
        { id: string, value: string, onChange: React.ChangeEventHandler }
) {
    return <GeneralInput type="password" {...{ id, value, onChange }} />;
}
