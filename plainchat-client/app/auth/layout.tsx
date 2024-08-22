export default function AuthLayout({
    children,
}: Readonly<{
    children: React.ReactNode;
}>) {
    return (
        <>
            <div className="bg-white text-black lg:h-screen lg:w-1/3 \ 
        flex flex-col items-center justify-start px-8 py-12 lg:fixed \ 
        shadow-[10px_10px_0px_0px_var(--color-5)] 
    ">
                <h1 className="text-4xl lg:text-6xl">Plain Chat</h1>
                <p className="text-xl lg:text-3xl my-8 lg:my-20 leading-relaxed">
                    A minimalist chat app designed to be simple yet intuitive,
                    with a focus on privacy and speed.
                </p>
            </div>
            <div className="lg:w-2/3 lg:ml-[33.3%]">
                {children}
            </div>
        </>
    );
}
