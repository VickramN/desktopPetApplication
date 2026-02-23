import { useState, useEffect, useRef, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Window } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";
// Import sprite sheet
import spriteSheet from "./assets/Fox Sprite Sheet.png";
import catSpriteSheet from "./assets/Cat Sprite Sheet.png";
import redPandaSpriteSheet from "./assets/Red Panda Sprite Sheet.png";
import Settings from "./Settings";

// Constants for configuration
const DEFAULT_WINDOW_WIDTH = 1920;
const DEFAULT_WINDOW_HEIGHT = 1032;
const UPDATE_INTERVAL_MS = 50;

// Frame size and display constants - adjusted to match the sprite sheet
const FRAME_WIDTH = 32; // Actual pixel width of each frame
const FRAME_HEIGHT = 32; // Actual pixel height of each frame

// Animation Sequence coordinates
const CAT_ANIMATIONS = {
  idle: {
    frames: [
      [0, 0],
      [32, 0],
      [64, 0],
      [96, 0],
    ],
    frameDuration: 200,
  },
  run: {
    frames: [
      [0, 128],
      [32, 128],
      [64, 128],
      [96, 128],
      [128, 128],
      [160, 128],
      [192, 128],
      [224, 128],
    ],
    frameDuration: 100,
  },
  jump: {
    frames: [
      [0, 160],
      [32, 160],
      [64, 160],
    ],
    frameDuration: 150,
  },
  fall: {
    frames: [
      [96, 160],
      [128, 160],
      [128, 160],
      [160, 160],
    ],
    frameDuration: 150,
  },
} as const;

const FOX_ANIMATIONS = {
  idle: {
    frames: [
      [0, 0],
      [32, 0],
      [64, 0],
      [96, 0],
      [128, 0],
    ],
    frameDuration: 200,
  },
  run: {
    frames: [
      [0, 64],
      [32, 64],
      [64, 64],
      [96, 64],
      [128, 64],
      [160, 64],
      [192, 64],
      [224, 64],
    ],
    frameDuration: 100,
  },
  jump: {
    frames: [
      [0, 96],
      [32, 96],
      [64, 96],
      [96, 96],
      [128, 96],
    ],
    frameDuration: 150,
  },
  fall: {
    frames: [
      [128, 96],
      [160, 96],
      [192, 96],
      [224, 96],
      [256, 96],
      [288, 96],
      [320, 96],
    ],
    frameDuration: 150,
  },
} as const;

const RED_PANDA_ANIMATIONS = {
  idle: {
    frames: [
      [0, 32],
      [32, 32],
      [64, 32],
      [96, 32],
      [128, 32],
      [160, 32],
    ],
    frameDuration: 200,
  },
  run: {
    frames: [
      [0, 64],
      [32, 64],
      [64, 64],
      [96, 64],
      [128, 64],
      [160, 64],
      [192, 64],
      [224, 64],
    ],
    frameDuration: 100,
  },
  jump: {
    frames: [
      [0, 96],
      [32, 96],
      [64, 96],
      [32, 64],
      [64, 64],
    ],
    frameDuration: 150,
  },
  fall: {
    frames: [
      [96, 64],
      [128, 64],
    ],
    frameDuration: 100,
  },
} as const;

// Pet configuration mapping
const PET_CONFIG = {
  cat: { spriteSheet: catSpriteSheet, animations: CAT_ANIMATIONS },
  fox: { spriteSheet: spriteSheet, animations: FOX_ANIMATIONS },
  "red panda": {
    spriteSheet: redPandaSpriteSheet,
    animations: RED_PANDA_ANIMATIONS,
  },
} as const;

export type PetType = keyof typeof PET_CONFIG;

function App() {
  const [position, setPosition] = useState({ x: 0, y: 0 });
  const [windowSize, setWindowSize] = useState({
    width: DEFAULT_WINDOW_WIDTH,
    height: DEFAULT_WINDOW_HEIGHT,
  });
  const [isLoaded, setIsLoaded] = useState(false);
  const [isReady, setIsReady] = useState(false);
  const [animationState, setAnimationState] = useState("idle-right");
  const [frameIndex, setFrameIndex] = useState(0);
  const [currentPet, setCurrentPet] = useState<PetType>("cat");
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [isVisible, setIsVisible] = useState(true);

  //Ref
  const windowRef = useRef<HTMLDivElement>(null);
  const animationTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const positionUpdateIntervalRef = useRef<ReturnType<
    typeof setInterval
  > | null>(null);

  const currentConfig = PET_CONFIG[currentPet];
  const currentSpriteSheet = currentConfig.spriteSheet;
  const currentAnimations = currentConfig.animations;

  //settings Panel
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "." && (e.ctrlKey || e.metaKey)) {
        setSettingsOpen(true);
      }
    };
    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, []);

  // Get window size with memoized callback to prevent unnecessary renders
  const getWindowSize = useCallback(async () => {
    try {
      const appWindow = Window.getCurrent();
      const factor = (await appWindow.scaleFactor()) || 1;
      const innerSize = await appWindow.innerSize();

      const physicalSize = {
        width: Math.floor(innerSize.width / factor),
        height: Math.floor(innerSize.height / factor),
      };

      setWindowSize(physicalSize);
      setIsLoaded(true);

      if (physicalSize.width > 500 && physicalSize.height > 500) {
        setIsReady(true);
      }
    } catch (error) {
      console.error("Failed to get window size:", error);
      // Fall back to default dimensions
      setWindowSize({
        width: DEFAULT_WINDOW_WIDTH,
        height: DEFAULT_WINDOW_HEIGHT,
      });
      setIsLoaded(true);
    }
  }, []);

  // Setup window size listener
  useEffect(() => {
    getWindowSize();

    let unlisten: (() => void) | undefined;

    // Listen for window resize events
    const setupListener = async () => {
      try {
        const appWindow = Window.getCurrent();
        unlisten = await appWindow.listen("resize", getWindowSize);
      } catch (error) {
        console.error("Failed to set up resize listener:", error);
      }
    };

    setupListener();

    // Set up ResizeObserver as fallback for window size detection
    if (windowRef.current) {
      const resizeObserver = new ResizeObserver((entries) => {
        for (const entry of entries) {
          if (entry.contentRect) {
            const { width, height } = entry.contentRect;
            if (width > 0 && height > 0) {
              setWindowSize({ width, height });
            }
          }
        }
      });

      resizeObserver.observe(windowRef.current);

      return () => {
        if (unlisten) unlisten();
        resizeObserver.disconnect();
      };
    }

    return () => {
      if (unlisten) unlisten();
    };
  }, [getWindowSize]);

  // Animation frame timing effect
  useEffect(() => {
    if (!isLoaded) return;

    // Extract the base animation name without direction
    const baseAnimation = animationState.split(
      "-"
    )[0] as keyof typeof currentAnimations;
    const config = currentAnimations[baseAnimation];

    if (!config) return;

    // Clear any existing timer
    if (animationTimerRef.current) {
      clearTimeout(animationTimerRef.current);
    }

    // Set new timer for frame animation
    animationTimerRef.current = setTimeout(() => {
      setFrameIndex((prev) => (prev + 1) % config.frames.length);
    }, config.frameDuration);

    // Clean up on unmount
    return () => {
      if (animationTimerRef.current) {
        clearTimeout(animationTimerRef.current);
      }
    };
  }, [frameIndex, animationState, isLoaded, currentAnimations]);

  // Update pet position at regular intervals
  useEffect(() => {
    if (!isLoaded || windowSize.width <= 0 || windowSize.height <= 0) {
      return;
    }

    // Function to update position via Rust backend
    const updatePosition = async () => {
      try {
        const [x, y, state] = await invoke<[number, number, string]>(
          "get_pet_movement",
          {
            windowWidth: windowSize.width,
            windowHeight: windowSize.height,
          }
        );

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

    // Initial position update
    updatePosition();

    // Set interval for regular updates
    positionUpdateIntervalRef.current = setInterval(
      updatePosition,
      UPDATE_INTERVAL_MS
    );

    // Clean up on unmount
    return () => {
      if (positionUpdateIntervalRef.current) {
        clearInterval(positionUpdateIntervalRef.current);
      }
    };
  }, [windowSize, isLoaded, animationState]);

  // Toggle click-through based on settings state
  useEffect(() => {
    const toggleClickThrough = async () => {
      try {
        await invoke("set_click_through", { enabled: !settingsOpen });
      } catch (error) {
        console.error("Failed to toggle click-through:", error);
      }
    };

    if (isLoaded) {
      toggleClickThrough();
    }
  }, [settingsOpen, isLoaded]);

  //Tray Icon
  useEffect(() => {
    let unlisten: (() => void) | undefined;

    const setupTrayListener = async () => {
      try {
        unlisten = await listen("open-settings", () => {
          console.log("Opening settings from tray icon");
          setSettingsOpen(true);
        });
        console.log("Tray listener setup complete");
      } catch (error) {
        console.error("Failed to setup tray listener");
      }
    };
    setupTrayListener();

    return () => {
      if (unlisten) unlisten();
    };
  }, []);

  // Get the current frame from the animation sequence
  const getCurrentFrame = useCallback(() => {
    const baseAnimation = animationState.split(
      "-"
    )[0] as keyof typeof currentAnimations;
    const animation = currentAnimations[baseAnimation];

    if (!animation) return [0, 0];

    const safeIndex = Math.min(frameIndex, animation.frames.length - 1);
    return animation.frames[safeIndex];
  }, [animationState, frameIndex, currentAnimations]);

  // Calculate sprite style based on current frame
  const getSpriteStyle = useCallback(() => {
    const [x, y] = getCurrentFrame();
    const isFlipped = animationState.endsWith("-left");

    return {
      width: `${FRAME_WIDTH}px`,
      height: `${FRAME_HEIGHT}px`,
      backgroundImage: `url(${currentSpriteSheet})`,
      backgroundPosition: `-${x}px -${y}px`,
      backgroundSize: `${currentSpriteSheet.width}px ${currentSpriteSheet.height}px`,
      backgroundRepeat: "no-repeat",
      transform: isFlipped ? "scaleX(-1)" : "scaleX(1)",
      transformOrigin: "center",
      imageRendering: "pixelated" as const,
      willChange: "transform, background-position",
    };
  }, [getCurrentFrame, animationState, currentSpriteSheet]);

  // Reset pet position handler
  const handleReset = async () => {
    try {
      const [x, y, state] = await invoke<[number, number, string]>(
        "reset_pet_position",
        {
          windowWidth: windowSize.width,
          windowHeight: windowSize.height,
        }
      );

      setPosition({ x, y });
      setAnimationState(state);
      setFrameIndex(0);
    } catch (error) {
      console.error("Failed to reset pet position:", error);
    }
  };

  const handlePetChange = useCallback((petId: PetType) => {
    setCurrentPet(petId as PetType);
    setFrameIndex(0);
  }, []);

  return (
    <div
      className="w-full h-full"
      ref={windowRef}
      style={{
        overflow: "hidden",
        position: "relative",
        width: "100%",
        height: "100vh",
        backgroundColor: "transparent", // Make the background transparent
      }}
      onDoubleClick={handleReset} // Double click to reset pet position
    >
      {isReady && isVisible && (
        <div
          className="absolute"
          style={{
            left: `${position.x}px`,
            top: `${position.y}px`,
            transition: "top 50ms linear, left 50ms linear", // Smooth movement
          }}
        >
          <div style={getSpriteStyle()} draggable={false} />
        </div>
      )}
      <Settings
        isOpen={settingsOpen}
        onClose={() => setSettingsOpen(false)}
        currentPet={currentPet}
        isVisible={isVisible}
        onPetChange={handlePetChange}
        onVisibilityChange={setIsVisible}
      />
    </div>
  );
}

export default App;
