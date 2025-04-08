import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Window } from "@tauri-apps/api/window";
// Import your single sprite sheet
import spriteSheet from "./assets/Fox Sprite Sheet.png";

function App() {
  const [position, setPosition] = useState({ x: 0, y: 0 });
  const [windowSize, setWindowSize] = useState({ width: 400, height: 300 });
  const [isLoaded, setIsLoaded] = useState(false);
  const [animationState, setAnimationState] = useState("idle-right");
  const [frameIndex, setFrameIndex] = useState(0);
  const windowRef = useRef(null);
  
  // Frame size constants - adjust these to match your actual sprite size
  const FRAME_WIDTH = 8;
  const FRAME_HEIGHT = 8;
  const DISPLAY_SCALE = 3; // Increase this to make the pet larger
  
  // Define animation sequences from the sprite sheet
  // For simplicity, we'll define all animations as if facing right
  // and use CSS transform to flip for left-facing animations
  const animations = {
    "idle": {
      frames: [
        [0, 0],       // x, y coordinates of each frame
        [32, 0],
        [64, 0],
        [96, 0],
        [128, 0],
      ],
      frameDuration: 200,
    },
    "run": {
      frames: [
        [0, 32],
        [32, 32],
        [64, 32],
        [96, 32],
      ],
      frameDuration: 100,
    },
    "jump": {
      frames: [
        [0, 64],
        [32, 64],
      ],
      frameDuration: 150,
    },
    "fall": {
      frames: [
        [128, 64],
      ],
      frameDuration: 150,
    },
  };

  // Initialize window size and listen for resize events
  useEffect(() => {
    const getWindowSize = async () => {
      try {
        const appWindow = Window.getCurrent();
        const factor = await appWindow.scaleFactor() || 1;
        const innerSize = await appWindow.innerSize();
        
        const physicalSize = {
          width: Math.floor(innerSize.width / factor),
          height: Math.floor(innerSize.height / factor)
        };
        
        setWindowSize(physicalSize);
        setIsLoaded(true);
      } catch (error) {
        console.error("Failed to get window size:", error);
        setIsLoaded(true);
      }
    };

    getWindowSize();

    const appWindow = Window.getCurrent();
    const cleanup = appWindow.listen("resize", getWindowSize);
    
    if (windowRef.current) {
      const resizeObserver = new ResizeObserver(entries => {
        for (const entry of entries) {
          if (entry.contentRect) {
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

  // Animation frame timing
  useEffect(() => {
    if (!isLoaded) return;
    
    // Extract the base animation name without direction
    const baseAnimation = animationState.split('-')[0];
    const config = animations[baseAnimation];
    
    if (!config) return;
    
    const frameTimer = setTimeout(() => {
      setFrameIndex(prev => (prev + 1) % config.frames.length);
    }, config.frameDuration);
    
    return () => clearTimeout(frameTimer);
  }, [frameIndex, animationState, isLoaded]);

  // Update pet position at regular intervals
  useEffect(() => {
    if (!isLoaded || windowSize.width <= 0 || windowSize.height <= 0) {
      return;
    }

    const updatePosition = async () => {
      try {
        const [x, y, state] = await invoke("get_pet_movement", {
          windowWidth: windowSize.width,
          windowHeight: windowSize.height
        });
        
        setPosition({ x, y });
        
        // Only change the animation state if it's different
        if (state !== animationState) {
          setAnimationState(state);
          setFrameIndex(0); // Reset frame index when changing animation
        }
      } catch (error) {
        console.error("Failed to update pet position:", error);
      }
    };

    updatePosition();
    const interval = setInterval(updatePosition, 50);
    return () => clearInterval(interval);
  }, [windowSize, isLoaded, animationState]);

  // Get the current frame from the animation sequence
  const getCurrentFrame = () => {
    // Extract the base animation name without direction
    const baseAnimation = animationState.split('-')[0];
    const animation = animations[baseAnimation];
    
    if (!animation) return [0, 0]; // Default frame
    
    return animation.frames[frameIndex];
  };

  // Calculate sprite style based on current frame
  const getSpriteStyle = () => {
    const [x, y] = getCurrentFrame();
    const isFlipped = animationState.endsWith('-left');
    
    return {
      width: `${FRAME_WIDTH * DISPLAY_SCALE}px`,
      height: `${FRAME_HEIGHT * DISPLAY_SCALE}px`,
      backgroundImage: `url(${spriteSheet})`,
      backgroundPosition: `-${x}px -${y}px`,
      backgroundSize: `${spriteSheet.width * DISPLAY_SCALE}px ${spriteSheet.height * DISPLAY_SCALE}px`,
      backgroundRepeat: "no-repeat",
      transform: isFlipped ? 'scaleX(-1)' : 'scaleX(1)',
      transformOrigin: 'center',
      imageRendering: 'pixelated', // For crisp pixel art scaling
    };
  };

  return (
    <div 
      className="w-full h-full"
      ref={windowRef}
      style={{ 
        overflow: "hidden", 
        position: "relative",
        width: "100%",
        height: "100vh",
        backgroundColor: "transparent" // Make the background transparent
      }}
    >
      <div
        className="absolute"
        style={{
          left: `${position.x}px`,
          top: `${position.y}px`,
          transition: "top 50ms linear, left 50ms linear", // Smooth movement
        }}
      >
        <div
          style={getSpriteStyle()}
          draggable="false"
        />
      </div>
    </div>
  );
}

export default App;