import { useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

//Import PetSprite sheets 
import foxSpriteSheet from "./assets/Fox Sprite Sheet.png";
import catSpriteSheet from "./assets/Cat Sprite Sheet.png"; 


interface SettingsProps {
    isOpen: boolean;
    onClose: () => void;
    currentPet: string;
    isVisible: boolean;
    onPetChange: (pet: string) => void;
    onVisibilityChange: (visible: boolean) => void;
}

export default function Settings({
    isOpen,
    onClose,
    currentPet,
    isVisible,
    onPetChange,
    onVisibilityChange,
}: SettingsProps) {
    const pets = [
        { id: "fox", name: "fox", image: foxSpriteSheet},
        { id: "cat", name: "cat", image: catSpriteSheet}
    ];

    //Handle clicking outside to close
    useEffect(() => {
        const handleClickOutside = (e: MouseEvent) => {
            const target = e.target as HTMLElement;
            if (isOpen && !target.closest(".settings-panel")) {
                onClose();
            }
        };

        if(isOpen) {
            document.addEventListener("mousedown", handleClickOutside);
        }

        return () => {
            document.removeEventListener("mousedown", handleClickOutside);
        }
    }, [isOpen, onClose]);

    useEffect(() => {
        const handleKeyDown = (e: KeyboardEvent) => {
            if (e.key === "Escape" && isOpen) {
              onClose();
            }
        };


        if (isOpen){
            document.addEventListener("keydown", handleKeyDown);
        }

        return () => {
            document.removeEventListener("keydown", handleKeyDown);
        };
    }, [isOpen, onClose]);

    const handleReset = async () => {
        try {
            await invoke("reset_pet_position", {
              windowWidth: window.innerWidth,
              windowHeight: window.innerHeight
            });
        } catch (error) {
            console.error("Failed to reset pet position:", error);
        }
    };

    if (!isOpen) return null;

    return (
        <div className="fixed inset-0 flex items-center justify-center bg-black bg-opacity-50 z-50">
          <div className="settings-panel bg-white rounded-lg shadow-lg p-6 w-80">
            <div className="flex justify-between items-center mb-4">
              <h2 className="text-lg font-bold">Pet Settings</h2>
              <button
                onClick={onClose}
                className="text-gray-500 hover:text-gray-700"
              >
                ✕
              </button>
            </div>
    
            <div className="mb-4">
              <h3 className="font-medium mb-2">Select Pet</h3>
              <div className="grid grid-cols-3 gap-2">
                {pets.map((pet) => (
                  <div
                    key={pet.id}
                    className={`cursor-pointer p-2 rounded border ${
                      currentPet === pet.id ? "border-blue-500 bg-blue-50" : "border-gray-200"
                    }`}
                    onClick={() => onPetChange(pet.id)}
                  >
                    <div className="flex flex-col items-center">
                      <div 
                        className="w-12 h-12 bg-contain bg-center bg-no-repeat" 
                        style={{ 
                          backgroundImage: `url(${pet.image})`,
                          backgroundPosition: "0 0",
                          backgroundSize: "48px 48px"
                        }}
                      />
                      <span className="text-xs mt-1">{pet.name}</span>
                    </div>
                  </div>
                ))}
              </div>
            </div>
    
            <div className="mb-4">
              <label className="flex items-center cursor-pointer">
                <input
                  type="checkbox"
                  checked={isVisible}
                  onChange={(e) => onVisibilityChange(e.target.checked)}
                  className="mr-2"
                />
                <span>Show Pet</span>
              </label>
            </div>
    
            <div className="flex justify-between">
              <button
                onClick={handleReset}
                className="px-3 py-1 bg-gray-200 hover:bg-gray-300 rounded"
              >
                Reset Position
              </button>
              <button
                onClick={onClose}
                className="px-3 py-1 bg-blue-500 hover:bg-blue-600 text-white rounded"
              >
                Close
              </button>
            </div>
          </div>
        </div>
      );
}