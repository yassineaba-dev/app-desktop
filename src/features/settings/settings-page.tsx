import { useState } from "react";
import {
  Users,
  KeyRound,
  ChevronLeft,
  Settings,
  LogOut,
  Database,
} from "lucide-react";
import { useAuthStore } from "@/stores/auth-store";
import { AdminManagementDialog } from "./admin-management";
import { PinSettingsDialog } from "./pin-settings";
import { DatabaseSettingsDialog } from "./database-settings";

interface Props {
  onBack: () => void;
}

const params = [
  { id: "admins" as const, label: "المسؤولون", desc: "إدارة حسابات المسؤولين", Icon: Users },
  { id: "pin" as const, label: "رمز PIN", desc: "رمز إعادة تعيين كلمة المرور", Icon: KeyRound },
  { id: "database" as const, label: "قاعدة البيانات", desc: "نسخ قاعدة البيانات أو استخدام قاعدة بيانات من الجهاز", Icon: Database },
];

export function SettingsPage({ onBack }: Props) {
  const clearAuth = useAuthStore((s) => s.clearAuth);
  const [activeParam, setActiveParam] = useState<string | null>(null);

  return (
    <div className="flex flex-col h-full bg-gray-50">
      <header className="bg-white border-b border-gray-200 px-6 py-4 flex items-center justify-between shrink-0">
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 rounded-lg bg-slate-800 flex items-center justify-center">
            <Settings className="w-5 h-5 text-white" />
          </div>
          <h1 className="text-lg font-bold text-slate-900">الإعدادات</h1>
        </div>

        <div className="flex items-center gap-1">
          <button
            onClick={onBack}
            className="p-2 text-slate-400 hover:text-slate-700 hover:bg-slate-100 rounded-lg transition-colors"
            title="العودة"
          >
            <ChevronLeft className="w-5 h-5" />
          </button>
          <div className="w-px h-6 bg-gray-200 mx-1" />
          <button
            onClick={clearAuth}
            className="p-2 text-slate-400 hover:text-red-600 hover:bg-red-50 rounded-lg transition-colors"
            title="تسجيل الخروج"
          >
            <LogOut className="w-5 h-5" />
          </button>
        </div>
      </header>

      <div className="flex-1 overflow-auto">
        <div className="max-w-2xl mx-auto px-8 py-6 space-y-3">
          {params.map(({ id, label, desc, Icon }) => (
            <button
              key={id}
              onClick={() => setActiveParam(id)}
              className="w-full flex items-center gap-4 px-5 py-4 bg-white border border-gray-200 rounded-xl hover:border-gray-300 hover:shadow-sm transition-all text-right"
            >
              <div className="w-10 h-10 rounded-lg bg-gray-100 flex items-center justify-center shrink-0">
                <Icon className="w-5 h-5 text-slate-600" />
              </div>
              <div className="flex-1 min-w-0">
                <p className="text-sm font-bold text-slate-800">{label}</p>
                <p className="text-xs text-slate-400 mt-0.5">{desc}</p>
              </div>
              <ChevronLeft className="w-4 h-4 text-slate-300" />
            </button>
          ))}
        </div>
      </div>

      {activeParam === "admins" && (
        <AdminManagementDialog onClose={() => setActiveParam(null)} />
      )}
      {activeParam === "pin" && (
        <PinSettingsDialog onClose={() => setActiveParam(null)} />
      )}
      {activeParam === "database" && (
        <DatabaseSettingsDialog onClose={() => setActiveParam(null)} />
      )}
    </div>
  );
}
