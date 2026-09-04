import { useState, useCallback } from "react";
import { Eye, EyeOff, Mail, Lock, ArrowLeft } from "lucide-react";
import logo from "@/assets/logo.svg";
import { authCommands } from "@/db/commands";
import { useAuthStore } from "@/stores/auth-store";

interface LoginPageProps {
  onForgotPassword: () => void;
}

export function LoginPage({ onForgotPassword }: LoginPageProps) {
  const [email, setEmail] = useState(() => localStorage.getItem("login-email") ?? "");
  const [password, setPassword] = useState("");
  const [showPassword, setShowPassword] = useState(false);
  const [errors, setErrors] = useState<{ email?: string; password?: string; general?: string }>({});
  const [isLoading, setIsLoading] = useState(false);
  const setAuth = useAuthStore((s) => s.setAuth);

  const validate = useCallback(() => {
    const newErrors: { email?: string; password?: string } = {};
    if (!email.trim()) newErrors.email = "يرجى إدخال البريد الإلكتروني";
    if (!password) newErrors.password = "يرجى إدخال كلمة المرور";
    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  }, [email, password]);

  const handleSubmit = useCallback(
    async (e: React.FormEvent) => {
      e.preventDefault();
      setErrors({});
      if (!validate()) return;

      setIsLoading(true);
      try {
        const response = await authCommands.login({ email, password });
        setAuth(response.user, response.token);
      } catch (err) {
        const message = typeof err === "string" ? err : "حدث خطأ غير متوقع";
        setErrors({ general: message });
      } finally {
        setIsLoading(false);
      }
    },
    [email, password, validate, setAuth],
  );

  return (
    <div className="flex h-full w-full bg-white">
      {/* Right side - Brand panel */}
      <div className="relative hidden w-[45%] overflow-hidden lg:flex">
        <div className="absolute inset-0 bg-gradient-to-br from-brand-700 via-brand-600 to-accent-600" />
        <div className="absolute inset-0 opacity-10">
          <div className="absolute -right-20 -top-20 h-96 w-96 rounded-full bg-white/20" />
          <div className="absolute -bottom-32 -left-32 h-[500px] w-[500px] rounded-full bg-white/10" />
          <div className="absolute right-1/4 top-1/3 h-40 w-40 rounded-full bg-pink-400/20" />
        </div>

        <div className="relative z-10 flex w-full flex-col items-center justify-center px-12 text-center text-white">
          <img src={logo} alt="Logo" className="mb-8 h-32 w-32 drop-shadow-2xl" />
          <h2 className="mb-4 text-center text-3xl font-bold">سجل الواردات والصادرات</h2>
          <p className="mt-2 text-center text-base text-white/80 leading-relaxed">
            نظام إلكتروني لتدبير وتوثيق
            <br />
            المراسلات الإدارية
          </p>
        </div>
      </div>

      {/* Left side - Login form */}
      <div className="flex w-full items-center justify-center px-6 sm:px-12 lg:w-[55%]">
        <div className="w-full max-w-md">
          <div className="mb-10 flex flex-col items-center lg:hidden">
            <img src={logo} alt="Logo" className="mb-4 h-16 w-16" />
            <h2 className="text-xl font-bold text-brand-900">سجل الواردات والصادرات</h2>
          </div>

          <div className="mb-10">
            <h1 className="mb-2 text-2xl font-bold text-gray-900">تسجيل الدخول</h1>
            <p className="text-sm text-gray-500">أدخل بياناتك للوصول إلى النظام</p>
          </div>

          {errors.general && (
            <div className="mb-6 rounded-lg border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700">
              {errors.general}
            </div>
          )}

          <form onSubmit={handleSubmit} className="space-y-5">
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
                  onChange={(e) => {
                    const v = e.target.value;
                    setEmail(v);
                    localStorage.setItem("login-email", v);
                    if (errors.email) setErrors((prev) => ({ ...prev, email: undefined }));
                  }}
                  className={`w-full rounded-xl border bg-gray-50/80 py-3 pr-11 pl-4 text-right text-sm text-gray-900 placeholder-gray-400 transition-all duration-200 focus:border-brand-500 focus:bg-white focus:outline-none focus:ring-2 focus:ring-brand-500/20 ${
                    errors.email ? "border-red-400 focus:border-red-500 focus:ring-red-500/20" : "border-gray-200"
                  }`}
                />
              </div>
              {errors.email && <p className="mt-1.5 text-xs text-red-500">{errors.email}</p>}
            </div>

            <div>
              <label htmlFor="password" className="mb-1.5 block text-sm font-medium text-gray-700">
                كلمة المرور
              </label>
              <div className="relative">
                <div className="pointer-events-none absolute inset-y-0 right-0 flex items-center pr-3.5">
                  <Lock className="h-4.5 w-4.5 text-gray-400" />
                </div>
                <input
                  id="password"
                  type={showPassword ? "text" : "password"}
                  dir="ltr"
                  autoComplete="new-password"
                  placeholder="أدخل كلمة المرور"
                  value={password}
                  onChange={(e) => {
                    setPassword(e.target.value);
                    if (errors.password) setErrors((prev) => ({ ...prev, password: undefined }));
                  }}
                  className={`w-full rounded-xl border bg-gray-50/80 py-3 pr-11 pl-11 text-right text-sm text-gray-900 placeholder-gray-400 transition-all duration-200 focus:border-brand-500 focus:bg-white focus:outline-none focus:ring-2 focus:ring-brand-500/20 ${
                    errors.password ? "border-red-400 focus:border-red-500 focus:ring-red-500/20" : "border-gray-200"
                  }`}
                />
                <button
                  type="button"
                  onClick={() => setShowPassword(!showPassword)}
                  className="absolute inset-y-0 left-0 flex items-center pl-3.5 text-gray-400 transition-colors hover:text-gray-600"
                >
                  {showPassword ? <EyeOff className="h-4.5 w-4.5" /> : <Eye className="h-4.5 w-4.5" />}
                </button>
              </div>
              {errors.password && <p className="mt-1.5 text-xs text-red-500">{errors.password}</p>}
            </div>

            <div className="flex items-center justify-between">
              <button
                type="button"
                onClick={onForgotPassword}
                className="text-sm text-brand-600 transition-colors hover:text-brand-700"
              >
                نسيت كلمة المرور؟
              </button>
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
                <span>تسجيل الدخول</span>
                <ArrowLeft className="h-4 w-4 transition-transform group-hover:-translate-x-0.5" />
              </div>
              {isLoading && (
                <div className="absolute inset-0 flex items-center justify-center">
                  <div className="flex items-center gap-2">
                    <div className="h-4 w-4 animate-spin rounded-full border-2 border-white/30 border-t-white" />
                    <span>جارٍ تسجيل الدخول...</span>
                  </div>
                </div>
              )}
            </button>
          </form>

          <p className="mt-8 text-center text-xs text-gray-400">
            جميع الحقوق محفوظة &copy; {new Date().getFullYear()}
          </p>
        </div>
      </div>
    </div>
  );
}
