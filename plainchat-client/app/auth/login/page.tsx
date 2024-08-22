'use client';

import { ChangeEvent, useLayoutEffect, useState } from "react";
import TokenStore from "@/app/lib/token";
import Form from "@/app/ui/authForm";
import { PasswordInput, TextInput } from "@/app/ui/inputs";
import InputLabel from "@/app/ui/inputLabel";
import { SubmitButton } from "@/app/ui/submitButton";
import Link from "next/link";
import { useRouter } from "next/navigation";
import { API_BASE_URL } from "@/app/lib/globals";

const userLengthViolation = "ue";
const passLengthViolation = "pe";

export default function Home() {

    let [user, setUser] = useState("");
    let [pass, setPass] = useState("");
    let [err, setErr] = useState("");

    let [valid, setValid] = useState(false);
    let [violations, setViolations] = useState<string[]>
        ([userLengthViolation, passLengthViolation]);

    useLayoutEffect(() => {
        setValid(violations.length === 0);
    }, [violations]);

    const router = useRouter();

    const login = async (e: React.FormEvent) => {

        e.preventDefault();
        try {
            const res = await fetch(API_BASE_URL + "user/auth", {
                method: "POST",
                headers: {
                    "Content-Type": "application/json"
                },
                body: JSON.stringify({
                    username: user,
                    password: pass,
                })
            }).then((r) => r.json());
            
            const err = res.error;
            if (err) {
                setErr(err.msg);
            } else {
                TokenStore.setToken(user, res.token);
                router.push("/chat");
                router.refresh();
            }
        } catch (error) {
            if (error instanceof Error) {
                setErr(error.message);
            }
        }

    }

    const setViolation = (v: string) => setViolations((prev) => {
        if (prev.find((vio) => vio === v)) {
            return prev;
        } else {
            return [...prev, v];
        }
    });
    const unsetViolation = (v: string) => setViolations((prev) => prev.filter((vio: string)  => vio !== v));

    const userInputHandler = (e: ChangeEvent<Element>) => {
        const userInput = (e.target as HTMLInputElement).value;
        setUser(userInput);
        userInput.length < 5 ? setViolation(userLengthViolation) : unsetViolation(userLengthViolation);
    };

    const passInputHandler = (e: ChangeEvent<Element>) => {
        const passInput = (e.target as HTMLInputElement).value;
        setPass(passInput);
        passInput.length < 6 ? setViolation(passLengthViolation) : unsetViolation(passLengthViolation);
    }
    const signupUrl = "/auth/signup";

    return (
        <div className="grid place-content-center my-12 lg:my-0 lg:h-screen">
            <p className="text-xl text-red-600 mb-12">{err}</p>
            <Form handler={login}>
                <div className="flex flex-row flex-wrap">
                    <TextInput id="username-input" value={user} onChange={userInputHandler} />
                    <InputLabel forId="username-input" content="Username" />
                </div>
                <div className="flex flex-row flex-wrap">
                    <PasswordInput id="password-input" value={pass} onChange={passInputHandler} />
                    <InputLabel forId="password-input" content="Password" />
                </div>
                <div className="flex flex-row mt-4">
                    <div className="w-1/2">
                        <SubmitButton content="Login" enable={valid}/>
                    </div>
                    <div className="grid place-content-center w-1/2">
                        <Link href={signupUrl} className="text-lg underline">SignUp Instead</Link>
                    </div>
                </div>
            </Form>

        </div>
    );
}