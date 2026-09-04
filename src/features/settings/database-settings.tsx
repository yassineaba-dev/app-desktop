import { useState } from "react";
import { X, Upload, HardDriveDownload, Loader2, Database } from "lucide-react";
import { useQueryClient } from "@tanstack/react-query";
import { open } from "@tauri-apps/plugin-dialog";
import { databaseCommands } from "@/db/commands";

interface Props {
  onClose: () => void;
}

export function DatabaseSettingsDialog({ onClose }: Props) {
  const qc = useQueryClient();
  const [busy, setBusy] = useState<"export" | "import" | null>(null);
  const [message, setMessage] = useState<{ ok: boolean; text: string } | null>(null);

  const handleExport = async () => {
    setBusy("export");
    setMessage(null);
    try {
      const path = await databaseCommands.exportToDesktop();
      setMessage({ ok: true, text: `تم نسخ قاعدة البيانات إلى:\n${path}` });
    } catch (e) {
      setMessage({ ok: false, text: e instanceof Error ? e.message : String(e) });
    } finally {
      setBusy(null);
    }
  };

  const handleImport = async () => {
    setBusy("import");
    setMessage(null);
    try {
      const selected = await open({
        multiple: false,
        directory: true,
      });
      if (!selected || typeof selected !== "string") {
        setBusy(null);
        return;
      }
      const imported = await databaseCommands.importFromPath(selected);
      await qc.invalidateQueries();
      setMessage({
        ok: true,
        text: `تم استيراد ${imported} سجل بنجاح من قاعدة البيانات`,
      });
      setBusy(null);
    } catch (e) {
      setMessage({ ok: false, text: e instanceof Error ? e.message : String(e) });
      setBusy(null);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40">
      <div className="bg-white rounded-2xl shadow-2xl w-full max-w-md mx-4 overflow-hidden">
        <div className="flex items-center justify-between px-6 py-4 border-b border-gray-100">
          <div className="flex items-center gap-2">
            <Database className="w-5 h-5 text-brand-600" />
            <h2 className="text-base font-bold text-slate-900">قاعدة البيانات</h2>
          </div>
          <button
            onClick={onClose}
            className="p-1.5 text-slate-400 hover:text-slate-600 hover:bg-slate-100 rounded-lg transition-colors"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        <div className="px-6 py-5 space-y-4">
          {message && (
            <div
              className={`whitespace-pre-line rounded-lg border px-4 py-3 text-sm ${
                message.ok
                  ? "border-green-200 bg-green-50 text-green-700"
                  : "border-red-200 bg-red-50 text-red-700"
              }`}
            >
              {message.text}
            </div>
          )}

          <button
            onClick={handleExport}
            disabled={busy !== null}
            className="w-full flex items-center gap-3 px-4 py-3 rounded-xl border border-gray-200 hover:border-gray-300 hover:bg-gray-50 transition-colors text-right disabled:opacity-50"
          >
            <div className="w-10 h-10 rounded-lg bg-blue-50 flex items-center justify-center shrink-0">
              <HardDriveDownload className="w-5 h-5 text-blue-600" />
            </div>
            <div className="flex-1 min-w-0">
              <p className="text-sm font-bold text-slate-800">نسخ قاعدة البيانات إلى سطح المكتب</p>
              <p className="text-xs text-slate-400 mt-0.5">حفظ نسخة احتياطية من بياناتك وملفاتك المرفقة</p>
            </div>
            {busy === "export" && <Loader2 className="w-5 h-5 animate-spin text-slate-400 shrink-0" />}
          </button>

          <button
            onClick={handleImport}
            disabled={busy !== null}
            className="w-full flex items-center gap-3 px-4 py-3 rounded-xl border border-gray-200 hover:border-gray-300 hover:bg-gray-50 transition-colors text-right disabled:opacity-50"
          >
            <div className="w-10 h-10 rounded-lg bg-emerald-50 flex items-center justify-center shrink-0">
              <Upload className="w-5 h-5 text-emerald-600" />
            </div>
            <div className="flex-1 min-w-0">
              <p className="text-sm font-bold text-slate-800">استخدام قاعدة بيانات من الجهاز</p>
              <p className="text-xs text-slate-400 mt-0.5">اختر مجلد «بيانات مفلترة» أو ملف .db لاستيرادها مع الملفات المرفقة</p>
            </div>
            {busy === "import" && <Loader2 className="w-5 h-5 animate-spin text-slate-400 shrink-0" />}
          </button>
        </div>

        <div className="px-6 py-4 border-t border-gray-100 flex items-center justify-end">
          <button
            onClick={onClose}
            className="px-4 py-2 text-sm font-medium text-slate-600 hover:bg-slate-100 rounded-lg transition-colors"
          >
            إغلاق
          </button>
        </div>
      </div>
    </div>
  );
}
