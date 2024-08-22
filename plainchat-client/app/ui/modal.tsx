import { MouseEventHandler } from "react";
import {createPortal} from "react-dom";
import { SettingsButton } from "./submitButton";

export default function Modal(
    {isOpen, onClose, children}:
    {isOpen: boolean, onClose: MouseEventHandler<HTMLButtonElement>,
         children: React.ReactNode}
) {

    if (!isOpen) return null;

    return createPortal(
        <div id="dialog-wrapper" 
        className="absolute w-full top-[50%] z-20" 
        >
            <dialog className="w-full lg:w-[640px] text-2xl p-4 translate-y-[-50%] border-2 border-black" open>
                <div id="button-wrapper" className="w-1/3 mb-8">
                    <SettingsButton content="Close" shadow={false} border={true} onClick={onClose} />
                </div>
                <div id="modal-content">
                    {children}
                </div>
            </dialog>
        </div>,
        document.body
    )
}