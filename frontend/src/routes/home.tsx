import { Link } from "react-router";

import reactLogo from "../assets/react.svg";
import viteLogo from "/vite.svg";
import "../app.css";
import type { GamepadUtils } from "../hooks/useGamepad";
import { ipc } from "../util/IPC";

type HomeProps = {
    gamepadUtils: GamepadUtils;
};

export default ({ gamepadUtils: { gamepads } }: HomeProps) => {
    return (
        <div>
            <p>{`pressed : ${gamepads[0]?.buttons[0].pressed}`}</p>
            <div>
                <a href="https://vite.dev" target="_blank">
                    <img src={viteLogo} className="logo" alt="Vite logo" />
                </a>
                <a href="https://react.dev" target="_blank">
                    <img
                        src={reactLogo}
                        className="logo react"
                        alt="React logo"
                    />
                </a>
            </div>
            <button
                className="theme-controller btn btn-success"
                value="vampire"
            >
                Button
            </button>
            <button className="theme-controller btn btn-secondary" value="mys">
                Two
            </button>
            <button className="btn btn-error" onClick={async () => {
                const list = await ipc.call("get_windows");
                console.log(list);
                
            }}>TESTING THIS IPC</button>
            <h1>TTTTTTTTTT</h1>
            <Link to="/about">About</Link>
            <Link to="/testing_home">testing home</Link>

            <div className="dropdown mb-72">
                <div tabIndex={0} role="button" className="btn m-1">
                    Theme
                    <svg
                        width="12px"
                        height="12px"
                        className="inline-block h-2 w-2 fill-current opacity-60"
                        xmlns="http://www.w3.org/2000/svg"
                        viewBox="0 0 2048 2048"
                    >
                        <path d="M1799 349l242 241-1017 1017L7 590l242-241 775 775 775-775z"></path>
                    </svg>
                </div>
                <ul
                    tabIndex={-1}
                    className="dropdown-content bg-base-300 rounded-box z-1 w-52 p-2 shadow-2xl"
                >
                    <li>
                        <input
                            type="radio"
                            name="theme-dropdown"
                            className="theme-controller w-full btn btn-sm btn-block btn-ghost justify-start"
                            aria-label="Light"
                            value="light"
                        />
                    </li>
                    <li>
                        <input
                            type="radio"
                            name="theme-dropdown"
                            className="theme-controller w-full btn btn-sm btn-block btn-ghost justify-start"
                            aria-label="Dracula"
                            value="dracula"
                        />
                    </li>
                    <li>
                        <input
                            type="radio"
                            name="theme-dropdown"
                            className="theme-controller w-full btn btn-sm btn-block btn-ghost justify-start"
                            aria-label="Vampire"
                            value="vampire"
                        />
                    </li>
                    <li>
                        <input
                            type="radio"
                            name="theme-dropdown"
                            className="theme-controller w-full btn btn-sm btn-block btn-ghost justify-start"
                            aria-label="Mys"
                            value="mys"
                        />
                    </li>
                </ul>
            </div>
        </div>
    );
};
