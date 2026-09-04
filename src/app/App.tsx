import { useEffect, useState, useCallback } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  SplashScreen,
  LoginPage,
  ForgotPasswordPage,
} from "@/features/auth";
import { RegistryPage } from "@/features/registry/registry-page";
import { useAuthStore } from "@/stores/auth-store";

type View =
  | "splash"
  | "login"
  | "forgot-password";

function App() {
  const [view, setView] = useState<View>("splash");
  const isAuthenticated = useAuthStore((s) => s.isAuthenticated);

  useEffect(() => {
    getCurrentWindow().setFocus().catch(() => {});
  }, []);

  const handleSplashComplete = useCallback(() => {
    setView("login");
  }, []);

  const handleForgotPassword = useCallback(() => {
    setView("forgot-password");
  }, []);

  const handleResetSuccess = useCallback(() => {
    setView("login");
  }, []);

  const handleBackToLogin = useCallback(() => {
    setView("login");
  }, []);

  if (view === "splash") {
    return <SplashScreen onComplete={handleSplashComplete} />;
  }

  if (isAuthenticated) {
    return <RegistryPage />;
  }

  if (view === "forgot-password") {
    return (
      <ForgotPasswordPage
        onBack={handleBackToLogin}
        onPinVerified={handleResetSuccess}
      />
    );
  }

  return <LoginPage onForgotPassword={handleForgotPassword} />;
}

export default App;
