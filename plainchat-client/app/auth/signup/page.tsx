'use client';

import { ChangeEvent, useLayoutEffect, useState } from "react";
import { useRouter } from "next/navigation";

import InputLabel from "@/app/ui/inputLabel";
import { TextInput, PasswordInput } from "@/app/ui/inputs";
import {SubmitButton} from "@/app/ui/submitButton";
import Form from "@/app/ui/authForm";
import Link from "next/link";
import { API_BASE_URL } from "@/app/lib/globals";

const emptyUserViolation = "eu";
const emptyPassViolation = "ep";
const emptyConfPassViolation = "ecp";

const userLengthViolation = "Username must be at least 5 chars.";
const passLengthViolation = "Password must be at least 6 chars.";
const passMatchViolation = "Passwords do not match";

export default function Home() {

    const router = useRouter();

    let [user, setUser] = useState("");
    let [pass, setPass] = useState("");
    let [confPass, setConfPass] = useState("");

    let [err, setErr] = useState("");
    let [violations, setViolations] = useState<string[]>
        ([emptyUserViolation, emptyPassViolation, emptyConfPassViolation]);
    let [valid, setValid] = useState(false);

    useLayoutEffect(() => {
        setValid(violations.length === 0);
    }, [violations]);

    const signUp = async (e: React.FormEvent) => {
        e.preventDefault();

        try {
            const res = await fetch(API_BASE_URL + "user", {
                method: "POST",
                headers: {
                    "Content-Type": "application/json"
                },
                body: JSON.stringify({
                    username: user,
                    password: pass,
                })
            }).then((r) => r.json());

            const err = res?.error;
            if (err) {
                setErr(err.msg);
            } else {
                router.push("/auth/login");
                router.refresh();
            }
        } catch (err) {
            setErr("Error: " + err);
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

        userInput.length === 0 ? setViolation(emptyUserViolation) : unsetViolation(emptyUserViolation);
        userInput.length < 5 ? setViolation(userLengthViolation) : unsetViolation(userLengthViolation);
    };

    const passInputHandler = (e: ChangeEvent<Element>, confPassV: string) => {
        const passInput = (e.target as HTMLInputElement).value;
        setPass(passInput);

        passInput.length === 0 ? setViolation(emptyPassViolation) : unsetViolation(emptyPassViolation);
        passInput.length < 6 ? setViolation(passLengthViolation) : unsetViolation(passLengthViolation);
        passInput !== confPassV ? setViolation(passMatchViolation) : unsetViolation(passMatchViolation);
    }

    const confPassInputHandler = (e: ChangeEvent<Element>, passV: string) => {
        const confPassInput = (e.target as HTMLInputElement).value;
        setConfPass(confPassInput);

        console.log(passV);
        confPassInput.length === 0 ? setViolation(emptyConfPassViolation) : unsetViolation(emptyConfPassViolation);
        confPassInput !== passV ? setViolation(passMatchViolation) : unsetViolation(passMatchViolation);
    }
    const loginUrl = "/auth/login";

    return (
        <div className="grid place-content-center my-12 lg:my-0 lg:h-screen">
            <p className="text-xl text-red-600 mb-8">{err}</p>
            <div className="text-xl text-[var(--color-1)] mb-8">
                {violations.filter((v) => v !== emptyUserViolation && 
                    v !== emptyPassViolation && v !== emptyConfPassViolation)
                    .map((v) => <p key={v}>{v}</p>)}
            </div> 
            <Form handler={signUp}>
                <div className="flex flex-row flex-wrap">
                    <TextInput id="username-input" value={user} onChange={userInputHandler}/>
                    <InputLabel forId="username-input" content="Username" />
                </div>
                <div className="flex flex-row flex-wrap">
                    <PasswordInput id="password-input" value={pass} onChange={(e) => passInputHandler(e, confPass)} />
                    <InputLabel forId="password-input" content="Password" />
                </div>
                <div className="flex flex-row flex-wrap">
                    <PasswordInput id="confirm-password-input" value={confPass} onChange={(e) => confPassInputHandler(e, pass)} />
                    <InputLabel forId="confirm-password-input" content="Confirm Password" /> 
                </div>
                <div className="flex flex-row mt-4">
                    <div className="w-1/2">
                        <SubmitButton content="Sign Up" enable={valid}/>
                    </div>
                    <div className="grid place-content-center w-1/2">
                        <Link href={loginUrl} className="text-lg underline">Login Instead</Link>
                    </div>
                </div>
            </Form>

        </div>
    );
}
