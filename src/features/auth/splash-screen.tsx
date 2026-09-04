import { useEffect, useState } from "react";
import logo from "@/assets/logo.svg";

interface SplashScreenProps {
  onComplete: () => void;
}

export function SplashScreen({ onComplete }: SplashScreenProps) {
  const [phase, setPhase] = useState<"enter" | "hold" | "exit">("enter");

  useEffect(() => {
    const holdTimer = setTimeout(() => setPhase("hold"), 600);
    const exitTimer = setTimeout(() => setPhase("exit"), 1400);
    const completeTimer = setTimeout(onComplete, 1800);

    return () => {
      clearTimeout(holdTimer);
      clearTimeout(exitTimer);
      clearTimeout(completeTimer);
    };
  }, [onComplete]);

  return (
    <div
      className={`flex h-full w-full flex-col items-center justify-center bg-gradient-to-br from-brand-50 via-white to-accent-50/30 ${
        phase === "exit" ? "animate-fade-out" : "animate-fade-in"
      }`}
    >
      <div className={`flex flex-col items-center ${phase === "exit" ? "" : "animate-scale-in"}`}>
        <div className="relative mb-8">
          <div className="absolute -inset-8 rounded-full bg-brand-500/5 blur-2xl" />
          <img
            src={logo}
            alt="Logo"
            className="relative h-28 w-28 drop-shadow-lg animate-float"
          />
        </div>

        <h1 className="mb-3 text-3xl font-bold tracking-tight text-brand-900">
          سجل الواردات والصادرات
        </h1>

        <div className="mt-2 h-0.5 w-16 rounded-full bg-gradient-to-r from-brand-400 via-accent-400 to-pink-500 opacity-60" />
      </div>
    </div>
  );
}
