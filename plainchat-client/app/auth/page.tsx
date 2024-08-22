import Link from "next/link";

export default function Home() {
    const signupUrl = "/auth/signup";
    const loginUrl = "/auth/login";

    return (
        <div className="flex flex-col justify-center ml-24 h-screen text-4xl">
            <h1 className="mb-12">Lets get started!</h1>
            <h2>Either 
            <Link 
                href={signupUrl}
                className="underline px-4"
            >
                Sign Up
            </Link>
            Or 
            <Link
                href={loginUrl}
                className="underline px-4"
            >
                Login
            </Link>
            </h2>
        </div>
    )
}