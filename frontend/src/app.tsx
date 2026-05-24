import { Routes, Route } from "react-router";
import Home from "./routes/home";
import About from "./routes/about";
import TestingHome from "./routes/testing_home";
import { useGamepad } from "./hooks/useGamepad";

export default () => {
    const gamepadUtils = useGamepad({ pollRate: 16, stickDeadzone: 0.2 });
    return (
        <Routes>
            <Route path="/" element={<Home gamepadUtils={gamepadUtils} />} />
            <Route path="/about" element={<About gamepadUtils={gamepadUtils} />} />
            <Route path="/testing_home" element={<TestingHome gamepadUtils={gamepadUtils} />} />
        </Routes>
    );
};
