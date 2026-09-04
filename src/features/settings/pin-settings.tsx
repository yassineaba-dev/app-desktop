import { useState, useEffect } from "react";
import { X, KeyRound, Loader2, CheckCircle2, Eye, EyeOff } from "lucide-react";
import { settingsCommands } from "@/db/commands";

interface Props {
  onClose: () => void;
}

export function PinSettingsDialog({ onClose }: Props) {
  const [pin, setPin] = useState("");
  const [confirmPin, setConfirmPin] = useState("");
  const [showPin, setShowPin] = useState(false);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [success, setSuccess] = useState(false);
  const [error, setError] = useState("");

  useEffect(() => {
    settingsCommands.getPinCode().then((p) => {
      if (p) setPin(p);
    }).finally(() => setLoading(false));
  }, []);

  const handleSave = async () => {
    setError("");
    if (!pin) {
      setError("يرجى إدخال رمز PIN");
      return;
    }
    if (pin.length !== 4) {
      setError("يجب أن يكون الرمز 4 أرقام");
      return;
    }
    if (pin !== confirmPin) {
      setError("الرمز غير متطابق");
      return;
    }

    setSaving(true);
    setSuccess(false);
    try {
      await settingsCommands.setPinCode(pin);
      setSuccess(true);
      setTimeout(() => setSuccess(false), 2000);
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40">
      <div className="bg-white rounded-2xl shadow-2xl w-full max-w-md mx-4 overflow-hidden">
        <div className="flex items-center justify-between px-6 py-4 border-b border-gray-100">
          <div className="flex items-center gap-2">
            <KeyRound className="w-5 h-5 text-brand-600" />
            <h2 className="text-base font-bold text-slate-900">رمز PIN لإعادة تعيين كلمة المرور</h2>
          </div>
          <button onClick={onClose} className="p-1.5 text-slate-400 hover:text-slate-600 hover:bg-slate-100 rounded-lg transition-colors">
            <X className="w-5 h-5" />
          </button>
        </div>

        <div className="px-6 py-5 space-y-4">
          {loading ? (
            <div className="flex items-center justify-center py-8">
              <Loader2 className="w-6 h-6 animate-spin text-slate-400" />
            </div>
          ) : (
            <>
              {error && (
                <div className="rounded-lg border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700">
                  {error}
                </div>
              )}

              <div>
                <label className="block text-sm font-medium text-slate-700 mb-1">رمز PIN (4 أرقام)</label>
                <div className="relative">
                  <input
                    type={showPin ? "text" : "password"}
                    dir="ltr"
                    inputMode="numeric"
                    maxLength={4}
                    placeholder="____"
                    value={pin}
                    onChange={(e) => { setPin(e.target.value.replace(/\D/g, "")); setError(""); }}
                    className="w-full rounded-lg border border-gray-200 px-3 py-2 pr-10 text-center text-lg font-mono tracking-[0.3em] focus:border-brand-500 focus:ring-2 focus:ring-brand-500/20 outline-none"
                  />
                  <button
                    type="button"
                    onClick={() => setShowPin(!showPin)}
                    className="absolute inset-y-0 left-0 flex items-center pl-3 text-gray-400 hover:text-gray-600"
                  >
                    {showPin ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
                  </button>
                </div>
              </div>

              <div>
                <label className="block text-sm font-medium text-slate-700 mb-1">تأكيد الرمز</label>
                <input
                  type={showPin ? "text" : "password"}
                  dir="ltr"
                  inputMode="numeric"
                  maxLength={4}
                  placeholder="____"
                  value={confirmPin}
                  onChange={(e) => { setConfirmPin(e.target.value.replace(/\D/g, "")); setError(""); }}
                  className="w-full rounded-lg border border-gray-200 px-3 py-2 text-center text-lg font-mono tracking-[0.3em] focus:border-brand-500 focus:ring-2 focus:ring-brand-500/20 outline-none"
                />
              </div>
            </>
          )}
        </div>

        <div className="px-6 py-4 border-t border-gray-100 flex items-center justify-end gap-2">
          {success && (
            <span className="flex items-center gap-1 text-sm text-green-600">
              <CheckCircle2 className="w-4 h-4" />
              تم الحفظ
            </span>
          )}
          <button onClick={onClose} className="px-4 py-2 text-sm font-medium text-slate-600 hover:bg-slate-100 rounded-lg transition-colors">
            إلغاء
          </button>
          <button onClick={handleSave} disabled={saving || loading} className="px-4 py-2 text-sm font-medium text-white bg-brand-600 hover:bg-brand-700 rounded-lg transition-colors disabled:opacity-50 flex items-center gap-2">
            {saving && <Loader2 className="w-4 h-4 animate-spin" />}
            حفظ
          </button>
        </div>
      </div>
    </div>
  );
}
