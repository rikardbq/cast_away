import { Link } from "react-router";

import "../app.css";
import { useEffect, useMemo, useRef, useState } from "react";
import { useRateLimit } from "../hooks/useRateLimit";
import type { GamepadUtils } from "../hooks/useGamepad";

// const ListItem = ({ id, name, description: _, focused, ...rest }: any) => {
//     return (
//         <li id={id} {...rest}>
//             <div
//                 className={`transition-all duration-150 ease-in-out min-w-40 min-h-40 ${focused ? "scale-125 border-primary rounded-xl shadow-[0px_0px_20px_5px_rgba(0,0,0,0.25)] shadow-primary" : "shadow-md rounded-lg border-transparent"}`}
//                 // style={{
//                 //     width: "500px",
//                 //     height: "500px",
//                 //     border: focused ? "2px solid green" : "",
//                 // }}
//             >
//                 {name}
//             </div>
//         </li>
//     );
// };

const testItems = [
    {
        name: "Paramount+",
        desc: "Halo",
        focused: false,
        vendor: "paramount",
    },
    {
        name: "Netflix",
        desc: "description",
        focused: false,
        vendor: "netflix",
    },
    {
        name: "HBO",
        desc: "description 3",
        focused: false,
        vendor: "hbo",
    },
    {
        name: "PRIME",
        desc: "description 7",
        focused: false,
        vendor: "prime",
    },
    {
        name: "Apple TV",
        desc: "description apple",
        focused: false,
        vendor: "appletv",
    },
    {
        name: "Viaplay",
        desc: "aaaaaaaaaa",
        focused: false,
        vendor: "viaplay",
    },
    {
        name: "chromecast",
        desc: "aaaaaaaaaa",
        focused: false,
        vendor: "chromecast",
    },
];

const keyDownHandler =
    (
        currFocus: number,
        setFocused: Function,
        list: any[],
        padded_list: any[],
    ) =>
    (ev: any) => {
        ev.preventDefault();
        const list_threshold = list.length > 3 ? 3 : list.length;
        if (ev.code === "ArrowLeft" || ev.code === "ArrowUp") {
            const nextFocus = currFocus - 1;
            const willoop = nextFocus < list_threshold;
            setFocused(
                willoop ? padded_list.length - (list_threshold + 1) : nextFocus,
            );
        } else if (ev.code === "ArrowRight" || ev.code === "ArrowDown") {
            const nextFocus = currFocus + 1;
            const willoop =
                nextFocus > padded_list.length - (list_threshold + 1);
            setFocused(willoop ? list_threshold : nextFocus);
        }
        console.log(ev.code);
    };

type Props = {
    gamepadUtils: GamepadUtils;
};

const getKeyFrameAnim = (
    idx: number,
    current_focus: number,
    previous_focus: number,
) => {
    for (let i = 1; i <= 3; i++) {
        let pn = undefined;
        if (idx === current_focus - i) pn = `left-${i}`;
        if (idx === current_focus + i) pn = `right-${i}`;
        if (pn) {
            if (current_focus > previous_focus) return `${pn} r`;
            if (current_focus < previous_focus) return `${pn} l`;
            return pn;
        }
    }

    return "";
};

const willoopStyles = (
    idx: number,
    current_focus: number,
    list: any[],
    padded_list: any[],
) => {
    const list_threshold = list.length > 3 ? 3 : list.length;
    let classes = "";
    if (
        current_focus === list_threshold &&
        idx === padded_list.length - (list_threshold + 1)
    ) {
        classes = "loop l";
    }
    if (
        current_focus === padded_list.length - (list_threshold + 1) &&
        idx === 3
    ) {
        classes = "loop r";
    }

    return classes;
};

export default ({
    gamepadUtils: {
        gamepads,
        isButtonPressed,
        stick: { moveX: _moveX, moveY, deadzone },
    },
}: Props) => {
    const limitRate = useRateLimit();
    const [focusedElem, _setFocusedElem] = useState("back_btn");
    const gamepad = useMemo(() => gamepads[0], [gamepads]);
    const [items, _setItems] = useState([
        ...testItems.slice(-3),
        ...testItems,
        ...testItems.slice(0, 3),
    ]);
    const [previousFocus, setPreviousFocus] = useState({
        idx: Math.floor(items.length / 2),
        vendor: items[Math.floor(items.length / 2)].vendor,
    });
    const [currentFocus, setCurrentFocus] = useState({
        idx: previousFocus.idx,
        vendor: previousFocus.vendor,
    });
    const setFocused = (next_focus: number) => {
        const pfocus =
            next_focus - currentFocus.idx > 1
                ? next_focus + 1
                : next_focus - currentFocus.idx < -1
                  ? next_focus - 1
                  : currentFocus.idx;
        setPreviousFocus({
            idx: pfocus,
            vendor: items[pfocus].vendor,
        });

        setCurrentFocus({ idx: next_focus, vendor: items[next_focus].vendor });
        document.getElementById(`${next_focus}`)?.scrollIntoView({
            behavior: "smooth",
        });
    };
    const [_vendorImage, setVendorImage] = useState<
        (React.JSX.Element | null)[]
    >([]);
    // const setFocused = (idx: number) => {
    //     setPreviousFocus(currentFocus);
    //     setCurrentFocus(idx);
    //     // setItems(
    //     //     items.map((y, i) => ({
    //     //         ...y,
    //     //         focused: idx === i,
    //     //     })),
    //     // );
    //     document.getElementById(`${idx}`)?.scrollIntoView({
    //         behavior: "smooth",
    //     });
    // };
    const navHandler = useRef(
        keyDownHandler(currentFocus.idx, setFocused, testItems, items),
    );

    useEffect(() => {
        setVendorImage(
            items.map((x) =>
                currentFocus.vendor === x.vendor ? (
                    <img
                        src={`../vendor/${x.vendor}_select.webp`}
                        className="vendor-image-select-slide-in"
                    />
                ) : null,
            ),
        );
        return () => {
            window.removeEventListener("keydown", navHandler.current);
        };
    }, []);

    useEffect(() => {
        window.removeEventListener("keydown", navHandler.current);
        navHandler.current = keyDownHandler(
            currentFocus.idx,
            setFocused,
            testItems,
            items,
        );
        window.addEventListener("keydown", navHandler.current);
        setVendorImage(
            items.map((x) =>
                currentFocus.vendor === x.vendor ? (
                    <img
                        src={`../vendor/${x.vendor}_select.webp`}
                        className="vendor-image-select-slide-in"
                    />
                ) : null,
            ),
        );
    }, [currentFocus]);

    useEffect(() => {
        if (gamepad) {
            if (
                isButtonPressed(gamepad, "XBOX.DPAD_UP") ||
                moveY(gamepad, "LEFT_STICK") < 0 - deadzone
            ) {
                const nextFocus = currentFocus.idx - 1;
                const willoop = nextFocus < 0;
                limitRate(
                    () => setFocused(willoop ? items.length - 1 : nextFocus),
                    250,
                );
            } else if (
                isButtonPressed(gamepad, "XBOX.DPAD_DOWN") ||
                moveY(gamepad, "LEFT_STICK") > 0 + deadzone
            ) {
                const nextFocus = currentFocus.idx + 1;
                const willoop = nextFocus > items.length - 1;
                limitRate(() => setFocused(willoop ? 0 : nextFocus), 250);
            }
        }
    });

    // prevent list from getting bounds issues
    // split list
    // floor value
    // slice from index floor val
    // take original list and slice on index 0 up to its split floor val
    // combine with first list slice
    /*
    > let list = [1,2,3,4,5];
    > let list_s1 = list.slice(Math.floor(list.length / 2));
    > list_s1
    [ 3, 4, 5 ]
    > let list_s2 = list.slice(0, Math.floor(list.length / 2))
    > list_s2
    [ 1, 2 ]
    > [...list_s1, ...list_s2]
    [ 3, 4, 5, 1, 2 ]
    >
    */

    return (
        <div
            className="h-lvh w-lvw bg-no-repeat bg-cover overflow-hidden"
            style={{
                background: "url(../wp.jpg)",
            }}
        >
            <div
                className="h-full w-full absolute bg-no-repeat left-0 top-0"
                style={{
                    backgroundImage:
                        "linear-gradient(to right, black 25%, transparent 100%)",
                }}
            />
            <div className="x-items h-screen content-center">
                {items.map((x, idx) => {
                    let fontFamily = "FetteUnzFraktur";
                    let textColor = "#FF4444";
                    if (x.name === "chromecast") {
                        fontFamily = "Cyberpunk";
                        textColor = "#ffff44";
                    }
                    return (
                        <div key={x.vendor}>
                            <div
                                className={`text-3xl left-1/5 fixed w-max ${getKeyFrameAnim(idx, currentFocus.idx, previousFocus.idx)}${idx === currentFocus.idx ? ` selected${currentFocus.idx > previousFocus.idx ? " r" : currentFocus.idx < previousFocus.idx ? " l" : ""}` : ""} ${willoopStyles(idx, currentFocus.idx, testItems, items)}`}
                                style={{
                                    fontFamily,
                                    color: textColor,
                                }}
                            >
                                <div
                                    className={
                                        currentFocus.vendor === "chromecast" && 
                                        idx === currentFocus.idx
                                            ? "glitch-effect-periodic"
                                            : ""
                                    }
                                >
                                    {x.name.toLowerCase()}
                                </div>
                            </div>
                            <img
                                src={`../vendor/${x.vendor}_select.webp`}
                                className={`h-3/4 right-1/12 fixed self-center place-content-center vendor-image${currentFocus.idx === idx ? " vendor-image-select-slide-in" : ""}`}
                            />
                        </div>
                    );
                })}
            </div>
            <div>
                <Link
                    style={{
                        fontFamily: "FetteUnzFraktur",
                        color: "#FF4444",
                        fontSize: "2rem",
                        lineHeight: "normal",
                        borderRadius: "10px",
                        padding: "4px 16px",
                        border:
                            focusedElem === "back_btn"
                                ? "2px solid #dddddd"
                                : "none",
                    }}
                    className="absolute bottom-5 left-8 hover-3d"
                    to="/"
                >
                    back
                </Link>
                {/* <div className="btn btn-error">
                </div> */}
            </div>
        </div>
    );
};
