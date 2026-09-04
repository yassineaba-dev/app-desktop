import { useState, useCallback } from "react";
import { Mail, ArrowRight, Shield, Lock, Eye, EyeOff } from "lucide-react";
import logo from "@/assets/logo.svg";
import { authCommands } from "@/db/commands";

interface ForgotPasswordPageProps {
  onBack: () => void;
  onPinVerified: (pin: string, email: string) => void;
}

export function ForgotPasswordPage({ onBack, onPinVerified }: ForgotPasswordPageProps) {
  const [step, setStep] = useState<"email" | "pin">("email");
  const [email, setEmail] = useState("");
  const [pin, setPin] = useState("");
  const [newPassword, setNewPassword] = useState("");
  const [showPassword, setShowPassword] = useState(false);
  const [error, setError] = useState("");
  const [success, setSuccess] = useState("");
  const [isLoading, setIsLoading] = useState(false);

  const handleEmailSubmit = useCallback(
    (e: React.FormEvent) => {
      e.preventDefault();
      setError("");
      if (!email.trim()) {
        setError("يرجى إدخال البريد الإلكتروني");
        return;
      }
      setStep("pin");
    },
    [email],
  );

  const handleResetSubmit = useCallback(
    async (e: React.FormEvent) => {
      e.preventDefault();
      setError("");
      setSuccess("");

      if (pin.length !== 4) {
        setError("يجب أن يكون الرمز 4 أرقام");
        return;
      }
      if (!newPassword) {
        setError("يرجى إدخال كلمة المرور الجديدة");
        return;
      }
      if (newPassword.length < 6) {
        setError("كلمة المرور يجب أن تكون 6 أحرف على الأقل");
        return;
      }

      setIsLoading(true);
      try {
        const msg = await authCommands.resetPasswordByPin(email, pin, newPassword);
        setSuccess(msg);
        setTimeout(() => onPinVerified(pin, email), 2000);
      } catch (err) {
        setError(typeof err === "string" ? err : "حدث خطأ غير متوقع");
      } finally {
        setIsLoading(false);
      }
    },
    [email, pin, newPassword, onPinVerified],
  );

  return (
    <div className="flex h-full w-full items-center justify-center bg-white px-6">
      <div className="w-full max-w-md">
        <div className="mb-10 flex flex-col items-center">
          <img src={logo} alt="Logo" className="mb-4 h-16 w-16" />
          <h2 className="text-xl font-bold text-brand-900">سجل الواردات والصادرات</h2>
        </div>

        <div className="mb-10 text-center">
          <div className="mx-auto mb-4 flex h-16 w-16 items-center justify-center rounded-full bg-brand-50">
            <Shield className="h-8 w-8 text-brand-600" />
          </div>
          <h1 className="mb-2 text-2xl font-bold text-gray-900">إعادة تعيين كلمة المرور</h1>
          {step === "email" ? (
            <p className="text-sm text-gray-500">أدخل بريدك الإلكتروني للمتابعة</p>
          ) : (
            <p className="text-sm text-gray-500">أدخل رمز PIN المكوّن من 4 أرقام</p>
          )}
        </div>

        {error && (
          <div className="mb-6 rounded-lg border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700">
            {error}
          </div>
        )}

        {success && (
          <div className="mb-6 rounded-lg border border-green-200 bg-green-50 px-4 py-3 text-sm text-green-700">
            {success}
          </div>
        )}

        {step === "email" ? (
          <form onSubmit={handleEmailSubmit} className="space-y-5">
            <div>
              <label htmlFor="email" className="mb-1.5 block text-sm font-medium text-gray-700">
                البريد الإلكتروني
              </label>
              <div className="relative">
                <div className="pointer-events-none absolute inset-y-0 right-0 flex items-center pr-3.5">
                  <Mail className="h-4.5 w-4.5 text-gray-400" />
                </div>
                <input
                  id="email"
                  type="email"
                  dir="ltr"
                  autoComplete="off"
                  placeholder="أدخل بريدك الإلكتروني"
                  value={email}
                  onChange={(e) => { setEmail(e.target.value); setError(""); }}
                  className="w-full rounded-xl border border-gray-200 bg-gray-50/80 py-3 pr-11 pl-4 text-right text-sm text-gray-900 placeholder-gray-400 transition-all duration-200 focus:border-brand-500 focus:bg-white focus:outline-none focus:ring-2 focus:ring-brand-500/20"
                />
              </div>
            </div>

            <button
              type="submit"
              className="group relative w-full overflow-hidden rounded-xl bg-gradient-to-r from-brand-600 via-brand-600 to-accent-600 py-3 text-sm font-semibold text-white shadow-lg shadow-brand-500/25 transition-all duration-200 hover:shadow-xl hover:shadow-brand-500/30 hover:brightness-110 active:scale-[0.98]"
            >
              <div className="flex items-center justify-center gap-2">
                <span>المتابعة</span>
                <ArrowRight className="h-4 w-4 transition-transform group-hover:translate-x-0.5" />
              </div>
            </button>
          </form>
        ) : (
          <form onSubmit={handleResetSubmit} className="space-y-5">
            <div>
              <label className="mb-1.5 block text-sm font-medium text-gray-700">رمز PIN</label>
              <div className="relative">
                <div className="pointer-events-none absolute inset-y-0 right-0 flex items-center pr-3.5">
                  <Lock className="h-4.5 w-4.5 text-gray-400" />
                </div>
                <input
                  type="password"
                  dir="ltr"
                  inputMode="numeric"
                  maxLength={4}
                  autoComplete="off"
                  placeholder="____"
                  value={pin}
                  onChange={(e) => { setPin(e.target.value.replace(/\D/g, "")); setError(""); }}
                  className="w-full rounded-xl border border-gray-200 bg-gray-50/80 py-3 pr-11 pl-4 text-center text-lg font-mono tracking-[0.3em] text-gray-900 placeholder-gray-300 transition-all duration-200 focus:border-brand-500 focus:bg-white focus:outline-none focus:ring-2 focus:ring-brand-500/20"
                />
              </div>
            </div>

            <div>
              <label className="mb-1.5 block text-sm font-medium text-gray-700">كلمة المرور الجديدة</label>
              <div className="relative">
                <div className="pointer-events-none absolute inset-y-0 right-0 flex items-center pr-3.5">
                  <Lock className="h-4.5 w-4.5 text-gray-400" />
                </div>
                <input
                  type={showPassword ? "text" : "password"}
                  dir="ltr"
                  autoComplete="new-password"
                  placeholder="أدخل كلمة المرور الجديدة"
                  value={newPassword}
                  onChange={(e) => { setNewPassword(e.target.value); setError(""); }}
                  className="w-full rounded-xl border border-gray-200 bg-gray-50/80 py-3 pr-11 pl-11 text-right text-sm text-gray-900 placeholder-gray-400 transition-all duration-200 focus:border-brand-500 focus:bg-white focus:outline-none focus:ring-2 focus:ring-brand-500/20"
                />
                <button
                  type="button"
                  onClick={() => setShowPassword(!showPassword)}
                  className="absolute inset-y-0 left-0 flex items-center pl-3.5 text-gray-400 transition-colors hover:text-gray-600"
                >
                  {showPassword ? <EyeOff className="h-4.5 w-4.5" /> : <Eye className="h-4.5 w-4.5" />}
                </button>
              </div>
            </div>

            <button
              type="submit"
              disabled={isLoading}
              className="group relative w-full overflow-hidden rounded-xl bg-gradient-to-r from-brand-600 via-brand-600 to-accent-600 py-3 text-sm font-semibold text-white shadow-lg shadow-brand-500/25 transition-all duration-200 hover:shadow-xl hover:shadow-brand-500/30 hover:brightness-110 active:scale-[0.98] disabled:cursor-not-allowed disabled:opacity-70"
            >
              <div
                className={`flex items-center justify-center gap-2 transition-all duration-200 ${
                  isLoading ? "opacity-0" : "opacity-100"
                }`}
              >
                <span>إعادة التعيين</span>
              </div>
              {isLoading && (
                <div className="absolute inset-0 flex items-center justify-center">
                  <div className="flex items-center gap-2">
                    <div className="h-4 w-4 animate-spin rounded-full border-2 border-white/30 border-t-white" />
                    <span>جارٍ المعالجة...</span>
                  </div>
                </div>
              )}
            </button>

            <button
              type="button"
              onClick={() => { setStep("email"); setPin(""); setNewPassword(""); setError(""); }}
              className="flex w-full items-center justify-center gap-2 rounded-xl border border-gray-200 py-3 text-sm font-medium text-gray-600 transition-colors hover:bg-gray-50"
            >
              <ArrowRight className="h-4 w-4" />
              العودة
            </button>
          </form>
        )}

        <button
          onClick={onBack}
          className="mt-6 flex w-full items-center justify-center gap-2 rounded-xl border border-gray-200 py-3 text-sm font-medium text-gray-600 transition-colors hover:bg-gray-50"
        >
          <ArrowRight className="h-4 w-4" />
          العودة إلى تسجيل الدخول
        </button>
      </div>
    </div>
  );
}
