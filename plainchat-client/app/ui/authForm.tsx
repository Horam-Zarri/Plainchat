import { FormEventHandler } from "react";

export default function AuthForm(
    {children, handler}: 
    {children: React.ReactNode,handler: FormEventHandler<HTMLFormElement>}
) {
    return <form onSubmit={handler} className="flex flex-col gap-8 w-[75vw] lg:w-[480px]">
        {children}
    </form>;
}