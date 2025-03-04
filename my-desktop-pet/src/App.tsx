import { useState } from "react";
import placeHolder from "./assets/placeholder.png";

function App() {
  const [position, setPosition] = useState({x: 200, y: 200});

  const movePet = () => {
    console.log("Pet clicked! Moving...");
    console.log("Window size:", window.innerWidth, window.innerHeight);
    setPosition({
      x: Math.random() * (window.innerWidth - 100),
      y: Math.random() * (window.innerHeight - 100),
    });
  };

  return (
    <div 
    className="absolute transition-all duration-500" 
    style={{
      top: `${position.y}px`, 
      left: `${position.x}px`,
      width: "100px",
      height: "100px",
      display: "flex",
      alignItems: "center",
      justifyContent: "center",
      pointerEvents: "auto",
      cursor: "pointer",
      userSelect: "none"
    }} 
    onClick={movePet}>
      <img
        src={placeHolder}
        alt="Pet"
        className="w-full h-full"
        style={{pointerEvents: "auto"}} 
      />
    </div>

  );
}

export default App;
