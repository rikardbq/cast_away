import { Link } from "react-router";

import "../app.css";
import { useEffect, useMemo, useRef, useState } from "react";
import { useRateLimit } from "../hooks/useRateLimit";
import type { GamepadUtils } from "../hooks/useGamepad";

const ListItem = ({ id, name, description: _, focused, ...rest }: any) => {
    return (
        <li id={id} {...rest}>
            <div
            className={`transition-all duration-150 ease-in-out min-w-40 min-h-40 ${focused ? "scale-125 border-primary rounded-xl shadow-[0px_0px_20px_5px_rgba(0,0,0,0.25)] shadow-primary" : "shadow-md rounded-lg border-transparent"}`}
                // style={{
                //     width: "500px",
                //     height: "500px",
                //     border: focused ? "2px solid green" : "",
                // }}
            >
                {name}
            </div>
        </li>
    );
};

const testItems = [
    {
        name: "Netflix",
        desc: "description",
        focused: true,
    },
    {
        name: "N chill",
        desc: "description 2 chill 2",
        focused: false,
    },
    {
        name: "HBO",
        desc: "description 3",
        focused: false,
    },
    {
        name: "PRIME",
        desc: "description 7",
        focused: false,
    },
    {
        name: "Apple TV",
        desc: "description apple",
        focused: false,
    },
];

const keyDownHandler =
    (currFocus: number, setFocused: Function, items: any[]) => (ev: any) => {
        ev.preventDefault();

        console.log(currFocus);

        if (ev.code === "ArrowLeft") {
            if (currFocus === 0) return;
            else setFocused(currFocus - 1);
        } else if (ev.code === "ArrowRight") {
            if (currFocus === items.length - 1) return;
            else setFocused(currFocus + 1);
        }
        console.log(ev.code);
    };

type Props = {
    gamepadUtils: GamepadUtils;
};

export default ({ gamepadUtils: { gamepads, isButtonPressed, stick: { moveX, deadzone } } }: Props) => {
    const limitRate = useRateLimit();
    const gamepad = useMemo(() => gamepads[0], [gamepads]);
    const [items, setItems] = useState(testItems);
    const [currentFocus, setCurrentFocus] = useState(0);
    const setFocused = (idx: number) => {
        setCurrentFocus(idx);
        setItems(
            items.map((y, i) => ({
                ...y,
                focused: idx === i,
            })),
        );
        document.getElementById(`${idx}`)?.scrollIntoView({
            behavior: "smooth",
        });
    };
    const navHandler = useRef(keyDownHandler(currentFocus, setFocused, items));

    useEffect(() => {
        return () => {
            window.removeEventListener("keydown", navHandler.current);
        };
    }, []);

    useEffect(() => {
        window.removeEventListener("keydown", navHandler.current);
        navHandler.current = keyDownHandler(currentFocus, setFocused, items);
        window.addEventListener("keydown", navHandler.current);
    }, [currentFocus]);

    if (gamepad) {
        if (isButtonPressed(gamepad, "XBOX.DPAD_LEFT") || moveX(gamepad, "LEFT_STICK") < 0 - deadzone) {
            if (currentFocus !== 0) {
                limitRate(() => setFocused(currentFocus - 1), 100);
            }
        } else if (isButtonPressed(gamepad, "XBOX.DPAD_RIGHT") || moveX(gamepad, "LEFT_STICK") > 0 + deadzone) {
            if (currentFocus !== items.length - 1) {
                limitRate(() => setFocused(currentFocus + 1), 100);
            }
        }
    }

    return (
        <>
            <div>
                <ul
                    style={{
                        display: "flex",
                        flexDirection: gamepad?.buttons[0]?.pressed
                            ? "column"
                            : "row",
                        gap: "5px",
                    }}
                >
                    {items.map((x, idx) => (
                        <ListItem
                            onClick={() => {
                                setFocused(idx);
                            }}
                            id={idx}
                            key={`${x.name}:${idx}`}
                            name={x.name}
                            focused={x.focused}
                        />
                    ))}
                </ul>
            </div>
            <h1>TEST</h1>
            <Link to="/">Home</Link>
        </>
    );
};
