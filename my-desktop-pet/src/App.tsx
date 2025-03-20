import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Window } from "@tauri-apps/api/window";
// import Draggable from "react-draggable";
import placeHolder from "./assets/placeholder.png";

function App() {
  const [position, setPosition] = useState({ x: 0, y: 0 });
  const [windowSize, setWindowSize] = useState({ width: 400, height: 300 }); // Default size from config
  const [isLoaded, setIsLoaded] = useState(false);
  const windowRef = useRef(null);

  // Initialize window size and listen for resize events
  useEffect(() => {
    const getWindowSize = async () => {
      try {
        const appWindow = Window.getCurrent();
        const factor = await appWindow.scaleFactor() || 1;
        const innerSize = await appWindow.innerSize();
        
        // Convert logical to physical pixels and account for scale factor
        const physicalSize = {
          width: Math.floor(innerSize.width / factor),
          height: Math.floor(innerSize.height / factor)
        };
        
        console.log("Window inner size:", innerSize.width, innerSize.height);
        console.log("Scale factor:", factor);
        console.log("Physical size:", physicalSize.width, physicalSize.height);
        
        setWindowSize(physicalSize);
        setIsLoaded(true);
      } catch (error) {
        console.error("Failed to get window size:", error);
        // Fall back to config defaults
        setIsLoaded(true);
      }
    };

    getWindowSize();

    // Set up resize listener
    const appWindow = Window.getCurrent();
    const cleanup = appWindow.listen("resize", getWindowSize);
    
    // Measure actual DOM element size as a fallback
    if (windowRef.current) {
      const resizeObserver = new ResizeObserver(entries => {
        for (const entry of entries) {
          if (entry.contentRect) {
            console.log("Container actual size:", entry.contentRect.width, entry.contentRect.height);
            if (windowSize.width === 0 || windowSize.height === 0) {
              setWindowSize({
                width: entry.contentRect.width,
                height: entry.contentRect.height
              });
            }
          }
        }
      });
      
      resizeObserver.observe(windowRef.current);
      return () => {
        cleanup.then(unlisten => unlisten());
        resizeObserver.disconnect();
      };
    }
    
    return () => {
      cleanup.then(unlisten => unlisten());
    };
  }, []);

  // Update pet position at regular intervals
  useEffect(() => {
    if (!isLoaded || windowSize.width <= 0 || windowSize.height <= 0) {
      return; // Skip if window size not initialized yet
    }

    const updatePosition = async () => {
      try {
        const [x, y] = await invoke("get_pet_movement", {
          windowWidth: windowSize.width,
          windowHeight: windowSize.height
        });
        
        // Occasional logging to avoid spamming the console
        if (Math.random() < 0.01) {
          console.log(`Pet position: x=${x}, y=${y}, window=${windowSize.width}x${windowSize.height}`);
        }
        
        setPosition({ x, y });
      } catch (error) {
        console.error("Failed to update pet position:", error);
      }
    };

    // Initial position update
    updatePosition();

    // Regular updates
    const interval = setInterval(updatePosition, 50);
    return () => clearInterval(interval);
  }, [windowSize, isLoaded]);

  return (
    <div 
      className="w-full h-full"
      style={{ 
        overflow: "hidden", 
        position: "relative",
        width: "100%",
        height: "100vh"
      }}
    >
      <div
        className="absolute transition-all duration-50"
        style={{
          left: `${position.x}px`,
          top: `${position.y}px`,
          width: "100px",
          height: "100px"
        }}
      >
        <img
          src={placeHolder}
          alt="Pet"
          className="w-full h-full"
          draggable="false"
        />
      </div>
    </div>
  );
}

export default App;
